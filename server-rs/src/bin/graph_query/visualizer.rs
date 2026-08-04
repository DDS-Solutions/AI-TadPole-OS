//! @docs ARCHITECTURE:CodeBaseIntelligence
//!
//! ### AI Assist Note
//! **Visualizer**: Generates interactive HTML and Mermaid representations of the blast radius.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: HTML generation parser failure, cyclic reference diagram rendering overflow.
//! - **Trace Scope**: `server-rs::bin::graph_query::visualizer`

use crate::graph::{CodeSymbolGraph, SymbolNode};
use petgraph::visit::EdgeRef;

pub fn generate_mermaid_diagram(nodes: &[SymbolNode], graph: &CodeSymbolGraph) -> String {
    let mut diagram = String::new();
    diagram.push_str("graph TD\n");

    let mut id_map = std::collections::HashMap::new();
    for (i, node) in nodes.iter().enumerate() {
        let clean_path = node.path.replace('\\', "/");
        let label = format!("\"{}<br/>({})\"", node.name, clean_path);
        let node_id = format!("N{}", i);
        diagram.push_str(&format!("    {}[{}]\n", node_id, label));
        id_map.insert((node.name.clone(), node.path.clone()), node_id);
    }

    let mut added_edges = std::collections::HashSet::new();
    for node_a in nodes {
        let key_a = format!("{}\x01{}", node_a.path, node_a.name);
        if let Some(&idx_a) = graph.index.get(&key_a) {
            for edge in graph.graph.edges(idx_a) {
                let idx_b = edge.target();
                let node_b = &graph.graph[idx_b];
                if let Some(id_a) = id_map.get(&(node_a.name.clone(), node_a.path.clone())) {
                    if let Some(id_b) = id_map.get(&(node_b.name.clone(), node_b.path.clone())) {
                        if added_edges.insert((id_a.clone(), id_b.clone())) {
                            diagram.push_str(&format!("    {} --> {}\n", id_a, id_b));
                        }
                    }
                }
            }
        }
    }

    diagram
}

pub fn generate_html_visualizer(nodes: &[SymbolNode], graph: &CodeSymbolGraph) -> String {
    let mut elements_json = Vec::new();

    let mut id_map = std::collections::HashMap::new();
    for (i, node) in nodes.iter().enumerate() {
        let clean_path = node.path.replace('\\', "/");
        let node_id = format!("N{}", i);
        id_map.insert((node.name.clone(), node.path.clone()), node_id.clone());

        let label = format!("{} ({})", node.name, clean_path);
        elements_json.push(serde_json::json!({
            "data": {
                "id": node_id,
                "label": label,
                "name": node.name,
                "path": clean_path,
                "kind": node.kind,
            }
        }));
    }

    let mut added_edges = std::collections::HashSet::new();
    for node_a in nodes {
        let key_a = format!("{}\x01{}", node_a.path, node_a.name);
        if let Some(&idx_a) = graph.index.get(&key_a) {
            for edge in graph.graph.edges(idx_a) {
                let idx_b = edge.target();
                let node_b = &graph.graph[idx_b];
                if let Some(id_a) = id_map.get(&(node_a.name.clone(), node_a.path.clone())) {
                    if let Some(id_b) = id_map.get(&(node_b.name.clone(), node_b.path.clone())) {
                        if added_edges.insert((id_a.clone(), id_b.clone())) {
                            let edge_id = format!("{}_{}", id_a, id_b);
                            elements_json.push(serde_json::json!({
                                "data": {
                                    "id": edge_id,
                                    "source": id_a,
                                    "target": id_b,
                                }
                            }));
                        }
                    }
                }
            }
        }
    }

    let elements_str = serde_json::to_string_pretty(&elements_json).unwrap_or("[]".to_string());

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8" />
    <title>Tadpole OS Codebase Blast Radius Visualizer</title>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/cytoscape/3.28.1/cytoscape.min.js"></script>
    <style>
        body {{
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            margin: 0;
            padding: 0;
            background-color: #0f172a;
            color: #f8fafc;
            display: flex;
            flex-direction: column;
            height: 100vh;
        }}
        header {{
            background-color: #1e293b;
            padding: 1rem 2rem;
            box-shadow: 0 4px 6px -1px rgb(0 0 0 / 0.1);
            z-index: 10;
        }}
        h1 {{
            margin: 0;
            font-size: 1.5rem;
            color: #38bdf8;
        }}
        #cy {{
            flex-grow: 1;
            width: 100%;
            height: 100%;
        }}
    </style>
</head>
<body>
    <header>
        <h1>Code Symbol Blast Radius Map</h1>
    </header>
    <div id="cy"></div>
    <script>
        var cy = cytoscape({{
            container: document.getElementById('cy'),
            elements: {elements_str},
            style: [
                {{
                    selector: 'node',
                    style: {{
                        'background-color': '#38bdf8',
                        'label': 'data(label)',
                        'color': '#f8fafc',
                        'font-size': '12px',
                        'text-valign': 'center',
                        'text-halign': 'right',
                        'text-margin-x': 8,
                        'width': 24,
                        'height': 24,
                        'overlay-padding': '6px',
                        'z-index': '10'
                    }}
                }},
                {{
                    selector: 'edge',
                    style: {{
                        'width': 2,
                        'line-color': '#475569',
                        'target-arrow-color': '#475569',
                        'target-arrow-shape': 'triangle',
                        'curve-style': 'bezier'
                    }}
                }}
            ],
            layout: {{
                name: 'cose',
                animate: true,
                nodeRepulsion: function( node ){{ return 2048; }},
                idealEdgeLength: function( edge ){{ return 64; }}
            }}
        }});
    </script>
</body>
</html>
"#
    )
}
