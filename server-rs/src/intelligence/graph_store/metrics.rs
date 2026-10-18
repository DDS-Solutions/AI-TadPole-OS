//! @docs ARCHITECTURE:Core:Intelligence
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / graph_store / metrics
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Deterministic risk metric scoring, community cohesion calculation, and priority-ranked flow tracing.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `intelligence::graph_store::metrics::tests`

use super::heuristics::is_security_relevant;
use super::model::{CommunityRow, CommunityRule, EdgeKind, EdgeRow, FlowRow, NodeRow, RiskRow};
use std::collections::{HashMap, HashSet, VecDeque};

pub struct MetricEngine;

impl MetricEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn process(
        &self,
        nodes: &mut [NodeRow],
        edges: &[EdgeRow],
        rules: &[CommunityRule],
        max_flows: usize,
    ) -> (Vec<CommunityRow>, Vec<RiskRow>, Vec<FlowRow>) {
        assign_communities(nodes, rules);
        let communities = build_communities(nodes, edges, rules);
        let risks = build_risks(nodes, edges);
        let flows = build_flows(nodes, edges, &risks, max_flows);
        (communities, risks, flows)
    }
}

pub fn assign_communities(nodes: &mut [NodeRow], rules: &[CommunityRule]) {
    for node in nodes {
        let rel = node.file_path.replace('\\', "/");
        let mut assigned = false;
        for rule in rules {
            if rel.contains(&rule.pattern) {
                node.community_id = Some(rule.id);
                assigned = true;
                break;
            }
        }
        if !assigned {
            node.community_id = Some(7);
        }
    }
}

pub fn build_communities(
    nodes: &[NodeRow],
    edges: &[EdgeRow],
    rules: &[CommunityRule],
) -> Vec<CommunityRow> {
    let mut by_id: HashMap<i64, Vec<&NodeRow>> = HashMap::new();
    for node in nodes {
        by_id
            .entry(node.community_id.unwrap_or(7))
            .or_default()
            .push(node);
    }
    let mut rows = Vec::new();
    for (id, group) in by_id {
        let mut langs = HashMap::<String, usize>::new();
        for node in &group {
            *langs.entry(node.language.clone()).or_default() += 1;
        }
        let dominant_language = langs
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(lang, _)| lang)
            .unwrap_or_default();
        let names = group
            .iter()
            .map(|n| n.qualified_name.as_str())
            .collect::<HashSet<_>>();
        let internal = edges
            .iter()
            .filter(|e| {
                names.contains(e.source_qualified.as_str())
                    && names.contains(e.target_qualified.as_str())
            })
            .count();
        let cohesion = if group.len() <= 1 {
            0.0
        } else {
            (internal as f64 / group.len() as f64).min(1.0)
        };
        let name = rules
            .iter()
            .find(|r| r.id == id)
            .map(|r| r.name.as_str())
            .unwrap_or_else(|| {
                if id == 7 {
                    "workspace-other"
                } else {
                    "unknown-community"
                }
            });
        rows.push(CommunityRow {
            id,
            name: name.to_string(),
            cohesion,
            size: group.len() as i64,
            dominant_language,
            description: format!("{} symbols grouped by workspace area", group.len()),
            risk: "heuristic".to_string(),
        });
    }
    rows.sort_by_key(|row| row.id);
    rows
}

pub fn build_risks(nodes: &[NodeRow], edges: &[EdgeRow]) -> Vec<RiskRow> {
    let tested = edges
        .iter()
        .filter(|e| e.kind == EdgeKind::Tests)
        .map(|e| e.target_qualified.clone())
        .collect::<HashSet<_>>();
    let mut caller_counts = HashMap::<String, i64>::new();
    for edge in edges {
        if edge.kind.is_reference_or_call() {
            *caller_counts
                .entry(edge.target_qualified.clone())
                .or_default() += 1;
        }
    }
    nodes
        .iter()
        .filter(|node| node.kind != "File")
        .map(|node| {
            let caller_count = *caller_counts.get(&node.qualified_name).unwrap_or(&0);
            let is_tested = tested.contains(&node.qualified_name) || node.is_test;
            let security_relevant = is_security_relevant(node);
            let mut score = 0.15;
            if !is_tested {
                score += 0.2;
            }
            if security_relevant {
                score += 0.3;
            }
            if caller_count > 0 {
                score += ((caller_count as f64).log10() * 0.15).min(0.15);
            }
            if node.file_path.contains("\\routes\\") || node.file_path.contains("/routes/") {
                score += 0.05;
            }
            if node.kind == "Class" {
                score += 0.02;
            }
            RiskRow {
                node_id: node.id,
                qualified_name: node.qualified_name.clone(),
                risk_score: score.min(0.85),
                caller_count,
                test_coverage: if is_tested { "tested" } else { "untested" }.to_string(),
                security_relevant,
            }
        })
        .collect()
}

pub fn build_flows(
    nodes: &[NodeRow],
    edges: &[EdgeRow],
    risks: &[RiskRow],
    max_flows: usize,
) -> Vec<FlowRow> {
    let risk_by_qn = risks
        .iter()
        .map(|r| (r.qualified_name.as_str(), r))
        .collect::<HashMap<_, _>>();
    let id_by_qn = nodes
        .iter()
        .map(|n| (n.qualified_name.as_str(), n.id))
        .collect::<HashMap<_, _>>();
    let mut adjacency = HashMap::<&str, Vec<&str>>::new();
    for edge in edges {
        if edge.kind.is_reference_or_call() {
            adjacency
                .entry(edge.source_qualified.as_str())
                .or_default()
                .push(edge.target_qualified.as_str());
        }
    }
    let mut candidates = nodes
        .iter()
        .filter(|node| {
            node.kind != "File"
                && (node.name == "main"
                    || node.name.ends_with("_handler")
                    || node.file_path.contains("\\routes\\")
                    || node.file_path.contains("/routes/")
                    || node.file_path.contains("\\pages\\")
                    || node.file_path.contains("/pages/")
                    || node.file_path.contains("\\services\\")
                    || node.file_path.contains("/services/")
                    || node.file_path.contains("\\stores\\")
                    || node.file_path.contains("/stores/"))
        })
        .collect::<Vec<_>>();

    // Prioritize candidates by risk score and caller count before truncating to max_flows
    candidates.sort_by(|a, b| {
        let risk_a = risk_by_qn
            .get(a.qualified_name.as_str())
            .map(|r| r.risk_score)
            .unwrap_or(0.0);
        let risk_b = risk_by_qn
            .get(b.qualified_name.as_str())
            .map(|r| r.risk_score)
            .unwrap_or(0.0);
        risk_b
            .partial_cmp(&risk_a)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                let callers_a = risk_by_qn
                    .get(a.qualified_name.as_str())
                    .map(|r| r.caller_count)
                    .unwrap_or(0);
                let callers_b = risk_by_qn
                    .get(b.qualified_name.as_str())
                    .map(|r| r.caller_count)
                    .unwrap_or(0);
                callers_b.cmp(&callers_a)
            })
            .then_with(|| a.qualified_name.cmp(&b.qualified_name))
    });

    let mut entries = candidates.into_iter().take(max_flows).collect::<Vec<_>>();
    entries.sort_by(|a, b| a.qualified_name.cmp(&b.qualified_name));

    let mut flows = Vec::new();
    for (idx, entry) in entries.into_iter().enumerate() {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut parents = HashMap::new();

        let entry_qn = entry.qualified_name.as_str();
        visited.insert(entry_qn);
        queue.push_back((entry_qn, 0i64));

        while let Some((current, depth)) = queue.pop_front() {
            if depth >= 6 {
                continue;
            }
            for next in adjacency.get(current).into_iter().flatten().take(20) {
                if visited.insert(next) {
                    parents.insert(*next, current);
                    queue.push_back((next, depth + 1));
                }
            }
        }
        let node_ids = visited
            .iter()
            .filter_map(|qn| id_by_qn.get(qn).copied())
            .collect::<Vec<_>>();
        let files = nodes
            .iter()
            .filter(|node| visited.contains(node.qualified_name.as_str()))
            .map(|node| node.file_path.as_str())
            .collect::<HashSet<_>>();
        let security_hits = visited
            .iter()
            .filter(|qn| {
                risk_by_qn
                    .get(**qn)
                    .map(|r| r.security_relevant)
                    .unwrap_or(false)
            })
            .count();
        let criticality = ((node_ids.len() as f64 * 0.015)
            + (files.len() as f64 * 0.02)
            + (security_hits as f64 * 0.1))
            .min(1.0);

        let target_node = visited.iter().max_by(|a, b| {
            let r_a = risk_by_qn.get(*a).map(|r| r.risk_score).unwrap_or(0.0);
            let r_b = risk_by_qn.get(*b).map(|r| r.risk_score).unwrap_or(0.0);
            r_a.partial_cmp(&r_b).unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut path = Vec::new();
        if let Some(mut curr) = target_node.copied() {
            path.push(curr.to_string());
            while let Some(parent) = parents.get(curr) {
                path.push(parent.to_string());
                curr = *parent;
            }
            path.reverse();
        }
        let critical_path = path.into_iter().take(25).collect::<Vec<_>>();

        flows.push(FlowRow {
            id: idx as i64 + 1,
            name: entry.name.clone(),
            entry_point_id: entry.id,
            entry_point: entry.qualified_name.clone(),
            depth: 6,
            node_count: node_ids.len() as i64,
            node_ids,
            critical_path,
            criticality,
            file_count: files.len() as i64,
        });
    }
    flows
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_node(id: i64, name: &str, file_path: &str) -> NodeRow {
        NodeRow {
            id,
            kind: "Function".to_string(),
            name: name.to_string(),
            qualified_name: name.to_string(),
            file_path: file_path.to_string(),
            line_start: Some(1),
            line_end: Some(10),
            language: "rust".to_string(),
            parent_name: None,
            params: None,
            return_type: None,
            modifiers: None,
            is_test: false,
            file_hash: "hash".to_string(),
            extra: "{}".to_string(),
            signature: format!("fn {name}()"),
            community_id: None,
        }
    }

    fn test_edge(source: &str, target: &str, file_path: &str, kind: EdgeKind) -> EdgeRow {
        EdgeRow {
            kind,
            source_qualified: source.to_string(),
            target_qualified: target.to_string(),
            file_path: file_path.to_string(),
            line: 2,
            extra: "{}".to_string(),
        }
    }

    fn test_risk(node_id: i64, name: &str, risk_score: f64, caller_count: i64) -> RiskRow {
        RiskRow {
            node_id,
            qualified_name: name.to_string(),
            risk_score,
            caller_count,
            test_coverage: "untested".to_string(),
            security_relevant: false,
        }
    }

    #[test]
    fn test_build_flows_happy_path() {
        let nodes = vec![
            test_node(1, "main", "main.rs"),
            test_node(2, "service_func", "service.rs"),
            test_node(3, "repo_func", "repo.rs"),
        ];

        let edges = vec![
            test_edge("main", "service_func", "main.rs", EdgeKind::Calls),
            test_edge("service_func", "repo_func", "service.rs", EdgeKind::Calls),
        ];

        let risks = vec![
            test_risk(1, "main", 0.10, 0),
            test_risk(2, "service_func", 0.20, 1),
            test_risk(3, "repo_func", 0.30, 1),
        ];

        let flows = build_flows(&nodes, &edges, &risks, 250);
        assert_eq!(flows.len(), 1);
        let flow = &flows[0];
        assert_eq!(flow.name, "main");
        assert_eq!(flow.entry_point, "main");
        assert_eq!(flow.node_count, 3);
        assert_eq!(flow.file_count, 3);
        assert_eq!(
            flow.critical_path,
            vec!["main", "service_func", "repo_func"]
        );
    }

    #[test]
    fn test_build_flows_disconnected_and_circular() {
        let nodes = vec![
            test_node(1, "main", "/routes/main.rs"),
            test_node(2, "service_func", "/services/service.rs"),
        ];

        let edges = vec![
            test_edge("main", "service_func", "/routes/main.rs", EdgeKind::Calls),
            test_edge(
                "service_func",
                "main",
                "/services/service.rs",
                EdgeKind::Calls,
            ),
        ];

        let risks = vec![
            test_risk(1, "main", 0.15, 1),
            test_risk(2, "service_func", 0.15, 1),
        ];

        let flows = build_flows(&nodes, &edges, &risks, 250);
        assert_eq!(flows.len(), 2);
    }

    #[test]
    fn test_build_flows_empty_and_boundaries() {
        let flows = build_flows(&[], &[], &[], 250);
        assert!(flows.is_empty());

        let nodes = vec![test_node(1, "main", "main.rs")];
        let risks = vec![test_risk(1, "main", 0.15, 0)];
        let flows = build_flows(&nodes, &[], &risks, 250);
        assert_eq!(flows.len(), 1);
        assert_eq!(flows[0].node_count, 1);
        assert_eq!(flows[0].file_count, 1);
        assert_eq!(flows[0].criticality, 0.035);
    }
}
