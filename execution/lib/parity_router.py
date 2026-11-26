"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / parity_router
- **Primary Entrypoints**: `normalize_path`, `scan_router`, `scan_openapi`

### ⚠️ Non-Negotiables
- `[Structural]` Deterministic route extraction via syn AST or fallback regex.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
"""

import os
import re
import json
import yaml
import subprocess
from pathlib import Path
from typing import Dict, List, Any, Optional

def normalize_path(path: str) -> str:
    """Normalizes paths from any format (:id, {id}, :agent_id) to a standard {PARAM} placeholder."""
    p = re.sub(r':([a-zA-Z0-9_]+)', r'{PARAM}', path)
    return re.sub(r'\{([a-zA-Z0-9_]+)\}', r'{PARAM}', p)

def find_graph_query_bin(repo_root: Path) -> Optional[Path]:
    """Locates graph_query binary in debug or release target directories."""
    bin_ext = ".exe" if os.name == "nt" else ""
    for profile in ("debug", "release"):
        candidate = repo_root / "server-rs" / "target" / profile / f"graph_query{bin_ext}"
        if candidate.is_file():
            return candidate
    return None

def try_rust_route_parity(router_path: Path) -> Optional[List[Dict[str, Any]]]:
    """Attempts to extract Axum routes via the compiled graph_query binary."""
    r_path = router_path.resolve()
    if r_path.name != "router.rs" or r_path.parent.name != "src" or r_path.parent.parent.name != "server-rs":
        return None
    repo_root = r_path.parent.parent.parent
    candidate = find_graph_query_bin(repo_root)
    if candidate is None:
        return None
    cmd = [str(candidate), "route-parity", "--root", str(repo_root), "--json"]
    try:
        res = subprocess.run(cmd, capture_output=True, text=True, timeout=5)
    except (OSError, subprocess.SubprocessError):
        return None
    if res.returncode != 0 or not res.stdout.strip():
        return None
    try:
        payload = json.loads(res.stdout)
        ast_routes = payload.get("routes")
        return ast_routes if isinstance(ast_routes, list) and len(ast_routes) > 0 else None
    except json.JSONDecodeError:
        return None

def find_matching_brace_end(content: str, open_brace_idx: int) -> int:
    """Finds index just past the matching closing brace for the brace at open_brace_idx."""
    brace_count = 0
    for i in range(open_brace_idx, len(content)):
        char = content[i]
        if char == '{':
            brace_count += 1
        elif char == '}':
            brace_count -= 1
            if brace_count == 0:
                return i + 1
    return open_brace_idx

def parse_routes_from_body(body: str) -> List[Dict[str, str]]:
    """Extracts route paths, HTTP methods, and handler names from Axum router body."""
    route_matches = re.finditer(
        r'\.route\s*\(\s*"([^"]+)"\s*,\s*(?:[a-z:]+::)?([a-z]+)\(([^)]+)\)',
        body,
        re.DOTALL
    )
    return [
        {"path": rm.group(1), "method": rm.group(2).upper(), "handler": rm.group(3).strip()}
        for rm in route_matches
    ]

def extract_function_routes(content: str) -> Dict[str, List[Dict[str, str]]]:
    """Extracts sub-router routes defined inside build_* functions."""
    function_routes: Dict[str, List[Dict[str, str]]] = {}
    matches = re.finditer(r'fn\s+(build_[a-z0-9_]+)\s*\([^)]*\)\s*->\s*Router[^{]*\{', content)
    for m in matches:
        func_name = m.group(1)
        start_idx = m.end() - 1
        end_idx = find_matching_brace_end(content, start_idx)
        if end_idx > start_idx:
            body = content[start_idx + 1 : end_idx - 1]
            function_routes[func_name] = parse_routes_from_body(body)
    return function_routes

def resolve_nested_routes(content: str, function_routes: Dict[str, List[Dict[str, str]]]) -> List[Dict[str, str]]:
    """Resolves routes registered via .nest() calls."""
    routes: List[Dict[str, str]] = []
    v1_nest_matches = re.finditer(r'\.nest\s*\(\s*"([^"]+)"\s*,\s*(build_[a-z0-9_]+)\s*\(\s*\)\s*\)', content)
    for nm in v1_nest_matches:
        prefix, func_name = nm.group(1), nm.group(2)
        if func_name not in function_routes:
            continue
        for r in function_routes[func_name]:
            full_path = f"/v1{prefix}{r['path']}".replace("//", "/")
            if full_path.endswith("/") and len(full_path) > 1:
                full_path = full_path[:-1]
            routes.append({"path": full_path, "method": r["method"], "handler": r["handler"]})
    return routes

def resolve_root_v1_routes(content: str) -> List[Dict[str, str]]:
    """Resolves routes directly bound in build_protected_v1_routes."""
    routes: List[Dict[str, str]] = []
    m_root_v1 = re.search(r'fn build_protected_v1_routes\s*\([^)]*\)\s*->\s*Router[^{]*\{', content)
    if not m_root_v1:
        return routes
    start_idx = m_root_v1.end() - 1
    end_idx = find_matching_brace_end(content, start_idx)
    if end_idx > start_idx:
        body = content[start_idx + 1 : end_idx - 1]
        for r in parse_routes_from_body(body):
            routes.append({"path": f"/v1{r['path']}", "method": r["method"], "handler": r["handler"]})
    return routes

def resolve_direct_v1_routes(function_routes: Dict[str, List[Dict[str, str]]]) -> List[Dict[str, str]]:
    """Resolves routes from builders that are directly merged under /v1."""
    direct_v1_builders = [
        "build_engine_public_routes",
        "build_engine_protected_routes",
        "build_remote_public_routes",
        "build_remote_paired_routes",
    ]
    routes: List[Dict[str, str]] = []
    for func in direct_v1_builders:
        if func not in function_routes:
            continue
        for r in function_routes[func]:
            routes.append({"path": f"/v1{r['path']}", "method": r["method"], "handler": r["handler"]})
    return routes

def scan_router_regex_fallback(router_path: Path) -> List[Dict[str, str]]:
    """Scans Axum router file using regex patterns when AST binary is absent."""
    with open(router_path, 'r', encoding='utf-8') as f:
        content = f.read()

    function_routes = extract_function_routes(content)
    routes: List[Dict[str, str]] = []
    routes.extend(resolve_nested_routes(content, function_routes))
    routes.extend(resolve_root_v1_routes(content))
    routes.extend(resolve_direct_v1_routes(function_routes))
    return routes

def scan_router(router_path: Any) -> List[Dict[str, Any]]:
    """Extracts routes from the Axum router.rs file via Rust syn AST with regex fallback."""
    p = Path(router_path)
    if not p.exists():
        return []

    ast_routes = try_rust_route_parity(p)
    if ast_routes is not None:
        return ast_routes

    return scan_router_regex_fallback(p)

def scan_openapi(openapi_path: Any) -> Dict[str, Any]:
    """Parses paths from openapi.yaml."""
    p = Path(openapi_path)
    if not p.exists():
        return {}
    with open(p, 'r', encoding='utf-8') as f:
        data = yaml.safe_load(f)
        return data.get("paths", {})

