#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Documentation

### AI Context Alignment
- **Subsystem**: Agent Skills / app-builder / app_scaffolder
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


KNOWN_STACKS = {
    "tauri-react": ["tauri.conf.json", "package.json", "src/App.tsx"],
    "vite-react": ["package.json", "vite.config.ts", "src/main.tsx"],
    "rust-axum": ["Cargo.toml", "src/main.rs"],
    "python-service": ["pyproject.toml", "requirements.txt"]
}


def detect_project_type(root_path: Path) -> dict:
    detected = []
    for stack, files in KNOWN_STACKS.items():
        match_count = sum(1 for f in files if (root_path / f).exists())
        if match_count >= 1:
            detected.append({"stack": stack, "confidence": match_count / len(files)})

    detected.sort(key=lambda x: x["confidence"], reverse=True)
    return {
        "primary_stack": detected[0]["stack"] if detected else "unknown",
        "detected": detected
    }


def run_self_tests():
    import tempfile
    with tempfile.TemporaryDirectory() as td:
        temp_dir = Path(td)
        (temp_dir / "package.json").write_text('{"name": "test-app"}', encoding="utf-8")
        (temp_dir / "vite.config.ts").write_text('export default {}', encoding="utf-8")

        res = detect_project_type(temp_dir)
        assert res["primary_stack"] == "vite-react", f"Unexpected stack: {res}"
        print("[PASS] app_scaffolder self-tests passed successfully.")
    return 0


def main():
    parser = argparse.ArgumentParser(description="App Builder Scaffolder & Validator")
    parser.add_argument("project_path", nargs="?", default=".", help="Target project root")
    parser.add_argument("--test", action="store_true", help="Run self-tests")
    parser.add_argument("--json", action="store_true", help="Output pure JSON")

    args = parser.parse_args()
    if args.test:
        sys.exit(run_self_tests())

    root_path = Path(args.project_path).resolve()
    result = detect_project_type(root_path)

    if args.json:
        print(json.dumps(result, indent=2))
    else:
        print(f"[app_scaffolder] Project Type: {result['primary_stack']}")
        for d in result["detected"]:
            print(f"  - {d['stack']} (confidence: {d['confidence']*100:.0f}%)")


if __name__ == "__main__":
    main()
