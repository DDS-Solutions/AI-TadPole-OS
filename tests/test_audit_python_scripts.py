"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Assist Note
Regression tests for the deterministic Python script auditor. The fixtures verify
safe parsing, missing dependency classification, portability findings, and exclusions.

### 🔍 Debugging & Observability
- **Failure Path**: Audit regressions that hide syntax, dependency, or portability defects.
- **Telemetry Link**: Search `[test_python_script_audit]` in test output.
"""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from execution.audit_python_scripts import audit_workspace, discover_python_files


class PythonScriptAuditTests(unittest.TestCase):
    def write_script(self, root: Path, relative: str, source: str) -> Path:
        path = root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(source, encoding="utf-8")
        return path

    def result_for(self, report: dict[str, object], path: str) -> dict[str, object]:
        results = report["results"]
        self.assertIsInstance(results, list)
        return next(result for result in results if result["path"] == path)

    def test_safe_script_passes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.write_script(
                root,
                "safe.py",
                "import json\n\ndef main():\n    return json.dumps({'ok': True})\n\n"
                "if __name__ == '__main__':\n    main()\n",
            )

            report = audit_workspace(root)

            result = self.result_for(report, "safe.py")
            self.assertEqual(result["status"], "pass")
            self.assertTrue(result["has_main_guard"])

    def test_syntax_error_fails(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.write_script(root, "broken.py", "def broken(:\n    pass\n")

            result = self.result_for(audit_workspace(root), "broken.py")

            self.assertEqual(result["status"], "fail")
            self.assertEqual(result["findings"][0]["code"], "SYNTAX_ERROR")

    def test_unavailable_unconditional_import_fails(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.write_script(root, "missing.py", "import package_that_does_not_exist_8f92ab\n")

            result = self.result_for(audit_workspace(root), "missing.py")

            self.assertEqual(result["status"], "fail")
            self.assertEqual(result["findings"][0]["code"], "MISSING_IMPORT")

    def test_guarded_import_and_absolute_path_require_review(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.write_script(
                root,
                "review.py",
                "try:\n    import package_that_does_not_exist_8f92ab\nexcept ImportError:\n    pass\n"
                "DEFAULT = 'D:/developer/machine/data.db'\n",
            )

            result = self.result_for(audit_workspace(root), "review.py")

            self.assertEqual(result["status"], "review")
            codes = {finding["code"] for finding in result["findings"]}
            self.assertEqual(codes, {"HARDCODED_ABSOLUTE_PATH", "MISSING_IMPORT"})

    def test_generated_and_dependency_directories_are_excluded(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            expected = self.write_script(root, "execution/tool.py", "VALUE = 1\n")
            self.write_script(root, "node_modules/vendor.py", "def broken(:\n")
            self.write_script(root, "target/generated.py", "def broken(:\n")
            self.write_script(root, ".venv/package.py", "def broken(:\n")

            files = discover_python_files(root)

            self.assertEqual(files, [expected])


if __name__ == "__main__":
    unittest.main()


# Metadata: [test_python_script_audit]
