"""@docs ARCHITECTURE:DesktopShell

### AI Assist Note
Exercise the packaging rejection paths using command doubles, without a Mac SDK.
These tests do not assert that a native application builds or launches.

### 🔍 Debugging & Observability
Subprocess stdout/stderr and the captured command log explain failed gates.
"""

import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]
BASH = (
    str(Path(os.environ.get("ProgramFiles", "C:/Program Files")) / "Git/bin/bash.exe")
    if os.name == "nt"
    else shutil.which("bash")
)


@unittest.skipUnless(BASH and Path(BASH).exists(), "bash is required")
class MacOSBundleGates(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="macos-gates-")
        self.addCleanup(self.temp.cleanup)
        self.fixture = Path(self.temp.name)
        self.bin = self.fixture / "bin"
        self.bin.mkdir()
        self.bundle = self.fixture / "bundle with spaces"
        self.app = self.bundle / "macos/AI-Tadpole-OS.app"
        self.executables = self.app / "Contents/MacOS"
        self.executables.mkdir(parents=True)
        for name in ("tadpole-os", "server-rs"):
            binary = self.executables / name
            binary.write_text("#!/bin/sh\nexit 0\n", encoding="utf-8")
            binary.chmod(0o755)
        self.dmg = self.bundle / "dmg/test.dmg"
        self.dmg.parent.mkdir()
        self.dmg.write_bytes(b"fixture")
        self.log = self.fixture / "commands.log"
        for command, body in {
            "uname": 'echo "${TEST_KERNEL:-Darwin}"',
            "lipo": 'echo "lipo $*" >> "$TEST_LOG"\nexit "${TEST_LIPO_EXIT:-0}"',
            "codesign": 'echo "codesign $*" >> "$TEST_LOG"\nexit "${TEST_SIGN_EXIT:-0}"',
            "plutil": 'echo "${TEST_MAIN_BINARY:-tadpole-os}"',
        }.items():
            path = self.bin / command
            path.write_text(f"#!/bin/sh\n{body}\n", encoding="utf-8")
            path.chmod(0o755)
        self.env = {
            **os.environ,
            "TEST_LOG": self.log.as_posix(),
        }

    def run_gate(self, **overrides):
        mock_path = self.bin.as_posix()
        if os.name == "nt":
            mock_path = "/" + mock_path[0].lower() + mock_path[2:]
        return subprocess.run(
            [
                BASH, "--noprofile", "--norc", "-c",
                'export PATH="$1:$PATH"; shift; source "$@"',
                "macos-gate-test", mock_path,
                (ROOT / "scripts/verify-macos-bundle.sh").as_posix(), self.bundle.as_posix(),
            ],
            env={**self.env, **overrides},
            capture_output=True,
            text=True,
            timeout=20,
            check=False,
        )

    def test_accepts_complete_bundle_and_checks_both_executables(self):
        result = self.run_gate()
        self.assertEqual(result.returncode, 0, result.stderr)
        log = self.log.read_text(encoding="utf-8")
        self.assertEqual(log.count("lipo -verify_arch arm64 x86_64"), 2)
        self.assertIn("MacOS/server-rs", log)
        self.assertIn("MacOS/tadpole-os", log)
        self.assertEqual(log.count("codesign --verify"), 3)

    def test_rejects_missing_disk_image(self):
        self.dmg.unlink()
        result = self.run_gate()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("No .dmg", result.stderr)

    def test_uses_the_bundle_declared_main_executable(self):
        (self.executables / "tadpole-os").rename(self.executables / "Custom Main")
        result = self.run_gate(TEST_MAIN_BINARY="Custom Main")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("MacOS/Custom Main", self.log.read_text(encoding="utf-8"))

    def test_rejects_empty_disk_image(self):
        self.dmg.write_bytes(b"")
        result = self.run_gate()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("Empty disk image", result.stderr)

    def test_rejects_missing_bundled_sidecar(self):
        (self.executables / "server-rs").unlink()
        result = self.run_gate()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("Missing executable", result.stderr)

    def test_rejects_missing_architecture_slice(self):
        result = self.run_gate(TEST_LIPO_EXIT="1")
        self.assertNotEqual(result.returncode, 0)
        self.assertNotIn("[OK]", result.stdout)
        self.assertIn("lipo -verify_arch", self.log.read_text(encoding="utf-8"))

    def test_rejects_invalid_signature(self):
        result = self.run_gate(TEST_SIGN_EXIT="1")
        self.assertNotEqual(result.returncode, 0)
        self.assertNotIn("[OK]", result.stdout)
        self.assertIn("codesign --verify", self.log.read_text(encoding="utf-8"))

    def test_rejects_non_macos_host(self):
        result = self.run_gate(TEST_KERNEL="Linux")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("requires macOS", result.stderr)

    def test_build_stages_both_sidecars_before_tauri_invocation(self):
        project = self.fixture / "project with spaces"
        (project / "scripts").mkdir(parents=True)
        (project / "server-rs").mkdir()
        for name in ("build-macos-universal.sh", "verify-macos-bundle.sh"):
            shutil.copyfile(ROOT / "scripts" / name, project / "scripts" / name)
        shutil.copytree(self.bundle, project / "src-tauri/target/universal-apple-darwin/release/bundle")
        commands = {
            "rustup": "exit 0",
            "node": "echo 11.0",
            "npm": "exit 0",
            "cargo": '''test "$MACOSX_DEPLOYMENT_TARGET" = "11.0"
while [[ $# -gt 0 ]]; do
  if [[ "$1" == "--target" ]]; then target="$2"; break; fi
  shift
done
mkdir -p "target/$target/release"
printf '#!/bin/sh\\nexit 0\\n' > "target/$target/release/server-rs"
chmod +x "target/$target/release/server-rs"''',
            "lipo": '''if [[ "$1" == "-create" ]]; then
  printf '#!/bin/sh\\nexit 0\\n' > "$3"
  chmod +x "$3"
fi''',
            "npx": '''set -e
for target in aarch64-apple-darwin x86_64-apple-darwin universal-apple-darwin; do
  test -x "src-tauri/bin/server-rs-$target"
done
test "$*" = "--no-install tauri build --target universal-apple-darwin -- --locked"
echo "tauri invoked with all sidecars" >> "$TEST_LOG"''',
        }
        for command, body in commands.items():
            path = self.bin / command
            path.write_text(f"#!/usr/bin/env bash\nset -euo pipefail\n{body}\n", encoding="utf-8")
            path.chmod(0o755)
        mock_path = self.bin.as_posix()
        if os.name == "nt":
            mock_path = "/" + mock_path[0].lower() + mock_path[2:]
        result = subprocess.run(
            [BASH, "--noprofile", "--norc", "-c", 'export PATH="$1:$PATH"; source "$2"',
             "macos-build-test", mock_path, (project / "scripts/build-macos-universal.sh").as_posix()],
            env=self.env, capture_output=True, text=True, timeout=30, check=False,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("tauri invoked with all sidecars", self.log.read_text(encoding="utf-8"))


if __name__ == "__main__":
    unittest.main()
