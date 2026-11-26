//! @docs ARCHITECTURE:Intelligence
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Intelligence / Graph Community Clustering
//! - **Primary Entrypoints**: `compute_graph_communities`
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Deterministic Mulberry32 Label Propagation modularity assignment.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none declared
//! - **Telemetry Targets**: none declared

use super::models::{SymbolEdge, SymbolNode};
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;

const HUB_DEGREE_THRESHOLD: usize = 15;
const LPA_ITERATIONS: usize = 8;
const MULBERRY_SEED: u32 = 1337;

struct Mulberry32 {
    state: u32,
}

impl Mulberry32 {
    fn new(seed: u32) -> Self {
        Self { state: seed }
    }

    fn next_f64(&mut self) -> f64 {
        self.state = self.state.wrapping_add(0x6D2B_79F5);
        let mut t = self.state;
        t = (t ^ (t >> 15)).wrapping_mul(t | 1);
        t ^= t.wrapping_add((t ^ (t >> 7)).wrapping_mul(t | 61));
        let res = t ^ (t >> 14);
        (res as f64) / 4_294_967_296.0
    }

    fn next_index(&mut self, bound: usize) -> usize {
        if bound <= 1 {
            return 0;
        }
        ((self.next_f64() * (bound as f64)).floor() as usize).min(bound - 1)
    }
}

fn build_undirected_adjacency(graph: &DiGraph<SymbolNode, SymbolEdge>) -> Vec<Vec<usize>> {
    let n = graph.node_count();
    let mut adj = vec![Vec::new(); n];
    for edge in graph.raw_edges() {
        let u = edge.source().index();
        let v = edge.target().index();
        if u < n && v < n && u != v {
            adj[u].push(v);
            adj[v].push(u);
        }
    }
    adj
}

fn find_dominant_neighbor_labels(
    neighbors: &[usize],
    community: &[Option<u32>],
    is_hub: &[bool],
    skip_hubs: bool,
) -> Vec<u32> {
    let mut counts: HashMap<u32, usize> = HashMap::new();
    let mut max_count = 0usize;
    let mut best_labels = Vec::new();
    for &nb in neighbors {
        if skip_hubs && is_hub[nb] {
            continue;
        }
        let Some(lbl) = community[nb] else {
            continue;
        };
        let c = counts.entry(lbl).or_insert(0);
        *c += 1;
        if *c > max_count {
            max_count = *c;
            best_labels.clear();
            best_labels.push(lbl);
        } else if *c == max_count {
            best_labels.push(lbl);
        }
    }
    best_labels
}

fn run_lpa_pass(
    non_hub_indices: &mut [usize],
    adj: &[Vec<usize>],
    is_hub: &[bool],
    community: &mut [Option<u32>],
    rng: &mut Mulberry32,
) -> bool {
    for i in (1..non_hub_indices.len()).rev() {
        let j = rng.next_index(i + 1);
        non_hub_indices.swap(i, j);
    }
    let mut changed = false;
    for &u in non_hub_indices.iter() {
        let best_labels = find_dominant_neighbor_labels(&adj[u], community, is_hub, true);
        if best_labels.is_empty() {
            continue;
        }
        let current_lbl = community[u].unwrap_or(0);
        let best_lbl = if best_labels.contains(&current_lbl) {
            current_lbl
        } else {
            best_labels[rng.next_index(best_labels.len())]
        };
        if community[u] != Some(best_lbl) {
            community[u] = Some(best_lbl);
            changed = true;
        }
    }
    changed
}

fn reattach_hubs(
    adj: &[Vec<usize>],
    is_hub: &[bool],
    community: &mut [Option<u32>],
    mut counter: u32,
    rng: &mut Mulberry32,
) {
    for u in 0..is_hub.len() {
        if !is_hub[u] {
            continue;
        }
        let best_labels = find_dominant_neighbor_labels(&adj[u], community, is_hub, false);
        let chosen = if !best_labels.is_empty() {
            best_labels[rng.next_index(best_labels.len())]
        } else {
            let lbl = counter;
            counter += 1;
            lbl
        };
        community[u] = Some(chosen);
    }
}

fn assign_dense_ids(graph: &mut DiGraph<SymbolNode, SymbolEdge>, community: &[Option<u32>]) {
    let mut sizes: HashMap<u32, usize> = HashMap::new();
    for lbl in community.iter().flatten() {
        *sizes.entry(*lbl).or_insert(0) += 1;
    }
    let mut sorted_labels: Vec<u32> = sizes.keys().copied().collect();
    sorted_labels.sort_by(|a, b| sizes[b].cmp(&sizes[a]).then_with(|| a.cmp(b)));

    let label_to_new_id: HashMap<u32, u32> = sorted_labels
        .into_iter()
        .enumerate()
        .map(|(idx, lbl)| (lbl, idx as u32))
        .collect();

    for (u, comm_opt) in community.iter().enumerate() {
        let final_id = comm_opt
            .and_then(|lbl| label_to_new_id.get(&lbl).copied())
            .unwrap_or(0);
        graph[NodeIndex::new(u)].community = Some(final_id);
    }
}

/// Computes community modularity using a deterministic Label Propagation Algorithm (LPA)
/// seeded with Mulberry32 (`1337`), hub node exclusion (`degree > 15`), and majority-vote
/// hub re-attachment. Assigns dense sequential community IDs (`0..k`) ordered by size.
pub fn compute_graph_communities(scratch_graph: &mut DiGraph<SymbolNode, SymbolEdge>) {
    let n = scratch_graph.node_count();
    if n == 0 {
        return;
    }

    let mut rng = Mulberry32::new(MULBERRY_SEED);
    let adj = build_undirected_adjacency(scratch_graph);
    let is_hub: Vec<bool> = (0..n)
        .map(|i| adj[i].len() > HUB_DEGREE_THRESHOLD)
        .collect();

    let mut community: Vec<Option<u32>> = vec![None; n];
    let mut counter: u32 = 0;
    for i in 0..n {
        if !is_hub[i] {
            community[i] = Some(counter);
            counter += 1;
        }
    }

    let mut non_hub_indices: Vec<usize> = (0..n).filter(|&i| !is_hub[i]).collect();
    non_hub_indices.sort_by(|&a, &b| {
        let na = &scratch_graph[NodeIndex::new(a)];
        let nb = &scratch_graph[NodeIndex::new(b)];
        na.path.cmp(&nb.path).then_with(|| na.name.cmp(&nb.name))
    });

    for _ in 0..LPA_ITERATIONS {
        if !run_lpa_pass(
            &mut non_hub_indices,
            &adj,
            &is_hub,
            &mut community,
            &mut rng,
        ) {
            break;
        }
    }

    reattach_hubs(&adj, &is_hub, &mut community, counter, &mut rng);
    assign_dense_ids(scratch_graph, &community);
}
