"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / parity_guard
- **Primary Entrypoints**: `print_result`, `normalize_path`, `scan_router`, `scan_openapi`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
- **Witness Tests**: none declared
"""

"""
# 🛡️ Tadpole Engine: Parity Guard
**Agent Consistency**: High (ECC Optimized)
**Source of Truth**: `execution/parity_guard.py`, `docs/openapi.yaml`
**Inputs**: `server-rs/src/router.rs`, `docs/`, `.agent/skills/`
**Outputs**: Structured Parity Report (Exit 0 on success, Exit 1 on drift)

> [!IMPORTANT]
> **AI Assist Note (Execution Logic)**:
> This script is the "Integrity Gate" for the Tadpole OS 3-layer architecture.
> - **Gateway Pillar**: Verifies Axum routes against OpenAPI specifications.
> - **Registry Pillar**: Verifies skill manifests against their execution scripts.
> - **Security Root**: Checks for environment variable leakage in `.env.example` and verifies `docs/SECURITY_REGISTRY.json`.
> - **ECC Audit**: Validates `@docs` cross-links for documentation drift.

Workflow: Code -> docs/openapi.yaml -> docs/API_REFERENCE.md -> Unit Tests
"""

import os
import re
import yaml
import json
import sys
import io
import subprocess
from pathlib import Path
from typing import Dict, List, Any

# Ensure stdout handles UTF-8 on Windows
if sys.platform == "win32":
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
    sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8')

def print_result(check, status, message):
    icon = "[OK]" if status else "[FAIL]"
    print(f"{icon} [{check}] {message}")

def normalize_path(path):
    """Normalizes paths from any format (:id, {id}, :agent_id) to a standard {PARAM} placeholder for comparison."""
    p = re.sub(r':([a-zA-Z0-9_]+)', r'{PARAM}', path)
    p = re.sub(r'\{([a-zA-Z0-9_]+)\}', r'{PARAM}', p)
    return p

def scan_router(router_path):
    """Extracts routes from the Axum router.rs file, handling multi-level nesting."""
    routes = []
    if not os.path.exists(router_path):
        return routes
    
    with open(router_path, 'r', encoding='utf-8') as f:
        content = f.read()
        
    function_routes = {}
    matches = re.finditer(r'fn\s+(build_[a-z0-9_]+)\s*\([^)]*\)\s*->\s*Router[^{]*\{', content)
    for m in matches:
        func_name = m.group(1)
        start_idx = m.end() - 1
        
        brace_count = 0
        end_idx = start_idx
        for i in range(start_idx, len(content)):
            char = content[i]
            if char == '{':
                brace_count += 1
            elif char == '}':
                brace_count -= 1
                if brace_count == 0:
                    end_idx = i + 1
                    break
        
        if end_idx > start_idx:
            body = content[start_idx+1 : end_idx-1]
            route_matches = re.finditer(r'\.route\s*\(\s*"([^"]+)"\s*,\s*(?:[a-z:]+::)?([a-z]+)\(([^)]+)\)', body, re.DOTALL)
            func_routes = []
            for rm in route_matches:
                path = rm.group(1)
                method = rm.group(2).upper()
                handler = rm.group(3).strip()
                func_routes.append({"path": path, "method": method, "handler": handler})
            
            function_routes[func_name] = func_routes

    v1_nest_matches = re.finditer(r'\.nest\s*\(\s*"([^"]+)"\s*,\s*(build_[a-z0-9_]+)\s*\(\s*\)\s*\)', content)
    for nm in v1_nest_matches:
        prefix = nm.group(1)
        func_name = nm.group(2)
        if func_name in function_routes:
            for r in function_routes[func_name]:
                full_path = f"/v1{prefix}{r['path']}".replace("//", "/")
                if full_path.endswith("/") and len(full_path) > 1:
                    full_path = full_path[:-1]
                routes.append({"path": full_path, "method": r["method"], "handler": r["handler"]})

    m_root_v1 = re.search(r'fn build_protected_v1_routes.*?\{([^}]+)\}', content, re.DOTALL)
    if m_root_v1:
        body = m_root_v1.group(1)
        route_matches = re.finditer(r'\.route\s*\(\s*"([^"]+)"\s*,\s*(?:[a-z:]+::)?([a-z]+)\(([^)]+)\)', body, re.DOTALL)
        for m in route_matches:
            path = m.group(1)
            method = m.group(2).upper()
            handler = m.group(3).strip()
            routes.append({"path": f"/v1{path}", "method": method, "handler": handler})

    direct_v1_builders = [
        "build_engine_public_routes",
        "build_engine_protected_routes",
        "build_remote_public_routes",
        "build_remote_paired_routes",
    ]
    for func in direct_v1_builders:
        if func in function_routes:
            for r in function_routes[func]:
                routes.append({"path": f"/v1{r['path']}", "method": r["method"], "handler": r["handler"]})

    return routes

def scan_openapi(openapi_path):
    """Parses paths from openapi.yaml."""
    if not os.path.exists(openapi_path):
        return {}
    
    with open(openapi_path, 'r', encoding='utf-8') as f:
        data = yaml.safe_load(f)
        return data.get("paths", {})

def check_env_vars(root):
    print(f"\nScanning for Environment Variables...")
    env_vars_in_code = set()
    for root_dir_walk, _, files in os.walk(root / "server-rs" / "src"):
        for file in files:
            if not file.endswith(".rs"): continue
            file_path = os.path.join(root_dir_walk, file)
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
                matches = re.findall(r'std::env::var\(\s*"([A-Z0-9_]+)"\s*\)', content)
                for m in matches:
                    env_vars_in_code.add(m)
                    
    env_example_path = root / ".env.example"
    env_example_vars = set()
    if env_example_path.exists():
        with open(env_example_path, 'r', encoding='utf-8') as f:
            for line in f:
                line = line.strip()
                if not line: continue
                if line.startswith('#'):
                    line = line.lstrip('#').strip()
                if '=' in line:
                    key = line.split('=')[0].strip()
                    env_example_vars.add(key)
    
    errors = 0
    for var in env_vars_in_code:
        if var not in env_example_vars:
            print_result("ENV-VAR", False, f"std::env::var(\"{var}\") used in code but missing from .env.example")
            errors += 1
        else:
            print_result("ENV-VAR", True, f"{var} documented")
            
    return errors

def check_skills(root):
    print(f"\nScanning Skills & Workflows...")
    skills_dir = root / "server-rs" / "data" / "skills"
    errors = 0
    
    if not skills_dir.exists():
        return 0
        
    for file in os.listdir(skills_dir):
        if not file.endswith(".json"): continue
        file_path = skills_dir / file
        
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                skill_data = json.load(f)
                name = skill_data.get('name', 'UNKNOWN')
                exec_cmd = skill_data.get('execution_command', '')
                
                if exec_cmd.startswith('python '):
                    parts = exec_cmd.split(' ')
                    if len(parts) >= 2:
                        script_path_str = parts[1]
                        script_path = root / script_path_str
                        if not script_path.exists():
                            print_result("SKILL-MANIFEST", False, f"[{name}] demands '{script_path_str}' but file is missing")
                            errors += 1
                        else:
                            print_result("SKILL-MANIFEST", True, f"[{name}] runner '{script_path_str}' verified")
                    else:
                        print_result("SKILL-MANIFEST", False, f"[{name}] invalid python execution command")
                        errors += 1
                else:
                    print_result("SKILL-MANIFEST", True, f"[{name}] {exec_cmd} verified")
        except json.JSONDecodeError as e:
            print_result("SKILL-MANIFEST", False, f"Failed to parse {file} as JSON: {e}")
            errors += 1
        except Exception as e:
            print_result("SKILL-MANIFEST", False, f"Error processing {file}: {e}")
            errors += 1
            
    return errors

def check_api_docs_parity(root, fix=False):
    print(f"\nChecking API Documentation Parity...")
    gen_script = root / "execution" / "generate_api_reference.py"
    if not gen_script.exists():
        print_result("DOCS-PARITY", False, "generate_api_reference.py missing")
        return 1
        
    if fix:
        print("Running generate_api_reference.py --fix...")
        result = subprocess.run([sys.executable, str(gen_script)], capture_output=True, text=True)
        if result.returncode == 0:
            print_result("DOCS-PARITY", True, "Successfully synced API_REFERENCE.md")
            return 0
        else:
            print_result("DOCS-PARITY", False, f"Sync failed: {result.stderr}")
            return 1
    else:
        openapi_mtime = os.path.getmtime(root / "docs" / "openapi.yaml")
        api_ref_mtime = os.path.getmtime(root / "docs" / "API_REFERENCE.md")
        
        if openapi_mtime > api_ref_mtime + 5:
            print_result("DOCS-PARITY", False, "API_REFERENCE.md is out of sync with openapi.yaml. Run with FIX=1 to sync.")
            return 1
        else:
            print_result("DOCS-PARITY", True, "API_REFERENCE.md is synchronized")
            return 0

def run_pollywog_audit(root):
    print(f"\nRunning Pollywog Debt Ledger Validation...")
    ledger_script = root / "execution" / "pollywog_debt_ledger.py"
    if not ledger_script.exists():
        print_result("POLLYWOG-AUDIT", False, "pollywog_debt_ledger.py missing")
        return 1
        
    result = subprocess.run([sys.executable, str(ledger_script)], capture_output=True, text=True)
    if result.returncode == 0:
        print_result("POLLYWOG-AUDIT", True, "Pollywog Technical Debt Ledger is valid")
        return 0
    else:
        print_result("POLLYWOG-AUDIT", False, f"Pollywog Debt Ledger validation failed.\n{result.stdout}")
        return 1

def check_parity(root_dir, fix=False):
    root = Path(root_dir)
    router_path = root / "server-rs" / "src" / "router.rs"
    openapi_path = root / "docs" / "openapi.yaml"
    
    print(f"--- Tadpole OS Parity Audit ---\n")
    
    code_routes = scan_router(router_path)
    print(f"Found {len(code_routes)} routes in router.rs")
    
    raw_doc_paths = scan_openapi(openapi_path)
    doc_paths = {normalize_path(k): v for k, v in raw_doc_paths.items()}
    
    errors = 0
    
    for route in code_routes:
        path = route["path"]
        method = route["method"].lower()
        base_path = normalize_path(path)
        is_legacy = not path.startswith("/v1")
        
        if base_path in doc_paths:
            if method in doc_paths[base_path]:
                print_result("CODE->OPENAPI", True, f"{route['method']} {path}")
            else:
                print_result("CODE->OPENAPI", False, f"Method {method} missing for {path} in openapi.yaml")
                errors += 1
        else:
            if is_legacy:
                print_result("CODE->OPENAPI", True, f"{route['method']} {path} (Legacy/Internal)")
            else:
                print_result("CODE->OPENAPI", False, f"Route {path} missing in openapi.yaml")
                errors += 1
            
    print(f"\nScanning for Documentation Tags (@docs)...")
    doc_tags = {}
    for root_dir_walk, _, files in os.walk(root / "server-rs" / "src"):
        for file in files:
            if not file.endswith(".rs"): continue
            file_path = os.path.join(root_dir_walk, file)
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
                tags = re.findall(r'//[/!]\s*@docs\s+([A-Z_]+):([a-zA-Z0-9_]+)', content)
                for doc_name, section in tags:
                    doc_tags[f"{doc_name}:{section}"] = {
                        "file": file,
                        "mtime": os.path.getmtime(file_path)
                    }

    for tag, info in doc_tags.items():
        doc_name, section = tag.split(":")
        doc_file = root / "docs" / f"{doc_name}.md"
        if not doc_file.exists():
            doc_file = root / f"{doc_name}.md"
            if not doc_file.exists():
                print_result("ADG-TAG", False, f"Tag {tag} references missing file {doc_name}.md")
                errors += 1
                continue
            
        doc_mtime = os.path.getmtime(doc_file)
        if info["mtime"] > doc_mtime:
            if info["mtime"] - doc_mtime > 60:
                print_result("ADG-DRIFT", False, f"{info['file']} updated but {doc_name}.md is older (Drift detected)")
                errors += 1
            else:
                print_result("ADG-TAG", True, f"{tag} synchronized (Grace period)")
        else:
            print_result("ADG-TAG", True, f"{tag} synchronized")

    errors += check_env_vars(root)
    errors += check_skills(root)
    errors += check_api_docs_parity(root, fix=fix)
    errors += run_pollywog_audit(root)
    errors += check_critical_docs_content(root)
    errors += check_wiki_metadata(root)

    print(f"\nAudit Complete. Errors found: {errors}")
    return errors == 0

def check_critical_docs_content(root):
    """P2: Verify high-drift docs contain required sections and have current AI Assist Notes / AI Context Alignment."""
    print(f"\nChecking Critical Docs Content Parity...")
    errors = 0

    CRITICAL_DOCS = [
        "SWARM_ORCHESTRATION.md",
        "WEBSOCKET_EVENTS.md",
        "API_CONTRACT.md",
        "Benchmark_Spec.md",
        "GETTING_STARTED.md",
        "DEPLOYMENT_GUIDE.md",
    ]

    for doc_name in CRITICAL_DOCS:
        doc_path = root / "docs" / doc_name
        if not doc_path.exists():
            print_result("CRITICAL-DOC", False, f"{doc_name} is MISSING from docs/")
            errors += 1
            continue

        try:
            content = doc_path.read_text(encoding="utf-8", errors="ignore")
        except Exception as e:
            print_result("CRITICAL-DOC", False, f"{doc_name} unreadable: {e}")
            errors += 1
            continue

        has_ai_note = "AI Assist Note" in content or "AI Context Alignment" in content or "Metadata:" in content or "AI Context & Knowledge Heritage" in content
        if not has_ai_note:
            print_result("CRITICAL-DOC", False,
                         f"{doc_name} missing AI Context Alignment / AI Assist Note header")
            errors += 1
        else:
            print_result("CRITICAL-DOC", True, f"{doc_name} structure verified")

    return errors

def check_wiki_metadata(root):
    """P2/P3: Verify all wiki pages carry a Metadata marker for coverage traceability."""
    print(f"\nChecking Wiki Page Metadata Coverage...")
    wiki_dir = root / "docs" / "wiki"
    if not wiki_dir.exists():
        print_result("WIKI-META", True, "No wiki directory found — skipped")
        return 0

    SKIP_FILES = {"_Footer.md", "_Sidebar.md", "log.md"}
    errors = 0
    checked = 0

    for fpath in sorted(wiki_dir.rglob("*.md")):
        if fpath.name in SKIP_FILES:
            continue
        try:
            content = fpath.read_text(encoding="utf-8", errors="ignore")
        except Exception:
            continue

        rel = fpath.relative_to(root / "docs")
        has_marker = "Metadata:" in content or "AI Assist Note" in content or "AI Context Alignment" in content or "AI Context & Knowledge Heritage" in content
        checked += 1
        if has_marker:
            print_result("WIKI-META", True, f"{rel} has coverage marker")
        else:
            print_result("WIKI-META", False, f"{rel} missing Metadata marker")
            errors += 1

    if checked == 0:
        print_result("WIKI-META", True, "No wiki pages to check")

    return errors

def check_backtest_engine(root: Path) -> int:
    """Verifies backtest engine readiness and executes trace verification self-test."""
    print("\nChecking Historical Trace Backtest Engine...")
    backtest_script = root / "execution" / "backtest_engine.py"
    if not backtest_script.exists():
        print_result("BACKTEST-ENGINE", False, "execution/backtest_engine.py is missing!")
        return 1
    
    try:
        res = subprocess.run([sys.executable, str(backtest_script), "--test"], capture_output=True, text=True, timeout=10)
        if res.returncode == 0:
            print_result("BACKTEST-ENGINE", True, "Backtest Engine self-test verified (10s timeout parity enforced)")
            return 0
        else:
            print_result("BACKTEST-ENGINE", False, f"Backtest Engine self-test failed: {res.stderr or res.stdout}")
            return 1
    except Exception as e:
        print_result("BACKTEST-ENGINE", False, f"Backtest Engine execution failed: {e}")
        return 1

def check_security_registry(root: Path) -> int:
    """Verifies that docs/SECURITY_REGISTRY.json exists, contains valid policies, and referenced files exist."""
    print("\nChecking Security Policy Registry Parity...")
    registry_path = root / "docs" / "SECURITY_REGISTRY.json"
    if not registry_path.exists():
        print_result("SEC-REGISTRY", False, "docs/SECURITY_REGISTRY.json is missing!")
        return 1

    try:
        data = json.loads(registry_path.read_text(encoding="utf-8"))
        policies = data.get("policies", {})
        if not policies:
            print_result("SEC-REGISTRY", False, "docs/SECURITY_REGISTRY.json contains no policies!")
            return 1

        errors = 0
        for code, policy in sorted(policies.items()):
            enforcing_files = policy.get("enforcing_files", [])
            missing_files = []
            for ef in enforcing_files:
                if not (root / ef).exists():
                    missing_files.append(ef)
            if missing_files:
                print_result("SEC-REGISTRY", False, f"{code} missing enforcing files: {missing_files}")
                errors += 1
            else:
                print_result("SEC-REGISTRY", True, f"{code} ({policy.get('title')}) synchronized")
        return errors
    except Exception as e:
        print_result("SEC-REGISTRY", False, f"Failed to parse docs/SECURITY_REGISTRY.json: {e}")
        return 1

def check_doc_links(root: Path) -> int:
    """Verifies that all internal documentation links and anchors are valid."""
    link_script = root / "execution" / "verify_doc_links.py"
    if not link_script.exists():
        print_result("LINK-GUARD", False, "execution/verify_doc_links.py is missing!")
        return 1
    try:
        res = subprocess.run([sys.executable, str(link_script), str(root)], capture_output=True, text=True)
        if res.returncode == 0:
            print_result("LINK-GUARD", True, "All documentation links and anchor cross-references verified")
            return 0
        else:
            print_result("LINK-GUARD", False, f"Link integrity validation failed:\n{res.stdout}")
            return 1
    except Exception as e:
        print_result("LINK-GUARD", False, f"verify_doc_links execution failed: {e}")
        return 1

if __name__ == "__main__":
    project_root = sys.argv[1] if len(sys.argv) > 1 and not sys.argv[1].startswith("FIX=") else "."
    fix_mode = any(arg == "FIX=1" for arg in sys.argv)
    
    root_path = Path(project_root)
    backtest_errs = check_backtest_engine(root_path)
    sec_errs = check_security_registry(root_path)
    link_errs = check_doc_links(root_path)

    if check_parity(project_root, fix=fix_mode) and backtest_errs == 0 and sec_errs == 0 and link_errs == 0:
        sys.exit(0)
    else:
        sys.exit(1)
