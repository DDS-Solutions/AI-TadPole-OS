#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Documentation

### AI Context Alignment
- **Subsystem**: Agent Skills / game-development / game_engine_linter
- **Primary Entrypoints**: `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
- **Witness Tests**: none declared
"""

import sys
import json
import argparse
from pathlib import Path

if hasattr(sys.stdout, 'reconfigure'):
    sys.stdout.reconfigure(encoding='utf-8')


def audit_game_manifest(project_path: Path) -> dict:
    findings = {"engines_detected": [], "audio_assets": 0, "graphic_assets": 0}
    if (project_path / "godot.project").exists():
        findings["engines_detected"].append("Godot")
    if (project_path / "Assets").exists():
        findings["engines_detected"].append("Unity/Custom")
    return findings


def run_self_tests():
    import tempfile
    with tempfile.TemporaryDirectory() as td:
        td_path = Path(td)
        (td_path / "godot.project").write_text("[config]", encoding="utf-8")
        res = audit_game_manifest(td_path)
        assert "Godot" in res["engines_detected"]
        print("[PASS] game_engine_linter self-tests passed successfully.")
    return 0


def main():
    parser = argparse.ArgumentParser(description="Game Asset & Engine Linter")
    parser.add_argument("project_path", nargs="?", default=".", help="Project root directory")
    parser.add_argument("--test", action="store_true", help="Run self-tests")

    args = parser.parse_args()
    if args.test:
        sys.exit(run_self_tests())

    res = audit_game_manifest(Path(args.project_path))
    print(f"[game_engine_linter] Detected: {res['engines_detected']}")


if __name__ == "__main__":
    main()
