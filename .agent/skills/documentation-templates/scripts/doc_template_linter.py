#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Registry:Skills

### AI Context Alignment
- **Subsystem**: Agent Skills / documentation-templates / doc_template_linter
- **Primary Entrypoints**: `lint_adr_document`, `lint_markdown_ai_context`, `lint_documentation_file`, `run_self_tests`, `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic validation of markdown documentation templates, ADRs, and AI context blocks without modifying files.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[doc_template_linter]`, `[PASS]`, `[FAIL]`, `[DOCS]`, `[LINT]`
- **Witness Tests**: none declared
"""

import sys
import io
import os
import json
import re
import argparse
from pathlib import Path
from datetime import datetime
from typing import List, Dict, Any, Tuple, Optional

# Fix Windows console encoding
if sys.platform == "win32":
    try:
        sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
        sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8')
    except Exception:
        pass


def lint_adr_document(content: str) -> List[str]:
    """
    Validate Architecture Decision Record (ADR) structure against Sovereign standards.
    Required sections:
    - Title: # ADR-XX: Title
    - Status: ## Status (Proposed, Accepted, Deprecated, Superseded)
    - Context: ## Context & Problem Statement (or ## Context)
    - Decision: ## Decision
    - Consequences: ## Consequences
    """
    issues: List[str] = []
    
    # 1. Title format
    if not re.search(r'^[ \t]*#\s+ADR-\d+[:\s]', content, re.MULTILINE | re.IGNORECASE):
        issues.append("Missing standard ADR title format: '# ADR-XX: [Title of Decision]'")
        
    # 2. Status section
    status_match = re.search(r'^[ \t]*##\s+Status\s*\n+([^\n#]+)', content, re.MULTILINE | re.IGNORECASE)
    if not status_match:
        issues.append("Missing required '## Status' section")
    else:
        status_val = status_match.group(1).strip().lower()
        valid_statuses = {"proposed", "accepted", "deprecated", "superseded"}
        if not any(s in status_val for s in valid_statuses):
            issues.append(f"Status '{status_val}' should be one of: Proposed, Accepted, Deprecated, Superseded")
            
    # 3. Context section
    if not re.search(r'^[ \t]*##\s+Context(?:\s+&\s+Problem\s+Statement)?\b', content, re.MULTILINE | re.IGNORECASE):
        issues.append("Missing required '## Context & Problem Statement' section")
        
    # 4. Decision section
    if not re.search(r'^[ \t]*##\s+Decision\b', content, re.MULTILINE | re.IGNORECASE):
        issues.append("Missing required '## Decision' section")
        
    # 5. Consequences section
    if not re.search(r'^[ \t]*##\s+Consequences\b', content, re.MULTILINE | re.IGNORECASE):
        issues.append("Missing required '## Consequences' section")
        
    return issues


def lint_markdown_ai_context(content: str) -> List[str]:
    """
    Validate Sovereign AI Context & Knowledge Heritage block in documentation files.
    """
    issues: List[str] = []
    
    if "AI Context & Knowledge Heritage" not in content:
        issues.append("Missing '> [!IMPORTANT]' block with 'AI Context & Knowledge Heritage'")
        return issues
        
    # Check Subsystem
    if not re.search(r'-\s+\*\*Subsystem\*\*:\s*([^\n]+)', content):
        issues.append("AI Context block missing '- **Subsystem**:' field")
        
    # Check Architecture
    arch_match = re.search(r'-\s+\*\*Architecture\*\*:\s*([^\n]+)', content)
    if not arch_match:
        issues.append("AI Context block missing '- **Architecture**:' field")
    elif "@docs" not in arch_match.group(1):
        issues.append("AI Context Architecture field should include '@docs ARCHITECTURE:...' link")
        
    # Check Failure Path
    if not re.search(r'-\s+\*\*Failure Path\*\*:\s*([^\n]+)', content):
        issues.append("AI Context block missing '- **Failure Path**:' field")
        
    # Check Observability
    if not re.search(r'-\s+\*\*Observability\*\*:\s*([^\n]+)', content):
        issues.append("AI Context block missing '- **Observability**:' field")
        
    return issues


def lint_documentation_file(file_path: Path) -> List[str]:
    """
    Audit a single documentation or template markdown file for structural consistency.
    """
    try:
        content = file_path.read_text(encoding='utf-8', errors='ignore')
    except Exception as e:
        return [f"Unable to read file: {e}"]
        
    issues: List[str] = []
    fname = file_path.name.lower()
    
    # 1. ADR check if filename starts with adr-
    if fname.startswith("adr-") or "adr" in file_path.parts:
        if fname.endswith(".md") and fname != "readme.md":
            issues.extend(lint_adr_document(content))
            
    # 2. Reference / Guide AI context check
    is_reference_doc = (
        "references" in file_path.parts
        or "skills" in file_path.parts
        or (file_path.parent.name == "docs" and fname not in {"error_registry.json", "telemetry_map.json"})
    )
    
    if is_reference_doc and file_path.suffix.lower() == ".md":
        # Only require AI Context if not a brief index or changelog
        if fname not in {"index.md", "changelog_reconstruction.md", "public", ".vitepress"}:
            ai_issues = lint_markdown_ai_context(content)
            issues.extend(ai_issues)
            
    return issues


def run_self_tests() -> bool:
    """
    Execute self-tests verifying ADR validation, AI context block checking, and lint diagnostics.
    """
    print("[doc_template_linter] Executing self-test suite...")
    
    # 1. ADR Linting Tests
    bad_adr = """
    # My Decision
    Some notes on what we did.
    """
    adr_issues = lint_adr_document(bad_adr)
    assert any("title format" in i for i in adr_issues), "Failed to detect non-standard ADR title"
    assert any("Status" in i for i in adr_issues), "Failed to detect missing Status section"
    assert any("Decision" in i for i in adr_issues), "Failed to detect missing Decision section"
    
    good_adr = """
    # ADR-001: Adopt SQLite with sqlx in Rust Backend
    
    ## Status
    Accepted
    
    ## Context & Problem Statement
    We needed a local-first embedded database engine without daemon overhead.
    
    ## Decision
    Adopt SQLite compiled with sqlx async features.
    
    ## Consequences
    - **Positive**: Sub-millisecond reads, zero external dependencies.
    - **Negative**: Single-writer lock concurrency.
    """
    good_adr_issues = lint_adr_document(good_adr)
    assert len(good_adr_issues) == 0, f"Expected 0 issues for valid ADR, got {good_adr_issues}"
    
    # 2. AI Context Block Tests
    bad_context = """
    # My Document
    > [!IMPORTANT]
    > **AI Context & Knowledge Heritage**
    > - **Subsystem**: Core Subsystem
    """
    ctx_issues = lint_markdown_ai_context(bad_context)
    assert any("Architecture" in i for i in ctx_issues), "Failed to detect missing Architecture field"
    assert any("Failure Path" in i for i in ctx_issues), "Failed to detect missing Failure Path field"
    assert any("Observability" in i for i in ctx_issues), "Failed to detect missing Observability field"
    
    good_context = """
    > [!IMPORTANT]
    > **AI Context & Knowledge Heritage**
    > - **Subsystem**: Agent Skills Registry / documentation-templates
    > - **Architecture**: `@docs ARCHITECTURE:Documentation`
    > - **Failure Path**: Stale documentation or unmigrated decision records.
    > - **Observability**: Traceability via `execution/parity_guard.py` (`[DOCS]`)
    """
    good_ctx_issues = lint_markdown_ai_context(good_context)
    assert len(good_ctx_issues) == 0, f"Expected 0 issues for valid AI Context block, got {good_ctx_issues}"
    
    print("[doc_template_linter] [PASS] All self-tests passed successfully.")
    return True


def main():
    parser = argparse.ArgumentParser(description="Deterministic Documentation Template Linter")
    parser.add_argument("path", nargs="?", default=".", help="Target file or directory to lint")
    parser.add_argument("--test", action="store_true", help="Execute self-test suite and exit")
    parser.add_argument("--json", action="store_true", help="Output machine-readable JSON report")
    
    args = parser.parse_args()
    
    if args.test:
        try:
            success = run_self_tests()
            sys.exit(0 if success else 1)
        except AssertionError as e:
            print(f"[doc_template_linter] [FAIL] Self-test assertion failed: {e}")
            sys.exit(1)
            
    target_path = Path(args.path).resolve()
    
    print("[doc_template_linter] Initializing documentation template linting...")
    print(f"[DOCS] Scanning target: {target_path}")
    
    files_to_check: List[Path] = []
    if target_path.is_file():
        if target_path.suffix.lower() == ".md":
            files_to_check.append(target_path)
    else:
        for root, dirs, files in os.walk(target_path):
            dirs[:] = [d for d in dirs if d not in {'.git', 'node_modules', 'dist', 'target', '.tmp', 'tmp'}]
            for f in files:
                if f.lower().endswith(".md"):
                    files_to_check.append(Path(root) / f)
                    
    results = []
    total_issues = 0
    
    for f in files_to_check:
        print(f"[LINT] Auditing {f.name}...")
        issues = lint_documentation_file(f)
        if issues:
            total_issues += len(issues)
            results.append({
                "file": str(f.name),
                "path": str(f),
                "issues": issues
            })
            
    passed = total_issues == 0
    report = {
        "script": "doc_template_linter",
        "timestamp": datetime.now().isoformat(),
        "target": str(target_path),
        "files_checked": len(files_to_check),
        "issues_found": total_issues,
        "passed": passed,
        "results": results
    }
    
    if args.json:
        print(json.dumps(report, indent=2))
        sys.exit(0 if passed else 1)
        
    print(f"\n{'='*64}")
    print(f"📖 [DOC TEMPLATE LINTER] Documentation Template Verification")
    print(f"{'='*64}")
    print(f"Target Directory : {target_path}")
    print(f"Markdown Files   : {len(files_to_check)}")
    print(f"Issues Detected  : {total_issues}")
    print(f"Timestamp        : {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    print(f"{'-'*64}")
    
    if results:
        for item in results[:10]:
            print(f"\n📂 {item['file']}:")
            for issue in item['issues'][:5]:
                print(f"   ⚠️  {issue}")
            if len(item['issues']) > 5:
                print(f"   ... and {len(item['issues']) - 5} more issues")
        if len(results) > 10:
            print(f"\n... and {len(results) - 10} more files with findings.")
    else:
        print("\n🎉 [PASS] All audited documentation files match template standards!")
        
    print(f"\n{'='*64}")
    sys.exit(0 if passed else 1)


if __name__ == "__main__":
    main()
