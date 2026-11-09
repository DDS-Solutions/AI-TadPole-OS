"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / parity_validators
- **Primary Entrypoints**: `check_skills`, `check_api_docs_parity`, `run_pollywog_audit`, `check_critical_docs_content`, `check_wiki_metadata`, `check_backtest_engine`, `check_security_registry`, `check_doc_links`, `check_dual_plane_identity_parity`, `check_rust_test_compilation`

### ⚠️ Non-Negotiables
- `[Structural]` Deterministic policy and documentation integrity validation.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
"""

import os
import re
import json
import sys
import subprocess
from pathlib import Path
from typing import Callable, List, Dict, Any

PrintFn = Callable[[str, bool, str], None]

def _verify_single_skill_manifest(file_path: Path, root: Path, print_fn: PrintFn) -> int:
    """Verifies that a single skill manifest references valid execution scripts."""
    try:
        skill_data = json.loads(file_path.read_text(encoding='utf-8'))
        name = skill_data.get('name', 'UNKNOWN')
        exec_cmd = skill_data.get('execution_command', '')

        if not exec_cmd.startswith('python '):
            print_fn("SKILL-MANIFEST", True, f"[{name}] {exec_cmd} verified")
            return 0

        parts = exec_cmd.split(' ')
        if len(parts) < 2:
            print_fn("SKILL-MANIFEST", False, f"[{name}] invalid python execution command")
            return 1

        script_path_str = parts[1]
        script_path = root / script_path_str
        if not script_path.exists():
            print_fn("SKILL-MANIFEST", False, f"[{name}] demands '{script_path_str}' but file is missing")
            return 1

        print_fn("SKILL-MANIFEST", True, f"[{name}] runner '{script_path_str}' verified")
        return 0
    except json.JSONDecodeError as err:
        print_fn("SKILL-MANIFEST", False, f"Failed to parse {file_path.name} as JSON: {err}")
        return 1
    except OSError as err:
        print_fn("SKILL-MANIFEST", False, f"Error processing {file_path.name}: {err}")
        return 1

def check_skills(root: Path, print_fn: PrintFn) -> int:
    """Validates that skill manifests reference existing runner scripts."""
    print("\nScanning Skills & Workflows...")
    skills_dir = root / "server-rs" / "data" / "skills"
    if not skills_dir.exists():
        return 0

    errors = 0
    for file in os.listdir(skills_dir):
        if file.endswith(".json"):
            errors += _verify_single_skill_manifest(skills_dir / file, root, print_fn)
    return errors

def check_api_docs_parity(root: Path, print_fn: PrintFn, fix: bool = False) -> int:
    """Checks API_REFERENCE.md synchronization with docs/openapi.yaml."""
    print("\nChecking API Documentation Parity...")
    gen_script = root / "execution" / "generate_api_reference.py"
    if not gen_script.exists():
        print_fn("DOCS-PARITY", False, "generate_api_reference.py missing")
        return 1

    if fix:
        print("Running generate_api_reference.py --fix...")
        result = subprocess.run([sys.executable, str(gen_script), "--fix"], capture_output=True, text=True, cwd=str(root))
        if result.returncode == 0:
            print_fn("DOCS-PARITY", True, "Successfully synced API_REFERENCE.md")
            return 0
        print_fn("DOCS-PARITY", False, f"Sync failed: {result.stderr or result.stdout}")
        return 1

    openapi_file = root / "docs" / "openapi.yaml"
    api_ref_file = root / "docs" / "API_REFERENCE.md"
    if not openapi_file.is_file() or not api_ref_file.is_file():
        print_fn("DOCS-PARITY", False, "docs/openapi.yaml or docs/API_REFERENCE.md missing")
        return 1

    if os.path.getmtime(openapi_file) > os.path.getmtime(api_ref_file) + 5:
        print_fn("DOCS-PARITY", False, "API_REFERENCE.md is out of sync with openapi.yaml. Run with FIX=1 to sync.")
        return 1

    print_fn("DOCS-PARITY", True, "API_REFERENCE.md is synchronized")
    return 0

def run_pollywog_audit(root: Path, print_fn: PrintFn) -> int:
    """Executes the pollywog debt ledger verification script."""
    print("\nRunning Pollywog Debt Ledger Validation...")
    ledger_script = root / "execution" / "pollywog_debt_ledger.py"
    if not ledger_script.exists():
        print_fn("POLLYWOG-AUDIT", False, "pollywog_debt_ledger.py missing")
        return 1

    result = subprocess.run([sys.executable, str(ledger_script)], capture_output=True, text=True)
    if result.returncode == 0:
        print_fn("POLLYWOG-AUDIT", True, "Pollywog Technical Debt Ledger is valid")
        return 0

    print_fn("POLLYWOG-AUDIT", False, f"Pollywog Debt Ledger validation failed.\n{result.stdout}")
    return 1

def check_critical_docs_content(root: Path, print_fn: PrintFn) -> int:
    """Verifies that high-drift critical documents contain required AI context headers."""
    print("\nChecking Critical Docs Content Parity...")
    critical_docs = [
        "SWARM_ORCHESTRATION.md",
        "WEBSOCKET_EVENTS.md",
        "API_CONTRACT.md",
        "Benchmark_Spec.md",
        "GETTING_STARTED.md",
        "DEPLOYMENT_GUIDE.md",
    ]
    errors = 0
    valid_markers = ("AI Assist Note", "AI Context Alignment", "Metadata:", "AI Context & Knowledge Heritage")

    for doc_name in critical_docs:
        doc_path = root / "docs" / doc_name
        if not doc_path.exists():
            print_fn("CRITICAL-DOC", False, f"{doc_name} is MISSING from docs/")
            errors += 1
            continue

        try:
            content = doc_path.read_text(encoding="utf-8", errors="ignore")
            if any(m in content for m in valid_markers):
                print_fn("CRITICAL-DOC", True, f"{doc_name} structure verified")
            else:
                print_fn("CRITICAL-DOC", False, f"{doc_name} missing AI Context Alignment / AI Assist Note header")
                errors += 1
        except OSError as err:
            print_fn("CRITICAL-DOC", False, f"{doc_name} unreadable: {err}")
            errors += 1

    return errors

def check_wiki_metadata(root: Path, print_fn: PrintFn) -> int:
    """Verifies that all wiki pages carry a metadata marker for traceability."""
    print("\nChecking Wiki Page Metadata Coverage...")
    wiki_dir = root / "docs" / "wiki"
    if not wiki_dir.exists():
        print_fn("WIKI-META", True, "No wiki directory found — skipped")
        return 0

    skip_files = {"_Footer.md", "_Sidebar.md", "log.md"}
    valid_markers = ("Metadata:", "AI Assist Note", "AI Context Alignment", "AI Context & Knowledge Heritage")
    errors = 0
    checked = 0

    for fpath in sorted(wiki_dir.rglob("*.md")):
        if fpath.name in skip_files:
            continue
        try:
            content = fpath.read_text(encoding="utf-8", errors="ignore")
        except (OSError, UnicodeDecodeError):
            continue

        rel = fpath.relative_to(root / "docs")
        checked += 1
        if any(marker in content for marker in valid_markers):
            print_fn("WIKI-META", True, f"{rel} has coverage marker")
        else:
            print_fn("WIKI-META", False, f"{rel} missing Metadata marker")
            errors += 1

    if checked == 0:
        print_fn("WIKI-META", True, "No wiki pages to check")
    return errors

def check_backtest_engine(root: Path, print_fn: PrintFn) -> int:
    """Verifies backtest engine readiness and executes trace verification self-test."""
    print("\nChecking Historical Trace Backtest Engine...")
    backtest_script = root / "execution" / "backtest_engine.py"
    if not backtest_script.exists():
        print_fn("BACKTEST-ENGINE", False, "execution/backtest_engine.py is missing!")
        return 1

    try:
        res = subprocess.run([sys.executable, str(backtest_script), "--test"], capture_output=True, text=True, timeout=10)
        if res.returncode == 0:
            print_fn("BACKTEST-ENGINE", True, "Backtest Engine self-test verified (10s timeout parity enforced)")
            return 0
        print_fn("BACKTEST-ENGINE", False, f"Backtest Engine self-test failed: {res.stderr or res.stdout}")
        return 1
    except (subprocess.SubprocessError, OSError) as err:
        print_fn("BACKTEST-ENGINE", False, f"Backtest Engine execution failed: {err}")
        return 1

def check_security_registry(root: Path, print_fn: PrintFn) -> int:
    """Verifies docs/SECURITY_REGISTRY.json and existence of enforcing files."""
    print("\nChecking Security Policy Registry Parity...")
    registry_path = root / "docs" / "SECURITY_REGISTRY.json"
    if not registry_path.exists():
        print_fn("SEC-REGISTRY", False, "docs/SECURITY_REGISTRY.json is missing!")
        return 1

    try:
        data = json.loads(registry_path.read_text(encoding="utf-8"))
        policies = data.get("policies", {})
        if not policies:
            print_fn("SEC-REGISTRY", False, "docs/SECURITY_REGISTRY.json contains no policies!")
            return 1

        errors = 0
        for code, policy in sorted(policies.items()):
            enforcing_files = policy.get("enforcing_files", [])
            missing_files = [ef for ef in enforcing_files if not (root / ef).exists()]
            if missing_files:
                print_fn("SEC-REGISTRY", False, f"{code} missing enforcing files: {missing_files}")
                errors += 1
            else:
                print_fn("SEC-REGISTRY", True, f"{code} ({policy.get('title')}) synchronized")
        return errors
    except (json.JSONDecodeError, OSError) as err:
        print_fn("SEC-REGISTRY", False, f"Failed to parse docs/SECURITY_REGISTRY.json: {err}")
        return 1

def check_doc_links(root: Path, print_fn: PrintFn) -> int:
    """Verifies that all internal documentation links and anchors are valid."""
    link_script = root / "execution" / "verify_doc_links.py"
    if not link_script.exists():
        print_fn("LINK-GUARD", False, "execution/verify_doc_links.py is missing!")
        return 1
    try:
        res = subprocess.run([sys.executable, str(link_script), str(root)], capture_output=True, text=True)
        if res.returncode == 0:
            print_fn("LINK-GUARD", True, "All documentation links and anchor cross-references verified")
            return 0
        print_fn("LINK-GUARD", False, f"Link integrity validation failed:\n{res.stdout}")
        return 1
    except (subprocess.SubprocessError, OSError) as err:
        print_fn("LINK-GUARD", False, f"verify_doc_links execution failed: {err}")
        return 1

def verify_canonical_dual_plane_docs(root: Path, print_fn: PrintFn) -> int:
    """Verifies canonical dual-plane markdown documents contain required markers."""
    canonical_docs = [
        "directives/IDENTITY.md",
        "AGENTS.md",
        "GEMINI.md",
        "CLAUDE.md",
        "README.md",
        "DEVELOPMENT.md",
        "ROADMAP.md",
        "docs/ARCHITECTURE.md",
        "docs/Architecture_Overview.md",
        "docs/Security_Model.md",
        "docs/SECURITY.md",
        ".agent/ARCHITECTURE.md",
        ".agent/skills/intelligent-routing/SKILL.md",
        ".agent/skills/behavioral-modes/SKILL.md",
        ".agent/skills/digital-twin-voice/SKILL.md",
        ".agent/skills/smb-data-ingestion/SKILL.md",
        ".agent/skills/pii-redaction/SKILL.md",
    ]
    errors = 0
    for rel_path in canonical_docs:
        fpath = root / rel_path
        if not fpath.exists():
            print_fn("DUAL-PLANE", False, f"Missing canonical Dual-Plane document: {rel_path}")
            errors += 1
            continue
        text = fpath.read_text(encoding="utf-8", errors="replace")
        if "FR-11" not in text or ("Plane 1" not in text and "Plane 2" not in text):
            print_fn("DUAL-PLANE", False, f"{rel_path} missing Dual-Plane (Plane 1/Plane 2) or FR-11 firewall markers")
            errors += 1
    return errors

def verify_markdown_dual_plane_references(root: Path, product_ver: str, print_fn: PrintFn) -> int:
    """Scans all repository markdown files for stale IDENTITY.md versions or missing schema fields."""
    exclude_dirs = {"node_modules", "target", ".git", "dist", ".tmp", "archive"}
    stale_ver_pattern = re.compile(rf"IDENTITY\.md\s+v(?!{re.escape(product_ver)}\b)\d+\.\d+(?:\.\d+)?", re.IGNORECASE)
    errors = 0

    for current_root, dirnames, filenames in os.walk(root):
        dirnames[:] = [d for d in dirnames if d not in exclude_dirs]
        for fname in filenames:
            if not fname.endswith(".md"):
                continue
            md_path = Path(current_root) / fname
            rel = md_path.relative_to(root).as_posix()
            text = md_path.read_text(encoding="utf-8", errors="replace")
            if stale_ver_pattern.search(text):
                print_fn("DUAL-PLANE", False, f"{rel} contains stale IDENTITY.md version reference (expected v{product_ver})")
                errors += 1
            if "Neural Lineage Schema" in text and "[Execution-Plane]" not in text and "documentation_drift_report.md" not in rel:
                print_fn("DUAL-PLANE", False, f"{rel} references Neural Lineage Schema without [Execution-Plane] field")
                errors += 1
    return errors

def check_dual_plane_identity_parity(root: Path, print_fn: PrintFn) -> int:
    """Deterministically verifies Dual-Plane Sovereign Hierarchy and IDENTITY.md version parity."""
    print("\nChecking Dual-Plane Sovereign Hierarchy & IDENTITY.md Parity...")
    try:
        version_data = json.loads((root / "version.json").read_text(encoding="utf-8"))
        product_ver = version_data.get("version", "")
    except (json.JSONDecodeError, OSError) as err:
        print_fn("DUAL-PLANE", False, f"Failed to read version.json: {err}")
        return 1

    errors = verify_canonical_dual_plane_docs(root, print_fn)
    errors += verify_markdown_dual_plane_references(root, product_ver, print_fn)

    if errors == 0:
        print_fn("DUAL-PLANE", True, f"All canonical Dual-Plane docs & repository-wide IDENTITY.md (v{product_ver}) references synchronized")
    return errors

def check_rust_test_compilation(root: Path, print_fn: PrintFn) -> int:
    """Verifies that all server-rs test targets compile cleanly via cargo test --no-run."""
    if os.environ.get("SKIP_RUST_COMPILE") == "1":
        print_fn("RUST-TEST-COMPILE", True, "Skipping redundant Rust compilation (covered by dedicated rust-check job)")
        return 0

    manifest = root / "server-rs" / "Cargo.toml"
    if not manifest.exists():
        return 0

    print("\nChecking Rust Test Target Compilation (cargo test --no-run)...")
    try:
        res = subprocess.run(
            ["cargo", "test", "--no-run", "--manifest-path", str(manifest)],
            capture_output=True,
            text=True,
        )
        if res.returncode == 0:
            print_fn("RUST-TEST-COMPILE", True, "All server-rs test targets compile cleanly")
            return 0
        err_snippet = res.stderr[-500:] if res.stderr else res.stdout[-500:]
        print_fn("RUST-TEST-COMPILE", False, f"cargo test --no-run failed:\n{err_snippet}")
        return 1
    except (subprocess.SubprocessError, OSError) as err:
        print_fn("RUST-TEST-COMPILE", False, f"cargo test --no-run invocation error: {err}")
        return 1

