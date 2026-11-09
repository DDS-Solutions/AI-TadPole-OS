"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / verify_ai_context
- **Primary Entrypoints**: `strip_comments_and_strings`, `get_workspace_test_symbols`, `verify_symbol_defined_in_file`, `extract_code_body`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Invariant taxonomy enforcement: bullets must be marked [Structural], [Behavioral], or [Advisory: UNVERIFIED].
  - enforced_by: `test_invariant_taxonomy_enforcement`
- `[Behavioral]` Behavioral invariants must provide a resolvable `enforced_by` witness test in the workspace test suite.
  - enforced_by: `test_behavioral_witness_enforcement`
- `[Structural]` Backticked entrypoint symbols must resolve to concrete definitions in the target file.
  - enforced_by: `test_symbol_definition_resolution`
- `[Structural]` Telemetry tags declared in header must be emitted via valid logging/printing constructs in the code body.
  - enforced_by: `test_telemetry_emitter_verification`

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[verify_ai_context]`
- **Witness Tests**: `test_invariant_taxonomy_enforcement`, `test_behavioral_witness_enforcement`, `test_symbol_definition_resolution`, `test_telemetry_emitter_verification`
"""

import os
import re
import sys
import json
import argparse
from pathlib import Path
from typing import List, Dict, Any, Optional, Set, Tuple
from datetime import datetime, timezone

# Add execution directory to sys.path for local module resolution
_EXEC_DIR = Path(__file__).resolve().parent
if str(_EXEC_DIR) not in sys.path:
    sys.path.insert(0, str(_EXEC_DIR))

from lib.ai_context_parser import (
    strip_comments_and_strings as _strip_comments_and_strings,
    extract_header_block as _extract_header_block,
    extract_code_body as _extract_code_body,
    extract_metadata as _extract_metadata,
)
from lib.ai_context_resolver import (
    verify_symbol_defined_in_file as _verify_symbol_defined_in_file,
    tag_is_emitted as _tag_is_emitted,
    harvest_file_tests as _harvest_file_tests,
    get_adg_manifest as _get_adg_manifest,
    get_manifest_witnessed_files as _get_manifest_witnessed_files,
    EMITTERS,
)

# --- Configuration & Globals ---
SKIP_DIRS = {
    '.git', 'node_modules', 'dist', 'target', 'build', '__pycache__',
    '.venv', 'venv', '.tmp', 'tmp', 'coverage', '3rdparty', 'wiki',
    'scratch', 'playwright-report', 'test-results'
}
EXTENSIONS = {'.rs', '.ts', '.tsx', '.js', '.jsx', '.py'}
ROOT = Path(__file__).resolve().parent.parent
ADG_MANIFEST_PATH = ROOT / 'adg.manifest.json'

_KNOWN_TESTS: Optional[Set[str]] = None
_ADG_MANIFEST: Optional[Dict[str, Any]] = None

def strip_comments_and_strings(content: str, file_ext: str) -> str:
    """Strips comments and strings for language extension."""
    return _strip_comments_and_strings(content, file_ext)

def extract_header_block(content: str, file_ext: str = "") -> str:
    """Extracts top header block from content."""
    return _extract_header_block(content, file_ext)

def extract_code_body(content: str, file_ext: str) -> str:
    """Returns code content excluding leading header comments/docstring."""
    return _extract_code_body(content, file_ext)

def extract_metadata(content: str, file_ext: str = "") -> Dict[str, Any]:
    """Extracts metadata dictionary from content header."""
    return _extract_metadata(content, file_ext)

def verify_symbol_defined_in_file(symbol: str, content: str, file_ext: str) -> bool:
    """Verifies backticked symbol is defined in target file."""
    return _verify_symbol_defined_in_file(symbol, content, file_ext)

def tag_is_emitted(tag: str, code_content: str) -> bool:
    """Verifies declared telemetry tag is emitted with a logger in code."""
    return _tag_is_emitted(tag, code_content)

def get_adg_manifest() -> Optional[Dict[str, Any]]:
    """Loads and caches the ADG v2 manifest if it exists."""
    global _ADG_MANIFEST
    if _ADG_MANIFEST is None:
        _ADG_MANIFEST = _get_adg_manifest(ADG_MANIFEST_PATH)
    return _ADG_MANIFEST

def get_manifest_witnessed_files() -> Set[str]:
    """Returns the set of source file paths that have witnesses in adg.manifest.json."""
    return _get_manifest_witnessed_files(ADG_MANIFEST_PATH)

def harvest_path_tests(p: Path) -> Set[str]:
    """Harvests tests from a single file path safely."""
    try:
        with open(p, 'r', encoding='utf-8', errors='ignore') as handle:
            raw_text = handle.read()
        return _harvest_file_tests(p, p.suffix.lower(), raw_text)
    except (OSError, UnicodeDecodeError):
        return set()

def get_workspace_test_symbols() -> Set[str]:
    """Harvests test symbols from the entire repository using language-specific parsers."""
    global _KNOWN_TESTS
    if _KNOWN_TESTS is not None:
        return _KNOWN_TESTS

    tests: Set[str] = set()
    for root_dir, dirs, files in os.walk(ROOT):
        dirs[:] = [d for d in dirs if d not in SKIP_DIRS]
        for f in files:
            p = Path(root_dir) / f
            if p.suffix.lower() in EXTENSIONS:
                tests.update(harvest_path_tests(p))

    _KNOWN_TESTS = tests
    return _KNOWN_TESTS

def _check_header_presence(meta: Dict[str, Any]) -> List[str]:
    """Checks presence of required header sections."""
    findings = []
    if not meta["has_note"]:
        findings.append("Missing '### AI Assist Note' or '### AI Context Alignment'")
    if not meta["has_debugging"]:
        findings.append("Missing '### 🔍 Debugging & Observability'")
    return findings

def _check_docs_link(meta: Dict[str, Any]) -> List[str]:
    """Validates existence of @docs target file."""
    if not meta["has_docs"]:
        return ["Missing '@docs' documentation link"]
    if not meta["docs_link"]:
        return []

    doc_identifier = meta["docs_link"].split(':')[0]
    candidates = [
        ROOT / "docs" / f"{doc_identifier}.md",
        ROOT / "docs" / f"{doc_identifier.lower()}.md",
        ROOT / f"{doc_identifier}.md",
        ROOT / f"{doc_identifier.lower()}.md",
        ROOT / "docs" / f"{doc_identifier}.json",
        ROOT / "docs" / f"{doc_identifier}.yaml",
        ROOT / "docs" / doc_identifier,
    ]
    if not any(c.exists() for c in candidates):
        return [f"Broken @docs link: Documentation file for '{meta['docs_link']}' not found"]
    return []

def _check_invariant_taxonomy(invariants: List[Dict[str, Any]]) -> List[str]:
    """Validates that each invariant bullet follows the required taxonomy."""
    findings = []
    for inv in invariants:
        if not inv["tag"]:
            findings.append(f"Uncategorized invariant: '{inv['text'][:60]}...' (must be marked [Structural], [Behavioral], or [Advisory: UNVERIFIED])")
        elif inv["tag"] == "Behavioral" and not inv["witness"]:
            findings.append(f"[Behavioral] invariant missing explicit 'enforced_by: `test_name`': '{inv['text'][:60]}...'")
    return findings

def _check_witness_resolution(meta: Dict[str, Any], all_tests: Set[str]) -> List[str]:
    """Validates that witness tests exist in the workspace test suite."""
    findings = []
    manifest_witnessed = get_manifest_witnessed_files()
    enforced_witnesses = {inv["witness"] for inv in meta["invariants"] if inv.get("witness")}
    declared_witnesses = set(meta["witness_tests"])

    for w_test in meta["witness_tests"]:
        if w_test.lower() not in {"none", "none declared", "n/a"} and w_test not in all_tests:
            findings.append(f"Witness Test `{w_test}` not found in repository test suite")

    for enf_w in enforced_witnesses:
        if enf_w and enf_w not in declared_witnesses:
            if not (manifest_witnessed and enf_w in all_tests):
                findings.append(f"Witness test `{enf_w}` is linked in an invariant but omitted from the 'Witness Tests' field")
        if enf_w and enf_w not in all_tests:
            findings.append(f"Invariant witness test `{enf_w}` not found in test suite")
    return findings

def _check_invariants_and_witnesses(meta: Dict[str, Any], all_tests: Set[str]) -> List[str]:
    """Validates invariant taxonomy categorization and witness test existence."""
    findings = _check_invariant_taxonomy(meta["invariants"])
    findings.extend(_check_witness_resolution(meta, all_tests))
    return findings

def verify_file(file_path: Path) -> Dict[str, Any]:
    """Runs all 7 AI Context Contract verification gates on a single source file."""
    try:
        content = file_path.read_text(encoding='utf-8', errors='ignore')
        file_ext = file_path.suffix.lower()
        meta = extract_metadata(content, file_ext)

        findings: List[str] = []
        findings.extend(_check_header_presence(meta))
        findings.extend(_check_docs_link(meta))

        code_body = extract_code_body(content, file_ext)
        for tag in meta["telemetry_tags"]:
            if not tag_is_emitted(tag, code_body):
                findings.append(f"Telemetry Tag '{tag}' defined in header but missing from executable log emitters in file body")

        for sym in meta["symbols"]:
            if not verify_symbol_defined_in_file(sym, content, file_ext):
                findings.append(f"Declared Symbol `{sym}` is not defined as an AST/code symbol in this file")

        all_known_tests = get_workspace_test_symbols()
        findings.extend(_check_invariants_and_witnesses(meta, all_known_tests))

        unique_findings = list(dict.fromkeys(findings))
        try:
            rel_path = file_path.relative_to(ROOT).as_posix()
        except ValueError:
            rel_path = file_path.as_posix()

        return {
            "file": rel_path,
            "passed": len(unique_findings) == 0,
            "findings": unique_findings,
            "meta": meta,
        }
    except OSError as err:
        return {
            "file": str(file_path).replace('\\', '/'),
            "passed": False,
            "findings": [f"Error processing file: {err}"],
        }

def load_baseline(baseline_file: Path) -> Set[Tuple[str, str]]:
    """Loads a baseline JSON file containing known (file, finding) tuples."""
    if not baseline_file.exists():
        return set()
    try:
        data = json.loads(baseline_file.read_text(encoding='utf-8'))
        baseline_set: Set[Tuple[str, str]] = set()
        for item in data.get("failures", []):
            fname = item.get("file", "").replace('\\', '/')
            for finding in item.get("findings", []):
                baseline_set.add((fname, finding))
        return baseline_set
    except (json.JSONDecodeError, OSError):
        return set()

def _validate_registry_entries(registry_data: Dict[str, Any]) -> List[str]:
    """Validates structural fields of ERROR_REGISTRY entries without jsonschema."""
    errors = []
    for code, entry in registry_data.items():
        if not re.match(r'^[A-Z0-9_]+$', code):
            errors.append(f"Error code '{code}' does not match pattern ^[A-Z0-9_]+$")
        if not isinstance(entry, dict):
            errors.append(f"Error code '{code}' entry must be an object")
            continue
        for req in ["type_uri", "source", "remediation"]:
            if req not in entry or not isinstance(entry[req], str) or not entry[req].strip():
                errors.append(f"Error code '{code}' missing required non-empty string field '{req}'")
        has_status = "status" in entry and isinstance(entry["status"], int) and 100 <= entry["status"] <= 599
        has_statuses = "statuses" in entry and isinstance(entry["statuses"], list) and len(entry["statuses"]) > 0
        if not (has_status or has_statuses):
            errors.append(f"Error code '{code}' must have valid 'status' (100-599) or 'statuses' list")
    return errors

def verify_error_registry() -> List[str]:
    """Validates docs/ERROR_REGISTRY.json against docs/ERROR_REGISTRY.schema.json."""
    reg_path = ROOT / "docs" / "ERROR_REGISTRY.json"
    schema_path = ROOT / "docs" / "ERROR_REGISTRY.schema.json"
    if not reg_path.exists() or not schema_path.exists():
        return [f"Missing registry or schema file: {reg_path} / {schema_path}"]

    try:
        registry_data = json.loads(reg_path.read_text(encoding="utf-8"))
        schema_data = json.loads(schema_path.read_text(encoding="utf-8"))
    except (json.JSONDecodeError, OSError) as err:
        return [f"Failed to read/parse error registry JSON: {err}"]

    try:
        import jsonschema
        validator = jsonschema.Draft7Validator(schema_data)
        return [
            f"Schema violation at '{' -> '.join(str(p) for p in err.path) if err.path else 'root'}': {err.message}"
            for err in validator.iter_errors(registry_data)
        ]
    except ImportError:
        return _validate_registry_entries(registry_data)

def _scan_directory_target(target: Path) -> Tuple[List[Dict[str, Any]], int]:
    """Recursively walks directory scanning source files against the contract."""
    results: List[Dict[str, Any]] = []
    skipped = 0
    for root_dir, dirs, files in os.walk(target):
        dirs[:] = [d for d in dirs if d not in SKIP_DIRS]
        for file in files:
            fpath = Path(root_dir) / file
            if fpath.suffix.lower() in EXTENSIONS:
                results.append(verify_file(fpath))
            else:
                skipped += 1
    return results, skipped

def scan_target_paths(paths: List[str], is_json: bool) -> Tuple[List[Dict[str, Any]], int]:
    """Scans requested paths or recursively walks directories for source files."""
    results: List[Dict[str, Any]] = []
    skipped_count = 0
    for path_str in paths:
        target = Path(path_str).resolve()
        if not is_json:
            print(f"[verify_ai_context] Scanning target: {target}")

        if target.is_file():
            if target.suffix.lower() in EXTENSIONS:
                results.append(verify_file(target))
            else:
                skipped_count += 1
            continue

        dir_results, dir_skipped = _scan_directory_target(target)
        results.extend(dir_results)
        skipped_count += dir_skipped
    return results, skipped_count

def output_scan_results(results: List[Dict[str, Any]], passed: list, failed: list, skipped: int, is_json: bool) -> None:
    """Outputs scan summary and failure lists in CLI or JSON format."""
    if is_json:
        print(json.dumps({
            "summary": {
                "total": len(results),
                "passed": len(passed),
                "failed": len(failed),
                "skipped_non_source": skipped,
                "timestamp": datetime.now(timezone.utc).isoformat(),
            },
            "failures": failed,
        }, indent=2))
        return

    print("\n--- 🛡️ AI Context Contract Report ---")
    print(f"Total Files Scanned: {len(results)} (Skipped non-source: {skipped})")
    print(f"✅ PASSED: {len(passed)}")
    print(f"❌ FAILED: {len(failed)}")
    print("------------------------------------\n")

    if failed:
        print("🚨 DETECTED CONTRACT VIOLATIONS:")
        for f in failed[:20]:
            print(f"- {f['file']}")
            for finding in f["findings"]:
                print(f"    ↳ {finding}")
        if len(failed) > 20:
            print(f"\n... and {len(failed) - 20} more files.")

    print(f"[verify_ai_context] Scanned {len(results)} file(s): {len(passed)} passed, {len(failed)} failed.")

def _append_registry_errors(results: List[Dict[str, Any]], is_json: bool) -> None:
    """Validates docs/ERROR_REGISTRY.json and appends failures if any exist."""
    reg_errors = verify_error_registry()
    if reg_errors:
        results.append({"file": "docs/ERROR_REGISTRY.json", "passed": False, "findings": reg_errors, "meta": {}})
    elif not is_json:
        print("[verify_ai_context] ✓ docs/ERROR_REGISTRY.json matches schema.")

def _filter_with_baseline(results: List[Dict[str, Any]], baseline_path: Optional[str]) -> Tuple[list, list]:
    """Filters results against known baseline findings and returns passed and failed lists."""
    if baseline_path:
        baseline_set = load_baseline(Path(baseline_path).resolve())
        for r in results:
            if not r["passed"]:
                new_findings = [f for f in r["findings"] if (r["file"], f) not in baseline_set]
                r["findings"] = new_findings
                r["passed"] = len(new_findings) == 0
    passed = [r for r in results if r["passed"]]
    failed = [r for r in results if not r["passed"]]
    return passed, failed

def _write_baseline_if_requested(update_baseline_path: str, results: List[Dict[str, Any]], raw_failures: list, is_json: bool) -> None:
    """Writes raw failures out to specified baseline JSON file."""
    base_path = Path(update_baseline_path).resolve()
    base_path.write_text(json.dumps({
        "summary": {"total": len(results), "failed": len(raw_failures), "timestamp": datetime.now(timezone.utc).isoformat()},
        "failures": raw_failures,
    }, indent=2), encoding='utf-8')
    if not is_json:
        print(f"[verify_ai_context] Baseline updated at: {base_path}")

def main() -> int:
    parser = argparse.ArgumentParser(description="Tadpole OS AI Context Contract Auditor")
    parser.add_argument("paths", nargs="*", default=["."], help="Root directories or files to scan")
    parser.add_argument("--json", action="store_true", help="Output results as JSON")
    parser.add_argument("--baseline", type=str, default=None, help="Path to baseline failures JSON file")
    parser.add_argument("--update-baseline", type=str, default=None, help="Path to output current failures as baseline")
    args = parser.parse_args()

    results, skipped_count = scan_target_paths(args.paths, args.json)
    _append_registry_errors(results, args.json)

    raw_failures = [r for r in results if not r["passed"]]
    passed, failed = _filter_with_baseline(results, args.baseline)

    if args.update_baseline:
        _write_baseline_if_requested(args.update_baseline, results, raw_failures, args.json)

    output_scan_results(results, passed, failed, skipped_count, args.json)
    return 0 if not failed else 1

if __name__ == "__main__":
    sys.exit(main())
