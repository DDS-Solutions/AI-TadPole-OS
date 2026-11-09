"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / parity_guard
- **Primary Entrypoints**: `print_result`, `check_env_vars`, `verify_routes_parity`, `check_parity`, `main`

### ⚠️ Non-Negotiables
- `[Structural]` Deterministic architectural parity gate for Axum routes, OpenAPI specs, and docs.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
"""

import os
import re
import sys
import io
from pathlib import Path
from typing import Dict, Set, Any, Optional

# Ensure execution directory is in sys.path for local module resolution
_EXEC_DIR = Path(__file__).resolve().parent
if str(_EXEC_DIR) not in sys.path:
    sys.path.insert(0, str(_EXEC_DIR))

from lib.parity_router import normalize_path, scan_router, scan_openapi
from lib.parity_validators import (
    check_skills as _check_skills,
    check_api_docs_parity as _check_api_docs_parity,
    run_pollywog_audit as _run_pollywog_audit,
    check_critical_docs_content as _check_critical_docs_content,
    check_wiki_metadata as _check_wiki_metadata,
    check_backtest_engine as _check_backtest_engine,
    check_security_registry as _check_security_registry,
    check_doc_links as _check_doc_links,
    check_dual_plane_identity_parity as _check_dual_plane_identity_parity,
    check_rust_test_compilation as _check_rust_test_compilation,
)

if sys.platform == "win32":
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
    sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8')

def print_result(check: str, status: bool, message: str) -> None:
    """Prints formatted test output line with OK/FAIL status badge."""
    icon = "[OK]" if status else "[FAIL]"
    print(f"{icon} [{check}] {message}")

def extract_env_vars_from_file(file_path: Path) -> Set[str]:
    """Extracts std::env::var occurrences from a single Rust source file."""
    try:
        content = file_path.read_text(encoding="utf-8", errors="ignore")
        return set(re.findall(r'std::env::var\(\s*"([A-Z0-9_]+)"\s*\)', content))
    except OSError:
        return set()

def scan_server_env_vars(src_dir: Path) -> Set[str]:
    """Scans all Rust files in server-rs/src for std::env::var statements."""
    env_vars: Set[str] = set()
    for root_dir_walk, _, files in os.walk(src_dir):
        for file in files:
            if file.endswith(".rs"):
                env_vars.update(extract_env_vars_from_file(Path(root_dir_walk) / file))
    return env_vars

def parse_env_example_keys(env_example_path: Path) -> Set[str]:
    """Parses keys defined in .env.example."""
    keys: Set[str] = set()
    if not env_example_path.exists():
        return keys
    with open(env_example_path, 'r', encoding='utf-8') as f:
        for line in f:
            stripped = line.strip()
            if stripped and not stripped.startswith('#') and '=' in stripped:
                keys.add(stripped.split('=')[0].strip())
    return keys

def check_env_vars(root: Path, env_vars_in_code: Optional[Set[str]] = None) -> int:
    """Verifies all environment variables used in code are documented in .env.example."""
    print("\nScanning for Environment Variables...")
    if env_vars_in_code is None:
        env_vars_in_code = scan_server_env_vars(root / "server-rs" / "src")

    documented_vars = parse_env_example_keys(root / ".env.example")
    errors = 0
    for var in sorted(env_vars_in_code):
        if var not in documented_vars:
            print_result("ENV-VAR", False, f'std::env::var("{var}") used in code but missing from .env.example')
            errors += 1
        else:
            print_result("ENV-VAR", True, f"{var} documented")
    return errors

def verify_single_route(route: Dict[str, str], doc_paths: Dict[str, Any]) -> int:
    """Verifies a single Axum route against OpenAPI path definitions."""
    path = route["path"]
    method = route["method"].lower()
    base_path = normalize_path(path)
    is_legacy = not path.startswith("/v1")

    if base_path in doc_paths:
        if method in doc_paths[base_path]:
            print_result("CODE->OPENAPI", True, f"{route['method']} {path}")
            return 0
        print_result("CODE->OPENAPI", False, f"Method {method} missing for {path} in openapi.yaml")
        return 1

    if is_legacy:
        print_result("CODE->OPENAPI", True, f"{route['method']} {path} (Legacy/Internal)")
        return 0

    print_result("CODE->OPENAPI", False, f"Route {path} missing in openapi.yaml")
    return 1

def verify_routes_parity(code_routes: list, openapi_path: Path) -> int:
    """Verifies that all Axum code routes are present in OpenAPI documentation."""
    raw_doc_paths = scan_openapi(openapi_path)
    doc_paths = {normalize_path(k): v for k, v in raw_doc_paths.items()}
    errors = 0
    for route in code_routes:
        errors += verify_single_route(route, doc_paths)
    return errors

def extract_doc_tags_from_file(file_path: Path) -> list:
    """Extracts @docs tag declarations from a single file."""
    try:
        content = file_path.read_text(encoding='utf-8')
        tags = re.findall(r'//[/!]\s*@docs\s+([A-Z_]+):([a-zA-Z0-9_]+)', content)
        mtime = os.path.getmtime(file_path)
        return [(doc, sec, file_path.name, mtime) for doc, sec in tags]
    except OSError:
        return []

def scan_server_doc_tags(src_dir: Path) -> Dict[str, Dict[str, Any]]:
    """Collects @docs tag references across server-rs source files."""
    doc_tags: Dict[str, Dict[str, Any]] = {}
    for root_dir_walk, _, files in os.walk(src_dir):
        for file in files:
            if not file.endswith(".rs"):
                continue
            for doc, sec, fname, mtime in extract_doc_tags_from_file(Path(root_dir_walk) / file):
                doc_tags[f"{doc}:{sec}"] = {
                    "file": fname,
                    "doc_name": doc,
                    "section": sec,
                    "mtime": mtime,
                }
    return doc_tags

def check_single_doc_tag(tag: str, info: Dict[str, Any], root: Path, cache: Dict[str, str]) -> int:
    """Verifies synchronization of a single @docs tag against target document."""
    doc_name = info["doc_name"]
    section = info["section"]
    doc_file = root / "docs" / f"{doc_name}.md"
    if not doc_file.exists():
        doc_file = root / f"{doc_name}.md"
        if not doc_file.exists():
            print_result("ADG-TAG", False, f"Tag {tag} references missing file {doc_name}.md")
            return 1

    doc_str = str(doc_file)
    if doc_str not in cache:
        try:
            cache[doc_str] = doc_file.read_text(encoding='utf-8')
        except OSError:
            cache[doc_str] = ""

    doc_content = cache[doc_str]
    is_synced = f"{doc_name}:{section}" in doc_content or f"@docs {doc_name}:{section}" in doc_content or section in doc_content
    if is_synced:
        print_result("ADG-TAG", True, f"{tag} synchronized")
        return 0

    doc_mtime = os.path.getmtime(doc_file)
    if info["mtime"] > doc_mtime and (info["mtime"] - doc_mtime > 300):
        print_result("ADG-DRIFT", False, f"{info['file']} updated but {doc_name}.md is older (Drift detected)")
        return 1

    print_result("ADG-TAG", True, f"{tag} synchronized (Grace period)")
    return 0

def verify_doc_tags_parity(root: Path, doc_tags: Dict[str, Dict[str, Any]]) -> int:
    """Verifies that all collected @docs tags are synchronized."""
    cache: Dict[str, str] = {}
    errors = 0
    for tag, info in sorted(doc_tags.items()):
        errors += check_single_doc_tag(tag, info, root, cache)
    return errors

def check_parity(root_dir: Any, fix: bool = False) -> bool:
    """Executes the full Tadpole OS Architectural Parity Audit."""
    root = Path(root_dir)
    print("--- Tadpole OS Parity Audit ---\n")

    code_routes = scan_router(root / "server-rs" / "src" / "router.rs")
    print(f"Found {len(code_routes)} routes in router.rs")

    errors = verify_routes_parity(code_routes, root / "docs" / "openapi.yaml")

    print("\nScanning Server Code for Environment Variables and Documentation Tags (@docs)...")
    env_vars_in_code = scan_server_env_vars(root / "server-rs" / "src")
    doc_tags = scan_server_doc_tags(root / "server-rs" / "src")

    errors += verify_doc_tags_parity(root, doc_tags)
    errors += check_env_vars(root, env_vars_in_code=env_vars_in_code)
    errors += _check_skills(root, print_result)
    errors += _check_api_docs_parity(root, print_result, fix=fix)
    errors += _run_pollywog_audit(root, print_result)
    errors += _check_critical_docs_content(root, print_result)
    errors += _check_wiki_metadata(root, print_result)

    print(f"\nAudit Complete. Errors found: {errors}")
    return errors == 0

# Backward compatibility wrappers
def check_skills(root: Path) -> int:
    return _check_skills(root, print_result)

def check_api_docs_parity(root: Path, fix: bool = False) -> int:
    return _check_api_docs_parity(root, print_result, fix=fix)

def run_pollywog_audit(root: Path) -> int:
    return _run_pollywog_audit(root, print_result)

def check_critical_docs_content(root: Path) -> int:
    return _check_critical_docs_content(root, print_result)

def check_wiki_metadata(root: Path) -> int:
    return _check_wiki_metadata(root, print_result)

def check_backtest_engine(root: Path) -> int:
    return _check_backtest_engine(root, print_result)

def check_security_registry(root: Path) -> int:
    return _check_security_registry(root, print_result)

def check_doc_links(root: Path) -> int:
    return _check_doc_links(root, print_result)

def check_dual_plane_identity_parity(root: Path) -> int:
    return _check_dual_plane_identity_parity(root, print_result)

def check_rust_test_compilation(root: Path) -> int:
    return _check_rust_test_compilation(root, print_result)

def main() -> int:
    project_root = sys.argv[1] if len(sys.argv) > 1 and not sys.argv[1].startswith("FIX=") else "."
    fix_mode = any(arg == "FIX=1" for arg in sys.argv)
    root_path = Path(project_root)

    backtest_errs = check_backtest_engine(root_path)
    sec_errs = check_security_registry(root_path)
    link_errs = check_doc_links(root_path)
    dual_plane_errs = check_dual_plane_identity_parity(root_path)
    rust_test_errs = check_rust_test_compilation(root_path)

    parity_ok = check_parity(project_root, fix=fix_mode)
    all_ok = parity_ok and backtest_errs == 0 and sec_errs == 0 and link_errs == 0 and dual_plane_errs == 0 and rust_test_errs == 0
    return 0 if all_ok else 1

if __name__ == "__main__":
    sys.exit(main())
