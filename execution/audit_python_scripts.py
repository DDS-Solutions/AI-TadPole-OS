#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / audit_python_scripts
- **Primary Entrypoints**: `Finding`, `ScriptResult`, `ImportRecord`, `ScriptVisitor`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[python_script_audit]`
- **Witness Tests**: none declared
"""

from __future__ import annotations

import argparse
import ast
import hashlib
import importlib.util
import json
import re
import sys
import sysconfig
import tokenize
from dataclasses import asdict, dataclass, field
from pathlib import Path
from typing import Iterable, Sequence


SCHEMA_VERSION = 1
DEFAULT_REPORT = Path("reports/intelligence/python_script_audit.json")
EXCLUDED_DIRS = {
    ".git",
    ".mypy_cache",
    ".pytest_cache",
    ".ruff_cache",
    ".tmp",
    ".venv",
    "__pycache__",
    "build",
    "dist",
    "node_modules",
    "target",
    "venv",
}
STDLIB_MODULES = frozenset(sys.stdlib_module_names)
ABSOLUTE_PATH_PATTERN = re.compile(
    r"^(?:[A-Za-z]:[\\/]|/(?:home|opt|private|tmp|Users|var)(?:/|$))"
)
MAIN_GUARD_PATTERN = ("__name__", "__main__")

NETWORK_CALL_PREFIXES = (
    "httpx.",
    "requests.",
    "urllib.request.urlopen",
    "urllib.request.urlretrieve",
    "websocket.create_connection",
)
DATABASE_CALL_PREFIXES = (
    "psycopg.",
    "psycopg2.",
    "sqlite3.connect",
    "sqlalchemy.create_engine",
)
PROCESS_CALL_PREFIXES = (
    "asyncio.create_subprocess_exec",
    "asyncio.create_subprocess_shell",
    "os.popen",
    "os.system",
    "subprocess.",
)
DESTRUCTIVE_CALLS = {
    "os.remove",
    "os.rmdir",
    "os.unlink",
    "Path.rmdir",
    "Path.unlink",
    "shutil.rmtree",
}
DYNAMIC_EXECUTION_CALLS = {"eval", "exec"}


@dataclass(frozen=True)
class Finding:
    code: str
    severity: str
    line: int
    message: str


@dataclass
class ScriptResult:
    path: str
    sha256: str = ""
    lines: int = 0
    status: str = "pass"
    has_main_guard: bool = False
    cli_parser: bool = False
    imports: list[str] = field(default_factory=list)
    capabilities: list[str] = field(default_factory=list)
    findings: list[Finding] = field(default_factory=list)

    def finalize(self) -> None:
        severities = {finding.severity for finding in self.findings}
        if "error" in severities:
            self.status = "fail"
        elif "warning" in severities:
            self.status = "review"
        else:
            self.status = "pass"


@dataclass(frozen=True)
class ImportRecord:
    module: str
    line: int
    deferred: bool
    guarded: bool


class ScriptVisitor(ast.NodeVisitor):
    """Collects imports and behavior without executing the inspected module."""

    def __init__(self) -> None:
        self.imports: list[ImportRecord] = []
        self.calls: list[tuple[str, ast.Call, bool]] = []
        self.string_literals: list[tuple[str, int]] = []
        self.has_main_guard = False
        self.cli_parser = False
        self._scope_depth = 0
        self._guard_depth = 0

    def visit_FunctionDef(self, node: ast.FunctionDef) -> None:
        self._scope_depth += 1
        self.generic_visit(node)
        self._scope_depth -= 1

    visit_AsyncFunctionDef = visit_FunctionDef

    def visit_ClassDef(self, node: ast.ClassDef) -> None:
        self._scope_depth += 1
        self.generic_visit(node)
        self._scope_depth -= 1

    def visit_Try(self, node: ast.Try) -> None:
        catches_import_failure = any(
            handler.type is None
            or _exception_name(handler.type) in {"Exception", "ImportError", "ModuleNotFoundError"}
            for handler in node.handlers
        )
        if catches_import_failure:
            self._guard_depth += 1
        for statement in node.body:
            self.visit(statement)
        if catches_import_failure:
            self._guard_depth -= 1
        for handler in node.handlers:
            self.visit(handler)
        for statement in (*node.orelse, *node.finalbody):
            self.visit(statement)

    def visit_If(self, node: ast.If) -> None:
        if _is_main_guard(node.test):
            self.has_main_guard = True
            self._scope_depth += 1
            for statement in node.body:
                self.visit(statement)
            self._scope_depth -= 1
            for statement in node.orelse:
                self.visit(statement)
            return

        is_conditional_import = _is_platform_or_type_check(node.test)
        if is_conditional_import:
            self._guard_depth += 1
        self.generic_visit(node)
        if is_conditional_import:
            self._guard_depth -= 1

    def visit_Import(self, node: ast.Import) -> None:
        for alias in node.names:
            self.imports.append(
                ImportRecord(
                    module=alias.name,
                    line=node.lineno,
                    deferred=self._scope_depth > 0,
                    guarded=self._guard_depth > 0,
                )
            )

    def visit_ImportFrom(self, node: ast.ImportFrom) -> None:
        if node.level == 0 and node.module:
            self.imports.append(
                ImportRecord(
                    module=node.module,
                    line=node.lineno,
                    deferred=self._scope_depth > 0,
                    guarded=self._guard_depth > 0,
                )
            )

    def visit_Call(self, node: ast.Call) -> None:
        name = _call_name(node.func)
        if name:
            self.calls.append((name, node, self._scope_depth == 0))
            if name.endswith("ArgumentParser") or name.endswith("parse_args"):
                self.cli_parser = True
        self.generic_visit(node)

    def visit_Constant(self, node: ast.Constant) -> None:
        if isinstance(node.value, str):
            self.string_literals.append((node.value, node.lineno))


def _call_name(node: ast.AST) -> str:
    if isinstance(node, ast.Name):
        return node.id
    if isinstance(node, ast.Attribute):
        prefix = _call_name(node.value)
        return f"{prefix}.{node.attr}" if prefix else node.attr
    return ""


def _exception_name(node: ast.AST) -> str:
    if isinstance(node, ast.Tuple):
        return ",".join(_exception_name(item) for item in node.elts)
    return _call_name(node)


def _is_main_guard(node: ast.AST) -> bool:
    if not isinstance(node, ast.Compare) or len(node.ops) != 1 or len(node.comparators) != 1:
        return False
    left = node.left
    right = node.comparators[0]
    return (
        isinstance(node.ops[0], ast.Eq)
        and isinstance(left, ast.Name)
        and isinstance(right, ast.Constant)
        and (left.id, right.value) == MAIN_GUARD_PATTERN
    )


def _is_platform_or_type_check(node: ast.AST) -> bool:
    names = {_call_name(item) for item in ast.walk(node)}
    return bool(names & {"TYPE_CHECKING", "os.name", "sys.platform"})


def discover_python_files(root: Path) -> list[Path]:
    files: list[Path] = []
    for path in root.rglob("*.py"):
        try:
            relative = path.relative_to(root)
        except ValueError:
            continue
        if any(part in EXCLUDED_DIRS for part in relative.parts):
            continue
        if path.is_file():
            files.append(path)
    return sorted(files, key=lambda item: item.relative_to(root).as_posix().casefold())


def build_local_module_index(root: Path, files: Iterable[Path]) -> set[str]:
    modules: set[str] = set()
    for path in files:
        relative = path.relative_to(root)
        modules.add(path.stem)
        if len(relative.parts) > 1 and relative.parts[0].isidentifier():
            modules.add(relative.parts[0])
        package_parts = [part for part in relative.with_suffix("").parts if part.isidentifier()]
        if package_parts:
            modules.add(".".join(package_parts))
    return modules


def _module_available(module: str, local_modules: set[str]) -> bool:
    top_level = module.split(".", 1)[0]
    if top_level in STDLIB_MODULES or top_level in local_modules:
        return True
    try:
        return importlib.util.find_spec(top_level) is not None
    except (ImportError, ModuleNotFoundError, ValueError):
        return False


def _has_shell_true(call: ast.Call) -> bool:
    return any(
        keyword.arg == "shell"
        and isinstance(keyword.value, ast.Constant)
        and keyword.value.value is True
        for keyword in call.keywords
    )


def _matches_prefix(name: str, prefixes: Sequence[str]) -> bool:
    return any(name == prefix.rstrip(".") or name.startswith(prefix) for prefix in prefixes)


def _add_finding(
    result: ScriptResult,
    code: str,
    severity: str,
    line: int,
    message: str,
) -> None:
    finding = Finding(code=code, severity=severity, line=line, message=message)
    if finding not in result.findings:
        result.findings.append(finding)


def audit_script(path: Path, root: Path, local_modules: set[str]) -> ScriptResult:
    relative = path.relative_to(root).as_posix()
    result = ScriptResult(path=relative)

    try:
        with tokenize.open(path) as source_file:
            source = source_file.read()
    except (OSError, SyntaxError, UnicodeError) as error:
        _add_finding(result, "SOURCE_UNREADABLE", "error", 1, str(error))
        result.finalize()
        return result

    result.sha256 = hashlib.sha256(source.encode("utf-8")).hexdigest()
    result.lines = len(source.splitlines())

    try:
        tree = ast.parse(source, filename=relative)
        compile(tree, relative, "exec", dont_inherit=True)
    except (SyntaxError, ValueError, TypeError) as error:
        line = getattr(error, "lineno", None) or 1
        _add_finding(result, "SYNTAX_ERROR", "error", line, str(error))
        result.finalize()
        return result

    visitor = ScriptVisitor()
    visitor.visit(tree)
    result.has_main_guard = visitor.has_main_guard
    result.cli_parser = visitor.cli_parser
    result.imports = sorted({record.module for record in visitor.imports})

    for record in visitor.imports:
        if _module_available(record.module, local_modules):
            continue
        severity = "warning" if record.deferred or record.guarded else "error"
        qualifier = "optional/deferred" if severity == "warning" else "unconditional"
        _add_finding(
            result,
            "MISSING_IMPORT",
            severity,
            record.line,
            f"{qualifier} import '{record.module}' is unavailable in this environment",
        )

    for literal, line in visitor.string_literals:
        if ABSOLUTE_PATH_PATTERN.match(literal.strip()):
            _add_finding(
                result,
                "HARDCODED_ABSOLUTE_PATH",
                "warning",
                line,
                f"non-portable absolute path: {literal[:120]}",
            )

    capabilities: set[str] = set()
    for name, call, at_module_scope in visitor.calls:
        if _matches_prefix(name, NETWORK_CALL_PREFIXES):
            capabilities.add("network")
            if at_module_scope:
                _add_finding(
                    result,
                    "IMPORT_TIME_EXTERNAL_IO",
                    "warning",
                    call.lineno,
                    f"network call '{name}' can run while importing the module",
                )
        if _matches_prefix(name, DATABASE_CALL_PREFIXES):
            capabilities.add("database")
            if at_module_scope:
                _add_finding(
                    result,
                    "IMPORT_TIME_EXTERNAL_IO",
                    "warning",
                    call.lineno,
                    f"database call '{name}' can run while importing the module",
                )
        if _matches_prefix(name, PROCESS_CALL_PREFIXES):
            capabilities.add("process")
            if at_module_scope:
                _add_finding(
                    result,
                    "IMPORT_TIME_PROCESS",
                    "warning",
                    call.lineno,
                    f"process call '{name}' can run while importing the module",
                )
        if name in DESTRUCTIVE_CALLS or name.endswith((".unlink", ".rmdir")):
            capabilities.add("filesystem-mutation")
        if name in DYNAMIC_EXECUTION_CALLS:
            capabilities.add("dynamic-execution")
            _add_finding(
                result,
                "DYNAMIC_EXECUTION",
                "warning",
                call.lineno,
                f"dynamic execution via '{name}' requires manual security review",
            )
        if name.startswith("subprocess.") and _has_shell_true(call):
            _add_finding(
                result,
                "SUBPROCESS_SHELL_TRUE",
                "warning",
                call.lineno,
                "subprocess call uses shell=True",
            )

    result.capabilities = sorted(capabilities)
    result.findings.sort(key=lambda item: (item.line, item.severity, item.code, item.message))
    result.finalize()
    return result


def audit_workspace(root: Path) -> dict[str, object]:
    root = root.resolve()
    files = discover_python_files(root)
    local_modules = build_local_module_index(root, files)
    results = [audit_script(path, root, local_modules) for path in files]

    counts = {
        "files": len(results),
        "pass": sum(result.status == "pass" for result in results),
        "review": sum(result.status == "review" for result in results),
        "fail": sum(result.status == "fail" for result in results),
        "errors": sum(
            finding.severity == "error" for result in results for finding in result.findings
        ),
        "warnings": sum(
            finding.severity == "warning" for result in results for finding in result.findings
        ),
    }
    return {
        "schema_version": SCHEMA_VERSION,
        "python_version": sys.version.split()[0],
        "stdlib": sysconfig.get_path("stdlib"),
        "root": root.as_posix(),
        "summary": counts,
        "results": [
            {
                **asdict(result),
                "findings": [asdict(finding) for finding in result.findings],
            }
            for result in results
        ],
    }


def write_report(report: dict[str, object], destination: Path) -> None:
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_text(
        json.dumps(report, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )


def print_report(report: dict[str, object], show_passes: bool = False) -> None:
    summary = report["summary"]
    assert isinstance(summary, dict)
    print(
        "[python_script_audit] "
        f"files={summary['files']} pass={summary['pass']} review={summary['review']} "
        f"fail={summary['fail']} errors={summary['errors']} warnings={summary['warnings']}"
    )

    results = report["results"]
    assert isinstance(results, list)
    for raw_result in results:
        assert isinstance(raw_result, dict)
        if raw_result["status"] == "pass" and not show_passes:
            continue
        print(f"[{str(raw_result['status']).upper()}] {raw_result['path']}")
        for finding in raw_result["findings"]:
            print(
                f"  {str(finding['severity']).upper()} "
                f"{finding['code']}:{finding['line']} {finding['message']}"
            )


def parse_args(argv: Sequence[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Safely audit every workspace Python script without importing or executing it."
    )
    parser.add_argument("root", nargs="?", type=Path, default=Path.cwd())
    parser.add_argument("--json", type=Path, default=DEFAULT_REPORT)
    parser.add_argument("--show-passes", action="store_true")
    parser.add_argument(
        "--strict-warnings",
        action="store_true",
        help="Return a failure exit code when warnings require manual review.",
    )
    return parser.parse_args(argv)


def main(argv: Sequence[str] | None = None) -> int:
    args = parse_args(argv)
    report = audit_workspace(args.root)
    report_path = args.json
    if not report_path.is_absolute():
        report_path = args.root.resolve() / report_path
    write_report(report, report_path)
    print_report(report, show_passes=args.show_passes)
    print(f"[python_script_audit] report={report_path.as_posix()}")

    summary = report["summary"]
    assert isinstance(summary, dict)
    if summary["errors"]:
        return 1
    if args.strict_warnings and summary["warnings"]:
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
