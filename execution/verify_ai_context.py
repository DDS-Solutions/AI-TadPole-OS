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

#!/usr/bin/env python3
"""
🛡️ Tadpole OS: AI Context Contract Verifier
Purpose: Enforces the Structured Header Contract across the Tadpole OS codebase.
Verification Gates:
1. Presence of '### AI Assist Note' or '### AI Context Alignment'
2. Presence of '### 🔍 Debugging & Observability'
3. Presence of '@docs' tag pointing to an existing documentation file
4. Invariant Taxonomy: all invariant bullets must be categorized ([Structural], [Behavioral], or [Advisory: UNVERIFIED])
5. Witness Test Verification: behavioral invariants and witness tests must exist in the workspace test suite
6. Symbol Definition Resolution: backticked entrypoint symbols must resolve to concrete definitions (Python AST / Rust & TS definition heuristics)
7. Telemetry Tag Integrity: declared tags must match log emitters in the code body
"""

import os
import re
import sys
import ast
import json
import argparse
from pathlib import Path
from typing import List, Dict, Any, Optional, Set, Tuple
from datetime import datetime, timezone

# --- Configuration ---
SKIP_DIRS = {
    '.git', 'node_modules', 'dist', 'target', 'build', '__pycache__',
    '.venv', 'venv', '.tmp', 'tmp', 'coverage', '3rdparty', 'wiki',
    'scratch', 'playwright-report', 'test-results'
}
EXTENSIONS = {'.rs', '.ts', '.tsx', '.js', '.jsx', '.py'}
ROOT = Path(__file__).resolve().parent.parent

# Cache of all test function names across the workspace to make witness verification fast
_KNOWN_TESTS: Optional[Set[str]] = None

EMITTERS = [
    # Rust logging & tracing macros, println/eprintln
    r'tracing::(?:info|warn|error|debug|trace)!',
    r'log::(?:info|warn|error|debug|trace)!',
    r'\b(?:println|eprintln)!',
    # Python logging, loguru, print
    r'\b(?:logger|logging)\.(?:info|warning|warn|error|debug|critical|exception)\b',
    r'\bprint\s*\(',
    # JS / TS / Node console and loggers
    r'\bconsole\.(?:log|warn|error|info|debug|trace)\b',
    r'\blogger\.(?:info|warn|error|debug)\b',
]

def strip_comments_and_strings(content: str, file_ext: str) -> str:
    """Strips string literals and comments safely according to language-specific grammar rules."""
    text = content.replace('\r\n', '\n')
    if file_ext == '.rs':
        # 1. Strip Rust multi-hash raw strings r#*""*#
        text = re.sub(r'r(#*)"[\s\S]*?"\1', '""', text)
        # 2. Strip standard strings (allowing multiline strings in Rust)
        text = re.sub(r'"(?:\\.|[^"\\])*?"', '""', text, flags=re.DOTALL)
        # 3. Strip character literals
        text = re.sub(r"'(?:\\.|[^'\\])'", "''", text)
        # 4. Strip block comments
        text = re.sub(r'/\*[\s\S]*?\*/', '', text)
        # 5. Strip line comments
        text = re.sub(r'//.*$', '', text, flags=re.MULTILINE)
        return text
    elif file_ext in {'.ts', '.tsx', '.js', '.jsx'}:
        # 1. Strip block comments first so apostrophes in comments don't span strings
        text = re.sub(r'/\*[\s\S]*?\*/', '', text)
        # 2. Strip line comments
        text = re.sub(r'//.*$', '', text, flags=re.MULTILINE)
        # 3. Strip template literals (which can be multiline)
        text = re.sub(r'`(?:\\.|[^`\\])*?`', '""', text, flags=re.DOTALL)
        # 4. Strip single and double quoted strings (single line only)
        text = re.sub(r'"(?:\\.|[^"\r\n\\])*"', '""', text)
        text = re.sub(r"'(?:\\.|[^'\r\n\\])*'", "''", text)
        return text
    elif file_ext == '.py':
        # 1. Strip docstrings / triple quoted strings
        text = re.sub(r'"""[\s\S]*?"""', '""', text)
        text = re.sub(r"'''[\s\S]*?'''", "''", text)
        # 2. Strip line comments
        text = re.sub(r'#.*$', '', text, flags=re.MULTILINE)
        # 3. Strip single-line standard strings
        text = re.sub(r'"(?:\\.|[^"\r\n\\])*"', '""', text)
        text = re.sub(r"'(?:\\.|[^'\r\n\\])*'", "''", text)
        return text
    return text

def get_workspace_test_symbols() -> Set[str]:
    """Harvests test symbols from the entire repository using attribute-anchored patterns on stripped text."""
    global _KNOWN_TESTS
    if _KNOWN_TESTS is not None:
        return _KNOWN_TESTS

    tests = set()
    for root, dirs, files in os.walk(ROOT):
        dirs[:] = [d for d in dirs if d not in SKIP_DIRS]
        for f in files:
            p = Path(root) / f
            ext = p.suffix.lower()
            if ext in {'.rs', '.ts', '.tsx', '.js', '.jsx', '.py'}:
                try:
                    with open(p, 'r', encoding='utf-8', errors='ignore') as handle:
                        raw_text = handle.read()

                    clean_code = strip_comments_and_strings(raw_text, ext)

                    if ext == '.rs':
                        # 1. Attribute-anchored Rust tests: #[test], #[tokio::test], #[actix_web::test], etc.
                        for m in re.finditer(
                            r'#\[(?:[\w:]*::)?(?:test|tokio::test|actix_web::test|test_case)(?:\([^)]*\))?\](?:\s*#\[[^\]]+\])*\s*(?:pub(?:\([^)]+\))?\s+)?(?:async\s+)?fn\s+([a-zA-Z0-9_]+)',
                            clean_code
                        ):
                            tests.add(m.group(1))

                        # 2. Files located inside dedicated test directories or test files
                        is_test_file = '/tests/' in p.as_posix() or p.name in {'tests.rs', 'test.rs'} or p.name.endswith('_test.rs')
                        if is_test_file:
                            for m in re.finditer(r'(?:async\s+)?fn\s+([a-zA-Z0-9_]+)\s*\(', clean_code):
                                tests.add(m.group(1))
                        else:
                            # 3. Inside explicit `mod tests { ... }` module declarations
                            for mod_match in re.finditer(r'mod\s+tests?\s*\{([^}]+)\}', clean_code):
                                for m in re.finditer(r'(?:async\s+)?fn\s+([a-zA-Z0-9_]+)\s*\(', mod_match.group(1)):
                                    tests.add(m.group(1))

                    elif ext in {'.ts', '.tsx', '.js', '.jsx'}:
                        # JS/TS: Only harvest identifier-compatible test names (no freeform strings with spaces)
                        for m in re.finditer(r'(?:test|it)\s*\(\s*["\'`]([a-zA-Z0-9_]+)["\'`]', clean_code):
                            tests.add(m.group(1))

                    elif ext == '.py':
                        # Python test functions anchored on def test_...
                        for m in re.finditer(r'def\s+(test_[a-zA-Z0-9_]+)\s*\(', clean_code):
                            tests.add(m.group(1))

                except Exception:
                    pass

    _KNOWN_TESTS = tests
    return _KNOWN_TESTS

def verify_symbol_defined_in_file(symbol: str, content: str, file_ext: str) -> bool:
    """Verifies that a backticked symbol is defined in the target file using AST for Python and stripped regex heuristics."""
    clean_sym = symbol.strip('`')
    if not clean_sym:
        return True

    if file_ext == '.py':
        try:
            tree = ast.parse(content)
            for node in ast.walk(tree):
                if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
                    if node.name == clean_sym:
                        return True
                elif isinstance(node, ast.Assign):
                    for target in node.targets:
                        if isinstance(target, ast.Name) and target.id == clean_sym:
                            return True
                elif isinstance(node, ast.AnnAssign):
                    if isinstance(node.target, ast.Name) and node.target.id == clean_sym:
                        return True
            return False
        except Exception:
            # Fallback to stripped text check if syntax error in py
            stripped = strip_comments_and_strings(content, '.py')
            patterns = [
                rf'\bdef\s+{re.escape(clean_sym)}\b',
                rf'\bclass\s+{re.escape(clean_sym)}\b',
                rf'^{re.escape(clean_sym)}\s*=',
            ]
            return any(re.search(p, stripped, re.MULTILINE) for p in patterns)

    elif file_ext == '.rs':
        stripped = strip_comments_and_strings(content, '.rs')
        patterns = [
            rf'\b(?:pub(?:\s*\([^)]+\))?\s+)?(?:async\s+|unsafe\s+|const\s+)*fn\s+{re.escape(clean_sym)}\b',
            rf'\b(?:pub(?:\s*\([^)]+\))?\s+)?struct\s+{re.escape(clean_sym)}\b',
            rf'\b(?:pub(?:\s*\([^)]+\))?\s+)?enum\s+{re.escape(clean_sym)}\b',
            rf'\b(?:pub(?:\s*\([^)]+\))?\s+)?trait\s+{re.escape(clean_sym)}\b',
            rf'\b(?:pub(?:\s*\([^)]+\))?\s+)?(?:const|static|type)\s+{re.escape(clean_sym)}\b',
            rf'\bmacro_rules!\s+{re.escape(clean_sym)}\b',
            rf'\bimpl(?:<[^>]+>)?\s+{re.escape(clean_sym)}\b',
            rf'\b(?:pub(?:\s*\([^)]+\))?\s+)?mod\s+{re.escape(clean_sym)}\b',
        ]
        return any(re.search(p, stripped) for p in patterns)

    elif file_ext in {'.ts', '.tsx', '.js', '.jsx'}:
        stripped = strip_comments_and_strings(content, file_ext)
        patterns = [
            rf'\b(?:export\s+)?(?:default\s+)?(?:async\s+)?function\s+{re.escape(clean_sym)}\b',
            rf'\b(?:export\s+)?(?:const|let|var)\s+{re.escape(clean_sym)}(?:\s*:[^=]+)?\s*=',
            rf'\b(?:export\s+)?class\s+{re.escape(clean_sym)}\b',
            rf'\b(?:export\s+)?interface\s+{re.escape(clean_sym)}\b',
            rf'\b(?:export\s+)?type\s+{re.escape(clean_sym)}(?:\s*<[^>]+>)?\s*=',
            rf'\b(?:export\s+)?enum\s+{re.escape(clean_sym)}\b',
            rf'\b{re.escape(clean_sym)}\s*\([^)]*\)\s*\{{',
            rf'\b(?:async\s+)?{re.escape(clean_sym)}\s*\(.*?\)\s*(?::\s*[^=]+)?\s*=>',
        ]
        return any(re.search(p, stripped) for p in patterns)

    return True

def extract_code_body(content: str, file_ext: str) -> str:
    """Returns the code content excluding the leading AI Context header/docstring."""
    norm = content.replace('\r\n', '\n')
    if file_ext == '.py':
        m = re.match(r'^(?:\s*#[^\n]*\n)*\s*(?:"""[\s\S]*?"""|\'\'\'[\s\S]*?\'\'\')', norm)
        if m:
            return norm[m.end():]
    elif file_ext == '.rs':
        lines = norm.split('\n')
        idx = 0
        while idx < len(lines) and (
            lines[idx].strip().startswith('//') or
            lines[idx].strip().startswith('/*') or
            lines[idx].strip().startswith('*') or
            not lines[idx].strip()
        ):
            idx += 1
        return '\n'.join(lines[idx:])
    elif file_ext in {'.ts', '.tsx', '.js', '.jsx'}:
        m = re.match(r'^(?:\s*//[^\n]*\n|\s*/\*[\s\S]*?\*/\s*)+', norm)
        if m:
            return norm[m.end():]
    return norm

def tag_is_emitted(tag: str, code_content: str) -> bool:
    """Verifies that a declared telemetry tag is emitted in bracketed format alongside a recognized logger in code."""
    tag_clean = tag.strip('`[] ')
    if not tag_clean or tag_clean.lower() in {"none", "none declared", "n/a", "no", "false"}:
        return True

    tag_fmt = f"[{tag_clean}]"
    for line in code_content.splitlines():
        if tag_fmt in line and any(re.search(e, line) for e in EMITTERS):
            return True
    return False

def extract_header_block(content: str, file_ext: str = "") -> str:
    """Extracts only the top header block (docstring or leading comments) to prevent code from contaminating metadata."""
    norm = content.replace('\r\n', '\n')
    if file_ext == '.py' or norm.strip().startswith('"""') or norm.strip().startswith("'''"):
        m = re.match(r'^(?:\s*#[^\n]*\n)*\s*(?:"""[\s\S]*?"""|\'\'\'[\s\S]*?\'\'\')', norm)
        if m:
            return m.group(0)
    elif file_ext == '.rs':
        lines = norm.split('\n')
        header_lines = []
        for line in lines:
            s = line.strip()
            if s.startswith('//') or s.startswith('/*') or s.startswith('*') or s.startswith('*/') or not s:
                header_lines.append(line)
            else:
                break
        return '\n'.join(header_lines)
    elif file_ext in {'.ts', '.tsx', '.js', '.jsx'}:
        lines = norm.split('\n')
        header_lines = []
        for line in lines:
            s = line.strip()
            if s.startswith('//') or s.startswith('/*') or s.startswith('*') or s.startswith('*/') or not s:
                header_lines.append(line)
            else:
                break
        return '\n'.join(header_lines)

    # Generic fallback: search up to first non-comment non-empty line or first 120 lines
    lines = norm.split('\n')
    return '\n'.join(lines[:120])

def extract_metadata(content: str, file_ext: str = "") -> Dict[str, Any]:
    """Extracts structured metadata and verification targets from the header comments or docstrings."""
    header_raw = extract_header_block(content, file_ext)
    cleaned_lines = []
    for line in header_raw.split('\n'):
        l = re.sub(r'^\s*(?://[!/]?|\*(?!/)|#(?!##))\s?', '', line)
        cleaned_lines.append(l)
    clean_text = '\n'.join(cleaned_lines)

    res = {
        "has_note": False,
        "has_debugging": False,
        "has_docs": False,
        "has_invariants": False,
        "telemetry_tags": [],
        "docs_link": None,
        "symbols": [],
        "invariants": [],
        "witness_tests": [],
        "local_errors": []
    }

    # 1. Check for AI Note or Context Alignment header
    if re.search(r'#{1,4}\s+AI\s+(?:Assist\s+Note|Context\s+Alignment)', clean_text, re.IGNORECASE):
        res["has_note"] = True

    # 2. Check for Debugging section
    if re.search(r'#{1,4}\s+.*?\s*Debugging\s+&\s+Observability', clean_text, re.IGNORECASE):
        res["has_debugging"] = True

    # 3. Check for Invariants section
    if re.search(r'#{1,4}\s+.*?\s*Invariants\s+&\s+Non-Negotiables', clean_text, re.IGNORECASE):
        res["has_invariants"] = True

    # 4. Check for @docs tag
    docs_match = re.search(r'@docs\s+([A-Za-z0-9_:\-]+)', clean_text)
    if docs_match:
        res["has_docs"] = True
        res["docs_link"] = docs_match.group(1)

    # 5. Extract Telemetry Tags
    tele_section = re.search(r'-\s+\*\*Telemetry (?:Targets|Link)\*\*:\s*([^\n]+)', clean_text)
    if tele_section:
        line_val = tele_section.group(1).strip()
        if not re.match(r'^(?:none\b|n/a\b|none declared\b)', line_val, re.IGNORECASE):
            bracket_tags = re.findall(r'\[([a-zA-Z0-9_:\-]+)\]', line_val)
            backtick_tags = re.findall(r'`([a-zA-Z0-9_:\-]+)`', line_val)
            for tag in bracket_tags + backtick_tags:
                tag = tag.strip('`[] ')
                if tag and tag.lower() not in {"none", "none declared", "n/a"} and tag not in res["telemetry_tags"]:
                    res["telemetry_tags"].append(tag)
    
    # Also support inline Telemetry Link patterns (e.g. Search `[tag]` in logs)
    tele_matches = re.finditer(
        r'Search (?:for |`)\[?([a-zA-Z0-9_:\-]+)\]?`? in (?:system|audit|test|log)',
        clean_text,
        re.IGNORECASE
    )
    for tm in tele_matches:
        tag = tm.group(1).strip('`[] ')
        if tag and tag.lower() not in {"none", "none declared", "n/a"} and tag not in res["telemetry_tags"]:
            res["telemetry_tags"].append(tag)

    # 6. Extract Entrypoint / Execution Symbols
    sym_section_matches = re.finditer(
        r'-\s+\*\*(?:Primary Entrypoints|Execution Symbols|Core Symbols)\*\*:\s*([^\n]+)',
        clean_text
    )
    for sm in sym_section_matches:
        symbols_line = sm.group(1)
        for sym in re.findall(r'`([a-zA-Z0-9_]+)`', symbols_line):
            if sym not in res["symbols"]:
                res["symbols"].append(sym)

    # 7. Extract Invariant taxonomy bullets
    invariants_match = re.search(
        r'#{1,4}\s+.*?\s*Invariants\s+&\s+Non-Negotiables\s*\n(.*?)(?=\n#{1,4}|\Z)',
        clean_text,
        re.DOTALL
    )
    if invariants_match:
        inv_body = invariants_match.group(1)
        raw_bullets = re.split(r'\n(?=-\s+)', inv_body)
        for b in raw_bullets:
            b_str = b.strip()
            if not b_str.startswith('-'):
                continue
            tag_match = re.search(r'-\s+`?\[(Structural|Behavioral|Advisory:\s*UNVERIFIED)\]`?', b_str)
            tag = tag_match.group(1) if tag_match else None
            enf_match = re.search(r'enforced_by:\s*`?([a-zA-Z0-9_]+)`?', b_str)
            witness = enf_match.group(1) if enf_match else None
            res["invariants"].append({
                "tag": tag,
                "text": b_str.split('\n')[0],
                "witness": witness
            })

    # 8. Extract Witness Tests
    witness_match = re.search(r'-\s+\*\*Witness Tests\*\*:\s*([^\n]+)', clean_text)
    if witness_match:
        for test in re.findall(r'`([a-zA-Z0-9_]+)`', witness_match.group(1)):
            if test not in res["witness_tests"]:
                res["witness_tests"].append(test)

    return res

def verify_file(file_path: Path) -> Dict[str, Any]:
    """Runs all 7 AI Context Contract verification gates on a single source file."""
    try:
        with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
            content = f.read()

        file_ext = file_path.suffix.lower()
        meta = extract_metadata(content, file_ext)
        findings = []

        # 1. Structural Header Presence
        if not meta["has_note"]:
            findings.append("Missing '### AI Assist Note' or '### AI Context Alignment'")

        if not meta["has_debugging"]:
            findings.append("Missing '### 🔍 Debugging & Observability'")

        # 2. Documentation Link Integrity (@docs presence and multi-segment target check)
        if not meta["has_docs"]:
            findings.append("Missing '@docs' documentation link")
        elif meta["docs_link"]:
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
                findings.append(f"Broken @docs link: Documentation file for '{meta['docs_link']}' not found")

        # 3. Telemetry Tag Integrity (Emitter verification in code body)
        code_body = extract_code_body(content, file_ext)
        for tag in meta["telemetry_tags"]:
            if not tag_is_emitted(tag, code_body):
                findings.append(f"Telemetry Tag '{tag}' defined in header but missing from executable log emitters in file body")

        # 4. Symbol Definition Resolution (AST / heuristic check)
        for sym in meta["symbols"]:
            if not verify_symbol_defined_in_file(sym, content, file_ext):
                findings.append(f"Declared Symbol `{sym}` is not defined as an AST/code symbol in this file")

        # 5. Invariant Taxonomy & Required Behavioral Witnesses
        all_known_tests = get_workspace_test_symbols()
        for inv in meta["invariants"]:
            if not inv["tag"]:
                findings.append(f"Uncategorized invariant: '{inv['text'][:60]}...' (must be marked [Structural], [Behavioral], or [Advisory: UNVERIFIED])")
            elif inv["tag"] == "Behavioral" and not inv["witness"]:
                findings.append(f"[Behavioral] invariant missing explicit 'enforced_by: `test_name`': '{inv['text'][:60]}...'")

        # 6. Top-Level Witness Tests Resolution & Decoupled Cross-Listing Validation
        enforced_witnesses = {inv["witness"] for inv in meta["invariants"] if inv.get("witness")}
        declared_witnesses = set(meta["witness_tests"])

        # Check 6A: Declared witness tests must exist in repository test suite
        for w_test in meta["witness_tests"]:
            if w_test.lower() not in {"none", "none declared", "n/a"} and w_test not in all_known_tests:
                findings.append(f"Witness Test `{w_test}` not found in repository test suite")

        # Check 6B: Any invariant with an enforced_by link must be cross-listed in the header's 'Witness Tests' section
        for enf_w in enforced_witnesses:
            if enf_w and enf_w not in declared_witnesses:
                findings.append(f"Witness test `{enf_w}` is linked in an invariant but omitted from the 'Witness Tests' field")

        # Check 6C: Any invariant with an enforced_by link must exist in the workspace test suite
        for enf_w in enforced_witnesses:
            if enf_w and enf_w not in all_known_tests:
                findings.append(f"Invariant witness test `{enf_w}` not found in test suite")

        # Deduplicate any identical findings while preserving order
        findings = list(dict.fromkeys(findings))

        try:
            rel_path = file_path.relative_to(ROOT).as_posix()
        except ValueError:
            rel_path = file_path.as_posix()

        return {
            "file": rel_path,
            "passed": len(findings) == 0,
            "findings": findings,
            "meta": meta
        }
    except Exception as e:
        return {
            "file": file_path.as_posix() if hasattr(file_path, 'as_posix') else str(file_path).replace('\\', '/'),
            "passed": False,
            "findings": [f"Error processing file: {str(e)}"]
        }

def load_baseline(baseline_file: Path) -> Set[Tuple[str, str]]:
    """Loads a baseline JSON file containing known (file, finding) tuples."""
    if not baseline_file.exists():
        return set()
    try:
        with open(baseline_file, 'r', encoding='utf-8') as f:
            data = json.load(f)
        baseline_set = set()
        for item in data.get("failures", []):
            file_name = item.get("file", "").replace('\\', '/')
            for finding in item.get("findings", []):
                baseline_set.add((file_name, finding))
        return baseline_set
    except Exception:
        return set()

def main():
    if sys.platform == "win32":
        import io
        sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
        sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8')

    parser = argparse.ArgumentParser(description="Tadpole OS AI Context Contract Auditor")
    parser.add_argument("paths", nargs="*", default=["."], help="Root directories or files to scan")
    parser.add_argument("--json", action="store_true", help="Output results as JSON")
    parser.add_argument("--baseline", type=str, default=None, help="Path to baseline failures JSON file")
    parser.add_argument("--update-baseline", type=str, default=None, help="Path to output current failures as a baseline JSON file")
    args = parser.parse_args()

    results = []
    skipped_count = 0
    baseline_set = set()
    if args.baseline:
        baseline_set = load_baseline(Path(args.baseline).resolve())

    for path_str in args.paths:
        scan_target = Path(path_str).resolve()
        if not args.json:
            print(f"[verify_ai_context] Scanning target: {scan_target}")

        if scan_target.is_file():
            if scan_target.suffix.lower() in EXTENSIONS:
                results.append(verify_file(scan_target))
            else:
                skipped_count += 1
                if not args.json:
                    print(f"[verify_ai_context] Skipping non-source file: {scan_target.name}")
        else:
            for root, dirs, files in os.walk(scan_target):
                dirs[:] = [d for d in dirs if d not in SKIP_DIRS]
                for file in files:
                    file_path = Path(root) / file
                    if file_path.suffix.lower() in EXTENSIONS:
                        results.append(verify_file(file_path))
                    else:
                        skipped_count += 1

    # Capture raw failures BEFORE baseline filtering so --update-baseline never drops existing entries
    raw_failures = [r for r in results if not r["passed"]]

    # If baseline is loaded, filter out known baseline findings for CI status determination
    if baseline_set:
        for r in results:
            if not r["passed"]:
                new_findings = [f for f in r["findings"] if (r["file"], f) not in baseline_set]
                r["findings"] = new_findings
                r["passed"] = len(new_findings) == 0

    passed = [r for r in results if r["passed"]]
    failed = [r for r in results if not r["passed"]]

    if args.update_baseline:
        base_path = Path(args.update_baseline).resolve()
        with open(base_path, 'w', encoding='utf-8') as f:
            json.dump({
                "summary": {
                    "total": len(results),
                    "failed": len(raw_failures),
                    "timestamp": datetime.now(timezone.utc).isoformat()
                },
                "failures": raw_failures
            }, f, indent=2)
        if not args.json:
            print(f"[verify_ai_context] Baseline updated at: {base_path}")

    if args.json:
        print(json.dumps({
            "summary": {
                "total": len(results),
                "passed": len(passed),
                "failed": len(failed),
                "skipped_non_source": skipped_count,
                "timestamp": datetime.now(timezone.utc).isoformat()
            },
            "failures": failed
        }, indent=2))
    else:
        print(f"\n--- 🛡️ AI Context Contract Report ---")
        print(f"Total Files Scanned: {len(results)} (Skipped non-source: {skipped_count})")
        print(f"✅ PASSED: {len(passed)}")
        print(f"❌ FAILED: {len(failed)}")
        print(f"------------------------------------\n")

        if failed:
            print("🚨 DETECTED CONTRACT VIOLATIONS:")
            for f in failed[:20]:
                print(f"- {f['file']}")
                for finding in f["findings"]:
                    print(f"    ↳ {finding}")

            if len(failed) > 20:
                print(f"\n... and {len(failed) - 20} more files.")

        print(f"[verify_ai_context] Scanned {len(results)} file(s): {len(passed)} passed, {len(failed)} failed.")
    sys.exit(0 if not failed else 1)

if __name__ == "__main__":
    main()
