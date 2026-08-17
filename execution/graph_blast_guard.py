#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Assist Note
**Graph Blast Guard**
Calculates the exact symbol blast radius and topology slice for target files/symbols.

### 🔍 Debugging & Observability
- **Failure Path**: Script error or execution logic mismatch.
- **Telemetry Link**: Search `[graph]` in system logs.
"""
# [graph] Graph Blast Guard implementation
"""
Graph Blast Guard (Pillar 1 & 4)
---------------------------------
Calculates the exact symbol blast radius and topology slice for target files/symbols.
Used before refactoring to prevent breaking changes and as a lightweight (~1k-3k token)
context injector for worker subagents.
"""

import sys
import json
import argparse
import subprocess
from pathlib import Path

WORKSPACE_ROOT = Path(__file__).resolve().parent.parent

def get_first_symbol_in_file(path: str) -> str:
    """Helper to query graph_query file and return first symbol name."""
    cmd = [
        "cargo", "run", "--manifest-path", "server-rs/Cargo.toml",
        "--bin", "graph_query", "--", "file",
        "--path", path,
        "--json"
    ]
    try:
        res = subprocess.run(cmd, cwd=WORKSPACE_ROOT, capture_output=True, text=True, encoding="utf-8", errors="replace", check=True)
        output = res.stdout.strip()
        json_start = output.find('{')
        if json_start != -1:
            data = json.loads(output[json_start:])
            symbols = data.get("symbols", [])
            if symbols:
                return symbols[0].get("name", "")
    except Exception:
        pass
    return ""

def run_blast_query(path: str, name: str = None, depth: int = 50):
    """Executes graph_query blast on server-rs binary."""
    target_name = name or get_first_symbol_in_file(path) or "main"
    cmd = [
        "cargo", "run", "--manifest-path", "server-rs/Cargo.toml",
        "--bin", "graph_query", "--", "blast",
        "--name", target_name,
        "--path", path,
        "--json"
    ]
    cmd.extend(["--depth", str(depth)])

    try:
        res = subprocess.run(
            cmd, cwd=WORKSPACE_ROOT, capture_output=True, text=True, encoding="utf-8", errors="replace", check=True
        )
        output = res.stdout.strip()
        json_start = output.find('[')
        if json_start == -1:
            json_start = output.find('{')
        if json_start != -1:
            output = output[json_start:]
        return json.loads(output), target_name
    except Exception as e:
        return {"error": f"Failed to execute graph_query blast: {e}"}, target_name

def extract_scoped_topology_slice(path: str, name: str = None) -> dict:
    """Extracts a lightweight topology slice (< 3k tokens) with pre-resolved 1-hop neighbor export signatures."""
    data, resolved_name = run_blast_query(path, name)
    if isinstance(data, dict) and "error" in data:
        return data

    symbols_list = data if isinstance(data, list) else []
    affected_files = list(set([item.get("path") for item in symbols_list if item.get("path")]))

    # Pre-resolved 1-hop neighbor signatures (neighborMap)
    neighbor_signatures = []
    for item in symbols_list[:10]:
        sym_name = item.get("name") or item.get("symbol_name")
        sym_file = item.get("path") or item.get("file_path")
        sym_kind = item.get("kind", "symbol")
        if sym_name and sym_file and sym_file != path:
            neighbor_signatures.append({
                "symbol": sym_name,
                "file": sym_file,
                "kind": sym_kind
            })

    slice_info = {
        "target_file": path,
        "target_symbol": resolved_name,
        "blast_radius_symbol_count": len(symbols_list),
        "blast_radius_file_count": len(affected_files),
        "symbols": symbols_list[:15],
        "affected_files": affected_files[:20],
        "neighbor_map": neighbor_signatures[:10],
        "token_budget_cap_tokens": 3000
    }
    return slice_info

def main():
    parser = argparse.ArgumentParser(description="Graph Blast Guard & Topology Slicer")
    parser.add_argument("--path", required=True, help="Target file path")
    parser.add_argument("--name", help="Optional target symbol name")
    parser.add_argument("--depth", type=int, default=50, help="Blast radius depth")
    parser.add_argument("--json", action="store_true", help="Output JSON slice")
    args = parser.parse_args()

    slice_data = extract_scoped_topology_slice(args.path, args.name)

    if args.json:
        print(json.dumps(slice_data, indent=2))
    else:
        print("==================================================")
        print("       GRAPH INTELLIGENCE - BLAST RADIUS GUARD    ")
        print("==================================================")
        print(f"Target File: {slice_data.get('target_file')}")
        print(f"Target Symbol: {slice_data.get('target_symbol')}")
        print(f"Blast Radius Symbol Count: {slice_data.get('blast_radius_symbol_count')}")
        print(f"Blast Radius File Count: {slice_data.get('blast_radius_file_count')}")
        print("--------------------------------------------------")

if __name__ == "__main__":
    main()
