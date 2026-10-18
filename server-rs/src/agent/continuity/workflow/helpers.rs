//! @docs ARCHITECTURE:Continuity:Workflow
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / helpers
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use super::types::{ConditionOperator, RoutingRule, RuleCondition, WorkflowStep};
use crate::error::AppError;
use std::collections::HashSet;
use std::sync::OnceLock;

pub fn get_concurrency_limit() -> usize {
    std::env::var("WORKFLOW_CONCURRENCY_LIMIT")
        .ok()
        .and_then(|val| val.parse().ok())
        .unwrap_or(5)
}

pub fn get_agent_timeout_duration() -> std::time::Duration {
    let secs = std::env::var("WORKFLOW_AGENT_TIMEOUT_SECS")
        .ok()
        .and_then(|val| val.parse().ok())
        .unwrap_or(600);
    std::time::Duration::from_secs(secs)
}

pub fn get_all_downstream(targets: &[String], steps: &[WorkflowStep]) -> HashSet<String> {
    let mut downstream = HashSet::new();
    let mut queue: Vec<String> = targets.to_vec();

    while let Some(current_id) = queue.pop() {
        for step in steps {
            let depends = step.depends_on.iter().any(|dep_id| dep_id == &current_id);
            if depends && downstream.insert(step.id.clone()) {
                queue.push(step.id.clone());
            }
        }
    }
    downstream
}

pub fn resolve_step_target(target: &str, steps: &[WorkflowStep]) -> Option<String> {
    let sanitized_target = sanitize_context_key(target);
    for step in steps {
        if step.id == target
            || step.name == target
            || sanitize_context_key(&step.name) == sanitized_target
        {
            return Some(step.id.clone());
        }
    }
    None
}

pub fn get_terminal_steps(steps: &[WorkflowStep]) -> Vec<&WorkflowStep> {
    let mut non_terminals = HashSet::new();
    for step in steps {
        for dep in &step.depends_on {
            non_terminals.insert(dep.as_str());
        }
    }
    steps
        .iter()
        .filter(|s| {
            !non_terminals.contains(s.id.as_str()) && !non_terminals.contains(s.name.as_str())
        })
        .collect()
}

pub fn evaluate_routing_rules(
    context: &parking_lot::Mutex<serde_json::Value>,
    rules: &[RoutingRule],
) -> Option<Vec<String>> {
    let ctx_lock = context.lock();
    for rule in rules {
        if evaluate_condition(&ctx_lock, &rule.condition) {
            return Some(rule.reset_steps.clone());
        }
    }
    None
}

pub fn get_context_as_string(
    context: &parking_lot::Mutex<serde_json::Value>,
) -> Result<String, AppError> {
    let ctx_lock = context.lock();
    serde_json::to_string(&*ctx_lock)
        .map_err(|e| AppError::InternalServerError(format!("Failed to serialize context: {}", e)))
}

pub fn get_fan_out_items(
    context: &parking_lot::Mutex<serde_json::Value>,
    array_path: &str,
) -> Vec<serde_json::Value> {
    let ctx_lock = context.lock();
    let mut current = &*ctx_lock;
    for part in array_path.split('.') {
        if let Some(next) = current.get(part) {
            current = next;
        } else {
            break;
        }
    }
    current.as_array().cloned().unwrap_or_default()
}

pub fn resolve_fan_out_prompt(
    context: &parking_lot::Mutex<serde_json::Value>,
    template: &str,
    placeholder: &str,
    item_str: &str,
) -> String {
    let ctx_lock = context.lock();
    let partial = substitute_placeholders(template, &ctx_lock);
    partial.replace(&format!("{{{{{}}}}}", placeholder), item_str)
}

pub fn resolve_tournament_prompt(
    context: &parking_lot::Mutex<serde_json::Value>,
    template: &str,
) -> String {
    let ctx_lock = context.lock();
    substitute_placeholders(template, &ctx_lock)
}

pub fn resolve_step_prompts(
    context: &parking_lot::Mutex<serde_json::Value>,
    template: &str,
) -> (String, Option<String>) {
    let ctx_lock = context.lock();
    let prompt = substitute_placeholders(template, &ctx_lock);
    let goal = ctx_lock
        .get("primary_goal")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    (prompt, goal)
}

pub fn sanitize_context_key(name: &str) -> String {
    let mut key = String::new();
    for c in name.chars() {
        if c.is_ascii_alphanumeric() || c == '_' {
            key.push(c.to_ascii_lowercase());
        } else if c == ' ' || c == '-' {
            key.push('_');
        }
    }

    if let Some(first) = key.chars().next() {
        if first.is_ascii_digit() {
            key = format!("_{}", key);
        }
    }

    if key.is_empty() {
        key = "_step".to_string();
    }

    key
}

pub fn insert_context_key(
    context: &parking_lot::Mutex<serde_json::Value>,
    key: String,
    value: String,
) -> Result<(), AppError> {
    static REGEX: OnceLock<regex::Regex> = OnceLock::new();
    let re = REGEX.get_or_init(|| regex::Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap());

    if !re.is_match(&key) {
        return Err(AppError::BadRequest(format!(
            "Invalid context key style: '{}'. Must match ^[a-zA-Z_][a-zA-Z0-9_]*$",
            key
        )));
    }

    if value.contains("{{") || value.contains("}}") {
        return Err(AppError::BadRequest(format!(
            "Context value for key '{}' contains forbidden template delimiters '{{{{' or '}}}}'",
            key
        )));
    }

    let mut ctx_lock = context.lock();
    if let Some(obj) = ctx_lock.as_object_mut() {
        obj.insert(key, serde_json::Value::String(value));
        Ok(())
    } else {
        Err(AppError::InternalServerError(
            "Context is not a JSON object".to_string(),
        ))
    }
}

pub fn detect_dependency_cycle(steps: &[WorkflowStep]) -> Result<(), AppError> {
    let mut adj = vec![Vec::new(); steps.len()];
    let mut in_degree = vec![0; steps.len()];

    for (v_idx, step) in steps.iter().enumerate() {
        let mut unique_deps = HashSet::new();
        for dep_id in &step.depends_on {
            if !unique_deps.insert(dep_id) {
                continue;
            }
            let u_idx_opt = steps.iter().position(|s| s.id == *dep_id);

            if let Some(u_idx) = u_idx_opt {
                adj[u_idx].push(v_idx);
                in_degree[v_idx] += 1;
            } else {
                return Err(AppError::BadRequest(format!(
                    "Step '{}' depends on non-existent step '{}'",
                    step.name, dep_id
                )));
            }
        }
    }

    let mut queue = Vec::new();
    for (idx, &deg) in in_degree.iter().enumerate() {
        if deg == 0 {
            queue.push(idx);
        }
    }

    let mut visited_count = 0;
    while let Some(u) = queue.pop() {
        visited_count += 1;
        for &v in &adj[u] {
            in_degree[v] -= 1;
            if in_degree[v] == 0 {
                queue.push(v);
            }
        }
    }

    if visited_count < steps.len() {
        return Err(AppError::BadRequest(
            "Workflow contains circular dependencies (cycle detected)".to_string(),
        ));
    }

    Ok(())
}

pub fn substitute_placeholders(template: &str, context: &serde_json::Value) -> String {
    static REGEX: OnceLock<regex::Regex> = OnceLock::new();
    let re = REGEX.get_or_init(|| regex::Regex::new(r"\{\{([a-zA-Z_][a-zA-Z0-9_]*)\}\}").unwrap());

    re.replace_all(template, |caps: &regex::Captures| {
        let key = &caps[1];
        if let Some(val) = context.get(key) {
            if let Some(s) = val.as_str() {
                s.to_string()
            } else {
                val.to_string()
            }
        } else {
            caps[0].to_string()
        }
    })
    .into_owned()
}

pub fn evaluate_condition(context: &serde_json::Value, condition: &RuleCondition) -> bool {
    let mut current = context;
    for part in condition.path.split('.') {
        if part == "__proto__" || part == "constructor" || part == "prototype" {
            return false;
        }
        if let Some(next) = current.get(part) {
            current = next;
        } else {
            return false;
        }
    }

    match condition.operator {
        ConditionOperator::Equals => current == &condition.value,
        ConditionOperator::NotEquals => current != &condition.value,
        ConditionOperator::Contains => {
            if let Some(arr) = current.as_array() {
                arr.contains(&condition.value)
            } else if let Some(s) = current.as_str() {
                if let Some(val_str) = condition.value.as_str() {
                    s.contains(val_str)
                } else {
                    false
                }
            } else {
                false
            }
        }
        ConditionOperator::GreaterThan => current
            .as_f64()
            .zip(condition.value.as_f64())
            .map(|(a, b)| a > b)
            .unwrap_or(false),
        ConditionOperator::LessThan => current
            .as_f64()
            .zip(condition.value.as_f64())
            .map(|(a, b)| a < b)
            .unwrap_or(false),
    }
}

pub fn prune_step_output(output: &str, config: &Option<serde_json::Value>) -> String {
    let Some(cfg) = config else {
        return output.to_string();
    };

    let mut result = output.to_string();

    if let Some(keys) = cfg.get("context_keys").and_then(|v| v.as_array()) {
        if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&result) {
            let mut extracted = serde_json::Map::new();
            for key_val in keys {
                if let Some(key_str) = key_val.as_str() {
                    if let Some(v) = json_val.get(key_str) {
                        extracted.insert(key_str.to_string(), v.clone());
                    }
                }
            }
            if !extracted.is_empty() {
                result = serde_json::Value::Object(extracted).to_string();
            }
        }
    } else if let Some(path) = cfg.get("context_json_path").and_then(|v| v.as_str()) {
        if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&result) {
            let mut current = &json_val;
            let mut found = true;
            for part in path.split('.') {
                if let Some(next) = current.get(part) {
                    current = next;
                } else {
                    found = false;
                    break;
                }
            }
            if found {
                if let Some(s) = current.as_str() {
                    result = s.to_string();
                } else {
                    result = current.to_string();
                }
            }
        }
    }

    if let Some(limit) = cfg.get("context_max_chars").and_then(|v| v.as_i64()) {
        if limit > 0 && result.chars().count() > limit as usize {
            let truncated: String = result.chars().take(limit as usize).collect();
            result = format!(
                "{}... [Truncated: exceeded context limit of {} chars]",
                truncated, limit
            );
        }
    }

    result
}

/// Maximum characters for candidate output injected into a judge prompt.
/// Prevents context-window overflow in the judge LLM.
const MAX_CANDIDATE_OUTPUT_CHARS: usize = 8_000;

/// Truncates candidate output for judge prompt injection.
/// If the output exceeds `MAX_CANDIDATE_OUTPUT_CHARS`, it is truncated with
/// a suffix indicating the truncation.
pub fn truncate_candidate_output(output: &str) -> String {
    if output.chars().count() <= MAX_CANDIDATE_OUTPUT_CHARS {
        output.to_string()
    } else {
        let truncated: String = output.chars().take(MAX_CANDIDATE_OUTPUT_CHARS).collect();
        format!(
            "{}... [Truncated: candidate output exceeded {} chars]",
            truncated, MAX_CANDIDATE_OUTPUT_CHARS
        )
    }
}
