"""
@docs ARCHITECTURE:Core, ARCHITECTURE:Sandboxing

### AI Context Alignment
- **Subsystem**: Sovereign Engine / Plane 1 Tooling Unit Tests
- **Primary Entrypoints**: `TestPlane1FastGate`, `TestParityGuardCli`, `TestPlane1Worktree`

### ⚠️ Non-Negotiables
- `[Structural]` Pure deterministic unit tests without mutating git history or live binaries.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
"""

import sys
import unittest
from pathlib import Path
from unittest.mock import patch, MagicMock

EXEC_DIR = Path(__file__).resolve().parent.parent
if str(EXEC_DIR) not in sys.path:
    sys.path.insert(0, str(EXEC_DIR))

import plane1_fast_gate
import plane1_worktree
import parity_guard


class TestPlane1FastGate(unittest.TestCase):
    def test_extract_failure_snippet_surfaces_early_fail_above_trailing_oks(self):
        lines = ["[FAIL] [DOCS-PARITY] API_REFERENCE.md is out of sync with openapi.yaml."]
        lines.extend(f"[OK] [WIKI-META] page_{i}.md has coverage marker" for i in range(35))
        snippet = plane1_fast_gate.extract_failure_snippet("\n".join(lines), "", max_lines=20)
        self.assertIn("[FAIL] [DOCS-PARITY]", snippet)

    def test_detect_warnings_catches_rust_and_ts_warnings(self):
        warnings = plane1_fast_gate.detect_warnings(
            "src/foo.ts: warning TS6133: unused var\nall good",
            "warning: unused variable `x`",
        )
        self.assertEqual(len(warnings), 2)

    def test_run_gate_strict_fails_when_warnings_emitted(self):
        fake_proc = MagicMock(returncode=0, stdout="", stderr="warning: unused import")
        with patch.object(plane1_fast_gate.subprocess, "run", return_value=fake_proc):
            res = plane1_fast_gate.run_gate("Rust Fast Check", ["cargo", "check"], strict=True)
        self.assertFalse(res["success"])
        self.assertEqual(res["warnings_count"], 1)
        self.assertIn("[STRICT]", res["snippet"])


class TestParityGuardCli(unittest.TestCase):
    def test_parse_cli_args_supports_env_flag_and_positional(self):
        root, fix = parity_guard.parse_cli_args(["parity_guard.py"], {"FIX": "1"})
        self.assertEqual(root, ".")
        self.assertTrue(fix)

        root2, fix2 = parity_guard.parse_cli_args(["parity_guard.py", "--fix"], {})
        self.assertEqual(root2, ".")
        self.assertTrue(fix2)

        root3, fix3 = parity_guard.parse_cli_args(["parity_guard.py", "FIX=1", "sub"], {})
        self.assertEqual(root3, "sub")
        self.assertTrue(fix3)


class TestPlane1Worktree(unittest.TestCase):
    def test_build_sandbox_env_defaults_to_shared_plane1_cargo_target(self):
        env = plane1_worktree.build_sandbox_env({"PATH": "/usr/bin"})
        self.assertEqual(env["CARGO_TARGET_DIR"], str(plane1_worktree.SHARED_CARGO_TARGET))

    def test_build_sandbox_env_preserves_caller_override(self):
        env = plane1_worktree.build_sandbox_env({"CARGO_TARGET_DIR": "/custom/target"})
        self.assertEqual(env["CARGO_TARGET_DIR"], "/custom/target")

    def test_get_worktree_path_defaults_to_reusable_active_slot(self):
        self.assertEqual(plane1_worktree.get_worktree_path("my-task"), plane1_worktree.ACTIVE_WORKTREE_PATH)
        self.assertEqual(
            plane1_worktree.get_worktree_path("my-task", isolated=True),
            plane1_worktree.WORKTREES_ROOT / "my-task",
        )
        with self.assertRaises(ValueError):
            plane1_worktree.validate_slug("_active")

    def test_parse_porcelain_paths_handles_modified_and_renamed_entries(self):
        paths = plane1_worktree.parse_porcelain_paths(" M README.md\nR  old.ts -> src/new.ts\n")
        self.assertEqual(paths, {"README.md", "src/new.ts"})

    def test_merge_worktree_refuses_dirty_root(self):
        with patch.object(plane1_worktree, "resolve_worktree_path", return_value=Path("/tmp/wt")), \
             patch.object(plane1_worktree, "check_root_clean", return_value=" M src/App.tsx"):
            rc = plane1_worktree.merge_worktree("sample-task", skip_test=True)
        self.assertEqual(rc, 1)

    def test_merge_worktree_stash_root_refuses_overlapping_files(self):
        with patch.object(plane1_worktree, "resolve_worktree_path", return_value=Path("/tmp/wt")), \
             patch.object(plane1_worktree, "check_root_clean", return_value=" M README.md"), \
             patch.object(plane1_worktree, "_collect_sandbox_changed_paths", return_value={"README.md"}):
            rc = plane1_worktree.merge_worktree("sample-task", skip_test=True, stash_root=True)
        self.assertEqual(rc, 1)

    def test_merge_worktree_stash_root_stashes_and_pops_non_overlapping_files(self):
        ok_proc = MagicMock(returncode=0, stdout="", stderr="")
        with patch.object(plane1_worktree, "resolve_worktree_path", return_value=Path("/tmp/wt")), \
             patch.object(plane1_worktree, "check_root_clean", return_value=" M README.md"), \
             patch.object(plane1_worktree, "_collect_sandbox_changed_paths", return_value={"src/a.ts"}), \
             patch.object(plane1_worktree, "run_git", return_value=ok_proc) as mock_git, \
             patch.object(plane1_worktree, "_execute_merge_steps", return_value=0):
            rc = plane1_worktree.merge_worktree("sample-task", skip_test=True, stash_root=True)
        self.assertEqual(rc, 0)
        git_cmds = [call.args[0] for call in mock_git.call_args_list]
        self.assertIn(["stash", "push", "-m", "plane1-merge-sample-task", "--", "README.md"], git_cmds)
        self.assertIn(["stash", "pop"], git_cmds)

    def test_api_docs_parity_ignores_clean_git_checkout_mtime_drift(self):
        import lib.parity_validators as validators
        with patch.object(validators.subprocess, "run") as mock_run:
            mock_run.side_effect = [
                MagicMock(returncode=0, stdout=""),
                MagicMock(returncode=0, stdout="1700000100\n"),
                MagicMock(returncode=0, stdout="1700000200\n"),
            ]
            self.assertFalse(validators._is_api_docs_genuinely_stale(Path(".")))

        with patch.object(validators.subprocess, "run") as mock_run:
            mock_run.return_value = MagicMock(returncode=0, stdout=" M docs/openapi.yaml\n")
            self.assertTrue(validators._is_api_docs_genuinely_stale(Path(".")))


if __name__ == "__main__":
    unittest.main()
