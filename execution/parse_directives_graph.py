"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / parse_directives_graph
- **Primary Entrypoints**: `parse_directives_and_skills`, `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[directives_graph]`
- **Witness Tests**: none declared
"""

import os
import re
import json
import sys
import io
from pathlib import Path

# Ensure stdout handles UTF-8 on Windows
if sys.platform == "win32":
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
    sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8')

WORKSPACE_ROOT = Path(__file__).resolve().parent.parent

def parse_directives_and_skills():
    """Parses directives, skills, workflows, and error registries into a structured graph."""
    nodes = []
    edges = []
    node_ids = set()

    def add_node(node_id, node_type, name, summary, tags=None, metadata=None):
        if node_id not in node_ids:
            node_ids.add(node_id)
            nodes.append({
                "id": node_id,
                "type": node_type,
                "name": name,
                "summary": summary or "",
                "tags": tags or [],
                "metadata": metadata or {}
            })

    def add_edge(source, target, edge_type, weight=1.0):
        edges.append({
            "source": source,
            "target": target,
            "type": edge_type,
            "weight": weight
        })

    # 1. Parse Directives
    directives_dir = WORKSPACE_ROOT / "directives"
    if directives_dir.exists():
        for file_path in directives_dir.glob("*.md"):
            rel_path = str(file_path.relative_to(WORKSPACE_ROOT)).replace("\\", "/")
            doc_id = f"directive:{file_path.stem}"
            content = file_path.read_text(encoding="utf-8", errors="ignore")
            
            # Extract title / summary
            first_line = content.splitlines()[0] if content.splitlines() else file_path.name
            summary = first_line.lstrip("#").strip()
            
            add_node(doc_id, "directive", file_path.stem, summary, tags=["sop", "directive"], metadata={"path": rel_path})

            # Find links to other directives or scripts
            script_matches = re.findall(r'`?(execution/[a-zA-Z0-9_]+\.py)`?', content)
            for script in set(script_matches):
                script_id = f"script:{Path(script).stem}"
                add_node(script_id, "script", Path(script).stem, f"Execution script {script}", tags=["execution"], metadata={"path": script})
                add_edge(doc_id, script_id, "invokes")

    # 2. Parse Workspace Skills
    skills_dir = WORKSPACE_ROOT / ".agent" / "skills"
    if skills_dir.exists():
        for skill_md in skills_dir.glob("**/SKILL.md"):
            rel_path = str(skill_md.relative_to(WORKSPACE_ROOT)).replace("\\", "/")
            skill_name = skill_md.parent.name
            skill_id = f"skill:{skill_name}"
            content = skill_md.read_text(encoding="utf-8", errors="ignore")

            # Extract description from frontmatter
            desc_match = re.search(r'description:\s*(.+)', content)
            summary = desc_match.group(1).strip() if desc_match else f"Skill {skill_name}"

            add_node(skill_id, "skill", skill_name, summary, tags=["skill", "agent-capability"], metadata={"path": rel_path})

            # Check execution script references
            exec_matches = re.findall(r'`?(scripts/[a-zA-Z0-9_]+\.py|execution/[a-zA-Z0-9_]+\.py)`?', content)
            for ex in set(exec_matches):
                script_id = f"script:{Path(ex).stem}"
                add_node(script_id, "script", Path(ex).stem, f"Skill script {ex}", tags=["execution"], metadata={"path": ex})
                add_edge(skill_id, script_id, "requires_script")

    # 3. Parse Workflows
    workflows_dir = WORKSPACE_ROOT / ".agent" / "workflows"
    if workflows_dir.exists():
        for wf_md in workflows_dir.glob("*.md"):
            rel_path = str(wf_md.relative_to(WORKSPACE_ROOT)).replace("\\", "/")
            wf_name = wf_md.stem
            wf_id = f"workflow:{wf_name}"
            content = wf_md.read_text(encoding="utf-8", errors="ignore")

            first_line = content.splitlines()[0] if content.splitlines() else wf_name
            summary = first_line.lstrip("#").strip()

            add_node(wf_id, "workflow", wf_name, summary, tags=["workflow", "slash-command"], metadata={"path": rel_path})

    # 4. Parse Error Registry if present
    err_reg_path = WORKSPACE_ROOT / "docs" / "ERROR_REGISTRY.json"
    if err_reg_path.exists():
        try:
            err_data = json.loads(err_reg_path.read_text(encoding="utf-8"))
            for code, details in err_data.items():
                err_id = f"error:{code}"
                msg = details.get("message") or details.get("description") or f"Error {code}"
                add_node(err_id, "error_code", code, msg, tags=["error-registry"], metadata=details)
        except Exception:
            pass

    # 5. Parse Telemetry Map if present
    tel_map_path = WORKSPACE_ROOT / "docs" / "TELEMETRY_MAP.json"
    if tel_map_path.exists():
        try:
            tel_data = json.loads(tel_map_path.read_text(encoding="utf-8"))
            for tag, details in tel_data.items():
                tag_id = f"telemetry:{tag}"
                loc = details.get("location") or details.get("file") or ""
                add_node(tag_id, "telemetry_tag", tag, f"Telemetry tag {tag}", tags=["telemetry"], metadata=details)
        except Exception:
            pass

    # Filter out dangling edges
    valid_edges = [e for e in edges if e["source"] in node_ids and e["target"] in node_ids]

    graph = {
        "version": "1.0.0",
        "generated_by": "parse_directives_graph.py",
        "nodes_count": len(nodes),
        "edges_count": len(valid_edges),
        "nodes": nodes,
        "edges": valid_edges
    }

    # Ensure .tmp directory exists
    tmp_dir = WORKSPACE_ROOT / ".tmp"
    tmp_dir.mkdir(parents=True, exist_ok=True)
    out_path = tmp_dir / "directives_graph.json"
    out_path.write_text(json.dumps(graph, indent=2), encoding="utf-8")

    return graph, out_path

def main():
    graph, out_path = parse_directives_and_skills()
    print("==================================================")
    print("     SOVEREIGN DIRECTIVES KNOWLEDGE GRAPH PARSER  ")
    print("==================================================")
    print(f"📊 [directives_graph] Generated Output: {out_path.relative_to(WORKSPACE_ROOT)}")
    print(f"📊 [directives_graph] Parsed Nodes: {graph['nodes_count']}")
    print(f"📊 [directives_graph] Parsed Edges: {graph['edges_count']}")
    print("--------------------------------------------------")

if __name__ == "__main__":
    main()
