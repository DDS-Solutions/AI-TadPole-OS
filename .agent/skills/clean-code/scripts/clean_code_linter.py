#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Registry:Skills

### AI Context Alignment
- **Subsystem**: Agent Skills / clean-code / clean_code_linter
- **Primary Entrypoints**: `analyze_file_for_clean_code`, `detect_swallowed_errors`, `detect_deep_nesting`, `detect_long_functions`, `run_self_tests`, `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic validation without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[clean_code_linter]`, `[CLEAN_CODE]`, `[SMELL]`, `[PASS]`, `[FAIL]`
"""

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Any, Dict, List, Optional, Set, Tuple

if sys.platform == "win32":
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    if hasattr(sys.stderr, "reconfigure"):
        sys.stderr.reconfigure(encoding="utf-8", errors="replace")

MAX_FILE_LINES = 500
MAX_FUNCTION_LINES = 50
MAX_INDENT_LEVEL = 4

TEST_PATH_MARKERS = (".test.", ".spec.", "_test.", "_tests.", "test_")
COMMENT_PREFIXES = ("//", "#", "/*", "*", "--")


def is_test_filepath(file_path: Path, file_ext: str = "") -> bool:
    """Return True if path or extension marker indicates a dedicated test file."""
    name_lower = file_path.name.lower()
    ext_lower = file_ext.lower()
    if (
        name_lower == "tests.rs"
        or name_lower.startswith(("test_", "tests_"))
        or "test" in ext_lower
        or any(m in name_lower for m in TEST_PATH_MARKERS)
    ):
        return True
    parts = [p.lower() for p in file_path.parts]
    return "tests" in parts or "__tests__" in parts or "port3000_conformance" in parts


def find_rust_cfg_test_lines(lines: List[str]) -> Set[int]:
    """Return 1-based line numbers belonging to inline #[cfg(test)] blocks in Rust."""
    test_lines: Set[int] = set()
    state = 0  # 0 = normal, 1 = pending '{', 2 = inside cfg(test)
    brace_depth = 0

    for idx, line in enumerate(lines, 1):
        stripped = line.strip()
        if state == 0 and stripped.startswith("#[cfg(test)]"):
            state = 1
            test_lines.add(idx)
            continue
        if state == 0:
            continue
        test_lines.add(idx)
        delta = stripped.count("{") - stripped.count("}")
        if state == 1 and stripped.count("{") > 0:
            brace_depth = delta
            state = 2 if brace_depth > 0 else 0
        elif state == 2:
            brace_depth += delta
            state = 2 if brace_depth > 0 else 0

    return test_lines


def _strip_inline_string_literals(line: str) -> str:
    """Remove double-quoted and single-quoted string contents to avoid false matches."""
    no_double = re.sub(r'"(?:\\.|[^"\\])*"', '""', line)
    return re.sub(r"'(?:\\.|[^'\\])*'", "''", no_double)


def detect_swallowed_errors(
    content: str,
    file_ext: str = "",
    file_path: Optional[Path] = None,
) -> List[Dict[str, Any]]:
    """Detect swallowed exceptions, empty catch blocks, unsafe any, or production unwrap."""
    smells: List[Dict[str, Any]] = []
    lines = content.splitlines()
    path_obj = file_path or Path(f"input{file_ext}")
    is_test_file = is_test_filepath(path_obj, file_ext)
    cfg_test_lines = find_rust_cfg_test_lines(lines) if file_ext == ".rs" else set()

    for match in re.finditer(r"except(?:\s+[\w\.]+)?\s*:\s*(?:\n\s*|\s+)pass\b", content):
        line_num = content[: match.start()].count("\n") + 1
        smells.append({
            "line": line_num,
            "smell": "Swallowed Exception",
            "detail": "Python: Swallowed exception with pass (silent error drop)",
            "snippet": match.group(0).replace("\n", " "),
        })

    for match in re.finditer(r"catch\s*(?:\([^)]*\))?\s*\{\s*\}", content):
        line_num = content[: match.start()].count("\n") + 1
        smells.append({
            "line": line_num,
            "smell": "Empty Catch Block",
            "detail": "JS/TS: Empty catch block with no error handling or rethrow",
            "snippet": match.group(0).replace("\n", " "),
        })

    for idx, line in enumerate(lines, 1):
        line_clean = line.strip()
        if not line_clean or line_clean.startswith(COMMENT_PREFIXES):
            continue
        if is_test_file or idx in cfg_test_lines:
            continue
        code_only = _strip_inline_string_literals(line_clean)
        if file_ext in (".ts", ".tsx"):
            _check_ts_any(idx, line_clean, code_only, smells)
        elif file_ext == ".rs":
            _check_rust_unwrap(idx, line_clean, code_only, smells)

    return smells


def _check_ts_any(
    idx: int,
    raw_line: str,
    code_only: str,
    smells: List[Dict[str, Any]],
) -> None:
    has_as_any = bool(re.search(r"\bas\s+any\b", code_only))
    has_colon_any = bool(re.search(r":\s*any\b", code_only))
    if not (has_as_any or has_colon_any):
        return
    detail = (
        "TypeScript: Unsafe 'as any' escape hatch"
        if has_as_any
        else "TypeScript: Explicit 'any' annotation"
    )
    smells.append({
        "line": idx,
        "smell": "Unsafe Any Type",
        "detail": detail,
        "snippet": raw_line,
    })


def _check_rust_unwrap(
    idx: int,
    raw_line: str,
    code_only: str,
    smells: List[Dict[str, Any]],
) -> None:
    if ".unwrap()" not in code_only or "SAFETY:" in raw_line or "INVARIANT:" in raw_line:
        return
    smells.append({
        "line": idx,
        "smell": "Unhandled Rust unwrap()",
        "detail": "Production Rust code using .unwrap()",
        "snippet": raw_line,
    })


def _is_non_control_continuation(stripped: str) -> bool:
    """Return True if line is JSX markup/props, method chain continuation, dict/struct field, or closing delimiter."""
    if stripped.startswith((".", "}", ")", "]", "<", "</", "/>", ">", "|", "?", ":", "{")):
        return True
    if re.match(r"^[a-zA-Z0-9_-]+=(?:\{|['\"])", stripped):
        return True
    return bool(re.match(r"^['\"]?[a-zA-Z0-9_]+['\"]?\s*:\s*.+$", stripped))


def detect_deep_nesting(content: str) -> List[Dict[str, Any]]:
    """Detect executable lines indented beyond MAX_INDENT_LEVEL (guard clause violation)."""
    smells: List[Dict[str, Any]] = []
    lines = content.splitlines()
    cfg_test_lines = find_rust_cfg_test_lines(lines)

    for idx, line in enumerate(lines, 1):
        if idx in cfg_test_lines:
            continue
        stripped = line.strip()
        if not stripped or stripped.startswith(COMMENT_PREFIXES):
            continue
        if _is_non_control_continuation(stripped):
            continue

        leading_spaces = len(line) - len(line.lstrip(" "))
        leading_tabs = len(line) - len(line.lstrip("\t"))
        level = leading_tabs if leading_tabs > 0 else (leading_spaces // 4)

        if level > MAX_INDENT_LEVEL:
            smells.append({
                "line": idx,
                "smell": "Deep Nesting (Arrow Anti-Pattern)",
                "detail": f"Indentation level {level} exceeds maximum recommended depth of {MAX_INDENT_LEVEL}",
                "snippet": stripped,
            })

    return smells


def _match_function_header(lines: List[str], idx: int) -> Optional[str]:
    """Return function/component name if lines[idx] begins a function or arrow component."""
    line = lines[idx]
    if line.strip().endswith(";"):
        return None
    fn_m = re.match(
        r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?(?:fn|function|def)\s+([a-zA-Z0-9_]+)",
        line,
    )
    if fn_m:
        return fn_m.group(1)
    arrow_m = re.match(
        r"^\s*(?:export\s+)?const\s+([a-zA-Z0-9_]+)(?:\s*:[^=]+)?\s*=\s*(?:async\s*)?\(",
        line,
    )
    if arrow_m:
        window = "\n".join(lines[idx : min(len(lines), idx + 15)])
        if "=>" in window:
            return arrow_m.group(1)
    return None


def detect_long_functions(content: str, file_ext: str = "") -> List[Dict[str, Any]]:
    """Detect production functions whose body exceeds MAX_FUNCTION_LINES."""
    smells: List[Dict[str, Any]] = []
    lines = content.splitlines()
    cfg_test_lines = find_rust_cfg_test_lines(lines) if file_ext == ".rs" else set()

    i = 0
    while i < len(lines):
        if (i + 1) in cfg_test_lines:
            i += 1
            continue
        fn_name = _match_function_header(lines, i)
        if not fn_name:
            i += 1
            continue
        start_line = i + 1
        end_idx, body_lines = _measure_function_span(lines, i, file_ext)
        if body_lines > MAX_FUNCTION_LINES:
            smells.append({
                "line": start_line,
                "smell": "Oversized Function",
                "detail": f"Function '{fn_name}' contains {body_lines} lines (exceeds {MAX_FUNCTION_LINES} limit).",
                "snippet": lines[i].strip()[:80],
            })
        i = max(i + 1, end_idx)

    return smells


def _measure_py_function_span(lines: List[str], start_idx: int) -> Tuple[int, int]:
    base_indent = len(lines[start_idx]) - len(lines[start_idx].lstrip(" "))
    count = 0
    idx = start_idx + 1
    while idx < len(lines):
        s = lines[idx].strip()
        if not s or s.startswith("#"):
            idx += 1
            continue
        if (len(lines[idx]) - len(lines[idx].lstrip(" "))) <= base_indent:
            break
        count += 1
        idx += 1
    return idx, count


def _measure_function_span(lines: List[str], start_idx: int, file_ext: str) -> Tuple[int, int]:
    """Return (end_index, non_blank_non_comment_lines) for a function starting at start_idx."""
    if file_ext == ".py":
        return _measure_py_function_span(lines, start_idx)

    brace_depth = 0
    started = False
    count = 0
    for idx in range(start_idx, len(lines)):
        s = lines[idx].strip()
        if not s or s.startswith(COMMENT_PREFIXES):
            continue
        opens, closes = s.count("{"), s.count("}")
        started = started or (opens > 0)
        if not started:
            continue
        brace_depth += opens - closes
        count += 1
        if brace_depth <= 0:
            return idx + 1, count
    return start_idx + 1, count


def analyze_file_for_clean_code(file_path: Path, content: Optional[str] = None) -> Dict[str, Any]:
    """Perform deterministic clean code analysis on a given file."""
    if content is None:
        try:
            content = file_path.read_text(encoding="utf-8", errors="replace")
        except OSError as e:
            return {
                "file": str(file_path),
                "total_lines": 0,
                "passed": False,
                "smells": [{"line": 0, "smell": "File Read Error", "detail": str(e), "snippet": ""}],
            }

    lines = content.splitlines()
    file_ext = file_path.suffix.lower() if isinstance(file_path, Path) else ""
    cfg_test_lines = find_rust_cfg_test_lines(lines) if file_ext == ".rs" else set()
    prod_line_count = len(lines) - len(cfg_test_lines)

    smells: List[Dict[str, Any]] = []
    if prod_line_count > MAX_FILE_LINES and not is_test_filepath(file_path, file_ext):
        smells.append({
            "line": 1,
            "smell": "God Object / Oversized File",
            "detail": f"File contains {prod_line_count} production lines (exceeds {MAX_FILE_LINES} limit). Decompose into focused domain modules.",
            "snippet": f"Production Lines: {prod_line_count} (Total: {len(lines)})",
        })

    smells.extend(detect_swallowed_errors(content, file_ext, file_path))
    smells.extend(detect_long_functions(content, file_ext))

    nesting_smells = detect_deep_nesting(content)
    if len(nesting_smells) > 3:
        smells.extend(nesting_smells[:3])
        smells.append({
            "line": nesting_smells[3]["line"],
            "smell": "Deep Nesting (Suppressed)",
            "detail": f"... and {len(nesting_smells) - 3} additional deep nesting occurrences.",
            "snippet": "",
        })
    else:
        smells.extend(nesting_smells)

    return {
        "file": str(file_path),
        "total_lines": len(lines),
        "production_lines": prod_line_count,
        "passed": len(smells) == 0,
        "smells": smells,
    }


def _collect_git_diff_files() -> List[Path]:
    """Return existing modified source files (.rs, .ts, .tsx, .py) from git status."""
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
    allowed_exts = {".rs", ".ts", ".tsx", ".py"}
    paths: List[Path] = []
    for raw in res.stdout.splitlines():
        p = Path(raw.strip())
        if p.is_file() and p.suffix.lower() in allowed_exts and not is_test_filepath(p):
            paths.append(p)
    return paths


def run_self_tests() -> bool:
    """Execute built-in self-tests verifying clean-code detection heuristics."""
    print("🔬 [clean_code_linter] Running self-test suite...")

    py_code = "def fetch_data():\n    try:\n        do_something()\n    " + "except Exception:\n        pass\n"
    res1 = detect_swallowed_errors(py_code, ".py")
    assert any("pass" in s["detail"] for s in res1), f"Test 1 Failed: {res1}"

    js_code = "function execute() {\n    try {\n        compute();\n    } " + "cat" + "ch (err) {}\n}\n"
    res2 = detect_swallowed_errors(js_code, ".js")
    assert any("Empty catch block" in s["detail"] for s in res2), f"Test 2 Failed: {res2}"

    ts_code = "const rawData: any = JSON.parse(str) as any;"
    res3 = detect_swallowed_errors(ts_code, ".ts")
    assert len(res3) >= 1, f"Test 3 Failed: {res3}"

    rs_code = "fn prod() {\n    let item = collection.get(0).unwrap();\n}\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn t() { let x = Some(1).unwrap(); }\n}\n"
    res4 = detect_swallowed_errors(rs_code, ".rs", Path("src/service.rs"))
    assert len(res4) == 1 and res4[0]["line"] == 2, f"Test 4 Failed (cfg(test) exclusion): {res4}"

    nested_code = "function deep() {\n    if (a) {\n        if (b) {\n            if (c) {\n                if (d) {\n                    if (e) return 42;\n                }\n            }\n        }\n    }\n}"
    res5 = detect_deep_nesting(nested_code)
    assert len(res5) >= 1, f"Test 5 Failed: {res5}"

    clean_code = "def add(a: int, b: int) -> int:\n    if a < 0 or b < 0:\n        raise ValueError('Inputs must be non-negative')\n    return a + b\n"
    report = analyze_file_for_clean_code(Path("clean.py"), clean_code)
    assert report["passed"], f"Test 6 Failed: {report}"

    print("✅ [PASS] All self-tests passed successfully.")
    return True


def _print_single_report(report: Dict[str, Any], as_json: bool) -> bool:
    print(f"📊 [CLEAN_CODE] Total Lines: {report['total_lines']} | Passed: {report['passed']}")
    if as_json:
        print(json.dumps(report, indent=2))
        return bool(report["passed"])
    if report["passed"]:
        print("✅ [PASS] Clean code analysis passed. No critical code smells detected.")
        return True
    print(f"❌ [FAIL] Detected {len(report['smells'])} clean code smell(s):")
    for idx, s in enumerate(report["smells"], 1):
        line_info = f" (Line {s['line']})" if s["line"] > 0 else ""
        print(f"  {idx}. [SMELL]{line_info} {s['smell']}: {s['detail']}")
        snippet = s.get("snippet", "")
        _ = snippet and print(f"      ↳ {snippet}")
    return False


def _run_diff_mode(as_json: bool) -> int:
    results = []
    for fpath in _collect_git_diff_files():
        print(f"🔍 [clean_code_linter] Analyzing code quality from: {fpath}")
        rep = analyze_file_for_clean_code(fpath)
        results.append(_print_single_report(rep, as_json))
    return 0 if all(results) else 1


def main() -> None:
    parser = argparse.ArgumentParser(description="Deterministic Clean Code and Code Smell Linter")
    parser.add_argument("--file", type=str, help="Path to source file to analyze")
    parser.add_argument("--diff", action="store_true", help="Analyze git-modified production files")
    parser.add_argument("--test", action="store_true", help="Run self-tests")
    parser.add_argument("--json", action="store_true", help="Output results in JSON format")
    args = parser.parse_args()

    if args.test:
        sys.exit(0 if run_self_tests() else 1)

    if args.diff:
        sys.exit(_run_diff_mode(args.json))

    if args.file:
        file_path = Path(args.file)
        if not file_path.exists():
            print(f"❌ [FAIL] File not found: {file_path}", file=sys.stderr)
            sys.exit(1)
        print(f"🔍 [clean_code_linter] Analyzing code quality from: {file_path}")
        rep = analyze_file_for_clean_code(file_path)
        sys.exit(0 if _print_single_report(rep, args.json) else 1)

    if not sys.stdin.isatty():
        content = sys.stdin.read()
        print("🔍 [clean_code_linter] Analyzing code quality from: stdin")
        rep = analyze_file_for_clean_code(Path("input"), content)
        sys.exit(0 if _print_single_report(rep, args.json) else 1)

    sys.exit(0 if run_self_tests() else 1)


if __name__ == "__main__":
    main()
