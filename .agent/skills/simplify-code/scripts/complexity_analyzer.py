#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Documentation

### AI Context Alignment
- **Subsystem**: Agent Skills / simplify-code / complexity_analyzer
- **Primary Entrypoints**: `analyze_file_complexity`, `run_self_tests`, `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
"""

import argparse
import json
import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any, Dict, List, Optional, Set, Tuple

if sys.platform == "win32":
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    if hasattr(sys.stderr, "reconfigure"):
        sys.stderr.reconfigure(encoding="utf-8", errors="replace")

MAX_NESTING_DEPTH = 4
MAX_FUNCTION_LINES = 60
MAX_CYCLOMATIC_BRANCHES = 12
MAX_DOMAIN_PARAMS = 6

TEST_PATH_MARKERS = (".test.", ".spec.", "_test.", "_tests.", "test_")
COMMENT_PREFIXES = ("//", "#", "/*", "*", "--")
AXUM_EXTRACTOR_PREFIXES = ("State(", "Path(", "Query(", "Json(", "Extension(", "Headers(")


def is_test_filepath(file_path: Path) -> bool:
    """Return True if path indicates a dedicated test file."""
    name_lower = file_path.name.lower()
    if (
        name_lower == "tests.rs"
        or name_lower.startswith(("test_", "tests_"))
        or any(m in name_lower for m in TEST_PATH_MARKERS)
    ):
        return True
    parts = [p.lower() for p in file_path.parts]
    return "tests" in parts or "__tests__" in parts or "port3000_conformance" in parts


def _find_rust_cfg_test_lines(lines: List[str]) -> Set[int]:
    """Return 1-based line numbers inside #[cfg(test)] blocks."""
    test_lines: Set[int] = set()
    state = 0
    depth = 0
    for idx, line in enumerate(lines, 1):
        s = line.strip()
        if state == 0 and s.startswith("#[cfg(test)]"):
            state = 1
            test_lines.add(idx)
            continue
        if state == 0:
            continue
        test_lines.add(idx)
        delta = s.count("{") - s.count("}")
        if state == 1 and s.count("{") > 0:
            depth = delta
            state = 2 if depth > 0 else 0
        elif state == 2:
            depth += delta
            state = 2 if depth > 0 else 0
    return test_lines


def _is_non_control_line(stripped: str) -> bool:
    """Ignore JSX tags/props, closing brackets, method chains, and dict/struct field lines."""
    if stripped.startswith((".", "}", ")", "]", "<", "</", "/>", ">", "|", "?", ":", "{")):
        return True
    if re.match(r"^[a-zA-Z0-9_-]+=(?:\{|['\"])", stripped):
        return True
    return bool(re.match(r"^['\"]?[a-zA-Z0-9_]+['\"]?\s*:\s*.+$", stripped))


def _scan_nesting(lines: List[str], filename: str, skip_lines: Set[int]) -> List[Dict[str, Any]]:
    findings: List[Dict[str, Any]] = []
    for idx, line in enumerate(lines, start=1):
        if idx in skip_lines:
            continue
        stripped = line.lstrip()
        if not stripped or stripped.startswith(COMMENT_PREFIXES) or _is_non_control_line(stripped):
            continue
        leading_spaces = len(line) - len(stripped)
        indent_level = line.count("\t") + (leading_spaces // 4)
        if indent_level > MAX_NESTING_DEPTH:
            findings.append({
                "file": filename,
                "line": idx,
                "metric": "nesting",
                "rule": f"Excessive nesting depth ({indent_level} levels > {MAX_NESTING_DEPTH} threshold). Flatten with guard clauses.",
                "snippet": stripped[:80],
            })
    return findings


def _strip_nested_brackets(s: str) -> str:
    """Remove nested <...>, (...), [...], and {...} groups so top-level parameter commas can be counted."""
    out: List[str] = []
    depth = 0
    for ch in s:
        if ch in "<([{":
            depth += 1
        elif ch in ">)]}" and depth > 0:
            depth -= 1
            if depth == 0 and ch == "}":
                out.append("props_obj")
        elif depth == 0:
            out.append(ch)
    return "".join(out)


def _count_domain_params(param_str: str) -> int:
    """Count function parameters excluding self and framework extractors."""
    cleaned = _strip_nested_brackets(param_str)
    if not cleaned.strip():
        return 0
    raw_parts = [p.strip() for p in cleaned.split(",") if p.strip()]
    count = 0
    for p in raw_parts:
        if p in ("self", "&self", "&mut self", "cls"):
            continue
        if any(ext in p for ext in AXUM_EXTRACTOR_PREFIXES) or p.startswith("_admin"):
            continue
        count += 1
    return count


def _count_branches(func_lines: List[str]) -> int:
    """Estimate cyclomatic complexity (base 1 + decision branches)."""
    branches = 1
    branch_re = re.compile(r"\b(?:if|else\s+if|elif|for|while|loop|case|catch)\b|&&|\|\|")
    for ln in func_lines:
        s = ln.strip()
        if not s or s.startswith(COMMENT_PREFIXES):
            continue
        cleaned = re.sub(r'"(?:\\.|[^"\\])*"|\'(?:\\.|[^\'\\])*\'', '""', s)
        branches += len(branch_re.findall(cleaned))
    return branches


def _extract_py_func_end(lines: List[str], start_idx: int) -> Tuple[int, List[str]]:
    base_indent = len(lines[start_idx]) - len(lines[start_idx].lstrip(" "))
    body: List[str] = []
    idx = start_idx + 1
    while idx < len(lines):
        s = lines[idx].strip()
        if not s or s.startswith("#"):
            idx += 1
            continue
        if (len(lines[idx]) - len(lines[idx].lstrip(" "))) <= base_indent:
            break
        body.append(lines[idx])
        idx += 1
    return idx, body


def _extract_func_end(lines: List[str], start_idx: int, ext: str) -> Tuple[int, List[str]]:
    if ext == ".py":
        return _extract_py_func_end(lines, start_idx)

    depth = 0
    started = False
    body: List[str] = []
    for idx in range(start_idx, len(lines)):
        s = lines[idx].strip()
        if not s or s.startswith(COMMENT_PREFIXES):
            continue
        opens, closes = s.count("{"), s.count("}")
        started = started or (opens > 0)
        if not started:
            continue
        depth += opens - closes
        body.append(lines[idx])
        if depth <= 0:
            return idx + 1, body
    return start_idx + 1, body


def _match_fn_signature(lines: List[str], idx: int) -> Optional[Tuple[str, str]]:
    """Match standard or multi-line arrow function signature and extract (name, param_str)."""
    line = lines[idx]
    if line.strip().endswith(";"):
        return None
    fn_m = re.match(
        r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?(?:fn|function|def)\s+([a-zA-Z0-9_]+)\s*\(",
        line,
    )
    arrow_m = re.match(
        r"^\s*(?:export\s+)?const\s+([a-zA-Z0-9_]+)(?:\s*:[^=]+)?\s*=\s*(?:async\s*)?\(",
        line,
    )
    if not fn_m and not arrow_m:
        return None
    window = " ".join(ln.strip() for ln in lines[idx : min(len(lines), idx + 15)])
    if arrow_m and "=>" not in window:
        return None
    fn_name = fn_m.group(1) if fn_m else arrow_m.group(1)  # type: ignore[union-attr]
    open_pos = window.find("(")
    if open_pos == -1:
        return fn_name, ""
    param_chars: List[str] = []
    depth = 0
    for ch in window[open_pos + 1 :]:
        if ch == "(":
            depth += 1
        elif ch == ")":
            if depth == 0:
                break
            depth -= 1
        param_chars.append(ch)
    return fn_name, "".join(param_chars)


def _scan_functions(
    lines: List[str],
    filename: str,
    ext: str,
    skip_lines: Set[int],
) -> List[Dict[str, Any]]:
    findings: List[Dict[str, Any]] = []
    i = 0
    while i < len(lines):
        if (i + 1) in skip_lines:
            i += 1
            continue
        sig = _match_fn_signature(lines, i)
        if not sig:
            i += 1
            continue
        fn_name, param_head = sig
        start_line = i + 1
        end_idx, body = _extract_func_end(lines, i, ext)
        findings.extend(_evaluate_function_metrics(fn_name, start_line, param_head, body, filename))
        i = max(i + 1, end_idx)
    return findings


def _evaluate_function_metrics(
    fn_name: str,
    start_line: int,
    param_head: str,
    body: List[str],
    filename: str,
) -> List[Dict[str, Any]]:
    findings: List[Dict[str, Any]] = []
    if len(body) > MAX_FUNCTION_LINES:
        findings.append({
            "file": filename,
            "line": start_line,
            "metric": "function_length",
            "rule": f"Oversized function '{fn_name}' ({len(body)} lines > {MAX_FUNCTION_LINES} threshold). Extract sub-functions.",
            "snippet": fn_name,
        })
    branches = _count_branches(body)
    if branches >= MAX_CYCLOMATIC_BRANCHES:
        findings.append({
            "file": filename,
            "line": start_line,
            "metric": "cyclomatic_complexity",
            "rule": f"High cyclomatic complexity in '{fn_name}' ({branches} branches >= {MAX_CYCLOMATIC_BRANCHES} threshold). Simplify branching.",
            "snippet": fn_name,
        })
    params = _count_domain_params(param_head)
    if params >= MAX_DOMAIN_PARAMS:
        findings.append({
            "file": filename,
            "line": start_line,
            "metric": "parameter_count",
            "rule": f"Excessive parameter count in '{fn_name}' ({params} params >= {MAX_DOMAIN_PARAMS} threshold). Bundle into options struct.",
            "snippet": fn_name,
        })
    return findings


def analyze_file_complexity(filepath: Path) -> List[Dict[str, Any]]:
    """Scan a source file for excessive nesting depth, function length, cyclomatic complexity, and parameter count."""
    try:
        content = filepath.read_text(encoding="utf-8", errors="ignore")
    except OSError as e:
        return [{"line": 0, "rule": f"Cannot read file: {e}"}]

    lines = content.splitlines()
    ext = filepath.suffix.lower()
    skip_lines = _find_rust_cfg_test_lines(lines) if ext == ".rs" else set()

    findings = _scan_nesting(lines, filepath.name, skip_lines)
    findings.extend(_scan_functions(lines, filepath.name, ext, skip_lines))
    return findings


def run_self_tests() -> bool:
    """Run built-in self-tests for nesting, function length, cyclomatic complexity, and parameter count."""
    nested_code = (
        "function process(a) {\n"
        "    if (a) {\n"
        "        if (a.b) {\n"
        "            if (a.b.c) {\n"
        "                if (a.b.c.d) {\n"
        "                    return a.b.c.d.val;\n"
        "                }\n"
        "            }\n"
        "        }\n"
        "    }\n"
        "}\n"
    )
    with tempfile.NamedTemporaryFile("w", suffix=".js", delete=False) as f:
        f.write(nested_code)
        tmp_name = f.name
    try:
        findings = analyze_file_complexity(Path(tmp_name))
        assert len(findings) >= 1, "Should detect nesting depth >= 4"
    finally:
        os.remove(tmp_name)

    flat_code = (
        "function process(a) {\n"
        "    if (!a?.b?.c?.d) return null;\n"
        "    return a.b.c.d.val;\n"
        "}\n"
    )
    with tempfile.NamedTemporaryFile("w", suffix=".js", delete=False) as f:
        f.write(flat_code)
        tmp_name2 = f.name
    try:
        findings2 = analyze_file_complexity(Path(tmp_name2))
        assert len(findings2) == 0, f"Expected 0 findings for flat code, got {len(findings2)}"
    finally:
        os.remove(tmp_name2)

    many_params_code = "fn too_many(a: i32, b: i32, c: i32, d: i32, e: i32, f: i32) {\n    let x = a + b;\n}\n"
    with tempfile.NamedTemporaryFile("w", suffix=".rs", delete=False) as f:
        f.write(many_params_code)
        tmp_name3 = f.name
    try:
        findings3 = analyze_file_complexity(Path(tmp_name3))
        assert any(item.get("metric") == "parameter_count" for item in findings3), f"Expected parameter_count finding: {findings3}"
    finally:
        os.remove(tmp_name3)

    print("[PASS] complexity_analyzer self-tests passed successfully.")
    return True


def _collect_git_diff_files() -> List[Path]:
    try:
        res = subprocess.run(
            ["git", "diff", "--name-only", "HEAD"],
            capture_output=True,
            text=True,
            timeout=5,
        )
    except (OSError, subprocess.SubprocessError):
        return []
    if res.returncode != 0:
        return []
    exts = {".rs", ".ts", ".tsx", ".py"}
    return [
        Path(r.strip())
        for r in res.stdout.splitlines()
        if Path(r.strip()).is_file()
        and Path(r.strip()).suffix.lower() in exts
        and not is_test_filepath(Path(r.strip()))
    ]


def _report_file(p: Path, as_json: bool) -> bool:
    findings = analyze_file_complexity(p)
    if as_json:
        print(json.dumps({"file": str(p), "passed": len(findings) == 0, "findings": findings}, indent=2))
        return len(findings) == 0
    if findings:
        print(f"[FAIL] Found {len(findings)} complexity violations in {p.name}:")
        for f in findings[:10]:
            print(f"  Line {f['line']}: {f['rule']}")
        return False
    print(f"[PASS] {p.name} satisfies nesting complexity standards.")
    return True


def _run_diff_mode(as_json: bool) -> int:
    results = [_report_file(fpath, as_json) for fpath in _collect_git_diff_files()]
    return 0 if all(results) else 1


def main() -> None:
    parser = argparse.ArgumentParser(description="Code Complexity & Nesting Analyzer")
    parser.add_argument("--test", action="store_true", help="Run internal self-tests")
    parser.add_argument("--file", type=str, help="Path to source file to analyze")
    parser.add_argument("--diff", action="store_true", help="Analyze git-modified source files")
    parser.add_argument("--json", action="store_true", help="Output JSON report")
    args = parser.parse_args()

    if args.test:
        sys.exit(0 if run_self_tests() else 1)

    if args.diff:
        sys.exit(_run_diff_mode(args.json))

    if args.file:
        p = Path(args.file)
        if not p.exists():
            print(f"[ERROR] File not found: {p}")
            sys.exit(1)
        sys.exit(0 if _report_file(p, args.json) else 1)

    parser.print_help()
    sys.exit(0)


if __name__ == "__main__":
    main()
