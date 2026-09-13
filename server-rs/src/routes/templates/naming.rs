//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / templates / naming
//! - **Primary Entrypoints**: `apply_swarm_namespace`, `apply_workflow_namespace`, `sanitize_agent_filename`, `sanitize_workflow_filename`
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Functions here never return an unsanitized name. Path traversal components (../, /) are strictly eliminated.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::BadRequest`, `AppError::Forbidden`
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `routes::templates::naming::tests::*`

use crate::error::AppError;
use std::path::Path;

/// Sanitizes an error message display string to prevent log forging and control character injection.
pub fn sanitize_error_str(input: &str) -> String {
    let clean: String = input
        .chars()
        .filter(|c| !c.is_control() && *c != '\n' && *c != '\r')
        .take(128)
        .collect();
    if clean.is_empty() {
        "unknown".to_string()
    } else {
        clean
    }
}

/// Sanitizes an incoming raw agent filename, ensuring .json extension and path-traversal safety.
pub fn sanitize_agent_filename(raw_filename: &str) -> Result<String, AppError> {
    if !raw_filename.to_lowercase().ends_with(".json") {
        return Err(AppError::Forbidden(format!(
            "Agent asset '{}' must use the .json extension",
            sanitize_error_str(raw_filename)
        )));
    }

    let stem = raw_filename
        .strip_suffix(".json")
        .or_else(|| raw_filename.strip_suffix(".JSON"))
        .unwrap_or(raw_filename);

    let path_stem = Path::new(stem)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(stem);

    let clean_base = crate::utils::security::sanitize_id(path_stem);
    if clean_base.is_empty() {
        return Err(AppError::BadRequest(format!(
            "Invalid agent filename '{}'",
            sanitize_error_str(raw_filename)
        )));
    }

    debug_assert!(!clean_base.contains(['/', '\\']) && !clean_base.contains(".."));
    Ok(clean_base)
}

/// Sanitizes an incoming raw workflow filename, ensuring .md extension and path-traversal safety.
pub fn sanitize_workflow_filename(raw_filename: &str) -> Result<String, AppError> {
    if !raw_filename.to_lowercase().ends_with(".md") {
        return Err(AppError::Forbidden(format!(
            "Workflow asset '{}' must use the .md extension",
            sanitize_error_str(raw_filename)
        )));
    }

    let stem = raw_filename
        .strip_suffix(".md")
        .or_else(|| raw_filename.strip_suffix(".MD"))
        .unwrap_or(raw_filename);

    let path_stem = Path::new(stem)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(stem);

    let clean_base = crate::utils::security::sanitize_id(path_stem);
    if clean_base.is_empty() {
        return Err(AppError::BadRequest(format!(
            "Invalid workflow filename '{}'",
            sanitize_error_str(raw_filename)
        )));
    }

    debug_assert!(!clean_base.contains(['/', '\\']) && !clean_base.contains(".."));
    Ok(clean_base)
}

/// Applies an optional swarm namespace to an agent and its filename.
/// Invariant: Guarantees returned names and IDs are strictly sanitized and path-safe.
pub fn apply_swarm_namespace(
    agent: &mut crate::agent::types::EngineAgent,
    clean_base: &str,
    namespace: Option<&str>,
) -> (String, String) {
    let clean_base = crate::utils::security::sanitize_id(clean_base);
    let mut clean_id = crate::utils::security::sanitize_id(&agent.identity.id);
    if clean_id.is_empty() {
        clean_id = clean_base.clone();
    }
    agent.identity.id = clean_id.clone();

    let Some(ns) = namespace.map(|s| s.trim()).filter(|s| !s.is_empty()) else {
        return (clean_id, format!("{}.json", clean_base));
    };

    let clean_ns = crate::utils::security::sanitize_id(ns);
    if clean_ns.is_empty() {
        return (clean_id, format!("{}.json", clean_base));
    }

    let namespaced_id = format!("{}__{}", clean_ns, clean_id);
    agent.identity.id = namespaced_id.clone();
    let namespaced_filename = format!("{}__{}.json", clean_ns, clean_base);
    (namespaced_id, namespaced_filename)
}

/// Applies an optional workflow namespace to a workflow clean base name.
/// Invariant: Guarantees returned filenames are strictly sanitized and path-safe.
pub fn apply_workflow_namespace(clean_base: &str, namespace: Option<&str>) -> String {
    let stem = clean_base
        .strip_suffix(".md")
        .or_else(|| clean_base.strip_suffix(".MD"))
        .unwrap_or(clean_base);
    let path_stem = Path::new(stem)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(stem);
    let clean_stem = crate::utils::security::sanitize_id(path_stem);
    let safe_stem = if clean_stem.is_empty() {
        "workflow".to_string()
    } else {
        clean_stem
    };

    let Some(ns) = namespace.map(|s| s.trim()).filter(|s| !s.is_empty()) else {
        return format!("{}.md", safe_stem);
    };

    let clean_ns = crate::utils::security::sanitize_id(ns);
    if clean_ns.is_empty() {
        return format!("{}.md", safe_stem);
    }

    format!("{}__{}.md", clean_ns, safe_stem)
}

/// Computes an isolated installed directory name for a swarm given its safe name and optional namespace.
pub fn get_installed_swarm_id(safe_name: &str, namespace: Option<&str>) -> String {
    let clean_safe = crate::utils::security::sanitize_id(safe_name);
    if let Some(ns) = namespace.map(|s| s.trim()).filter(|s| !s.is_empty()) {
        let clean_ns = crate::utils::security::sanitize_id(ns);
        if !clean_ns.is_empty() {
            return format!("{}__{}", clean_ns, clean_safe);
        }
    }
    clean_safe
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    fn test_sanitize_id_contract_invariants() {
        use crate::utils::security::sanitize_id;

        // 1. Idempotence: sanitize_id(sanitize_id(x)) == sanitize_id(x)
        let sample = "../../evil/path/name..json";
        assert_eq!(sanitize_id(&sanitize_id(sample)), sanitize_id(sample));

        // 2. Elimination of path traversal separators
        let sanitized = sanitize_id("../../etc/passwd");
        assert!(!sanitized.contains('/'));
        assert!(!sanitized.contains('\\'));
        assert!(!sanitized.contains(".."));

        // 3. Preservation of double underscore namespace delimiter
        let namespaced = sanitize_id("mkt__lead_gen");
        assert_eq!(namespaced, "mkt__lead_gen");

        // 4. Empty output handling
        assert_eq!(sanitize_id(""), "");
        assert_eq!(sanitize_id("///"), "");
    }

    #[test]
    fn test_sanitize_agent_filename() {
        assert_eq!(sanitize_agent_filename("analyst.json").unwrap(), "analyst");
        assert_eq!(sanitize_agent_filename("ANALYST.JSON").unwrap(), "ANALYST");
        assert_eq!(sanitize_agent_filename("../../evil.json").unwrap(), "evil");
        assert_eq!(
            sanitize_agent_filename("path/to/my_agent.json").unwrap(),
            "my_agent"
        );
        assert!(sanitize_agent_filename("bad.py").is_err());
        assert!(sanitize_agent_filename(".json").is_err());
    }

    #[test]
    fn test_sanitize_workflow_filename() {
        assert_eq!(
            sanitize_workflow_filename("daily_sync.md").unwrap(),
            "daily_sync"
        );
        assert_eq!(
            sanitize_workflow_filename("DAILY_SYNC.MD").unwrap(),
            "DAILY_SYNC"
        );
        assert_eq!(
            sanitize_workflow_filename("../../../evil.md").unwrap(),
            "evil"
        );
        assert!(sanitize_workflow_filename("evil.txt").is_err());
        assert!(sanitize_workflow_filename(".md").is_err());
    }

    #[test]
    fn test_apply_swarm_namespace() {
        let agent_json = serde_json::json!({
            "id": "dispatcher",
            "name": "Field Dispatcher",
            "role": "Coordinator",
            "department": "Operations",
            "description": "Dispatches trucks",
            "status": "active"
        });
        let mut agent: crate::agent::types::EngineAgent =
            serde_json::from_value(agent_json).unwrap();

        let (id, filename) =
            apply_swarm_namespace(&mut agent, "dispatcher", Some("field_services"));
        assert_eq!(id, "field_services__dispatcher");
        assert_eq!(filename, "field_services__dispatcher.json");
        assert_eq!(agent.identity.id, "field_services__dispatcher");

        let (id_unchanged, file_unchanged) = apply_swarm_namespace(&mut agent, "dispatcher", None);
        assert_eq!(id_unchanged, "field_services__dispatcher");
        assert_eq!(file_unchanged, "dispatcher.json");
    }

    #[test]
    fn test_apply_workflow_namespace() {
        let namespaced = apply_workflow_namespace("daily_dispatch", Some("field_services"));
        assert_eq!(namespaced, "field_services__daily_dispatch.md");

        let traversal = apply_workflow_namespace("../../secret.md", Some("mkt"));
        assert_eq!(traversal, "mkt__secret.md");

        let plain = apply_workflow_namespace("routine.md", None);
        assert_eq!(plain, "routine.md");
    }
}
