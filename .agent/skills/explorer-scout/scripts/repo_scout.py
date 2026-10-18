#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Documentation

### AI Context Alignment
- **Subsystem**: Agent Skills / explorer-scout / repo_scout
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


def scout_local_repo(repo_path: Path) -> dict:
    findings = {
        "manifests": [],
        "languages": set(),
        "has_license": False,
        "entrypoints": []
    }

    manifest_map = {
        "Cargo.toml": "Rust",
        "package.json": "TypeScript/JavaScript",
        "pyproject.toml": "Python",
        "go.mod": "Go"
    }

    for manifest, lang in manifest_map.items():
        if (repo_path / manifest).exists():
            findings["manifests"].append(manifest)
            findings["languages"].add(lang)

    for lic in ["LICENSE", "LICENSE.md", "LICENSE.txt"]:
        if (repo_path / lic).exists():
            findings["has_license"] = True
            break

    entry_candidates = ["src/main.rs", "src/index.ts", "src/main.tsx", "main.py"]
    for entry in entry_candidates:
        if (repo_path / entry).exists():
            findings["entrypoints"].append(entry)

    findings["languages"] = list(findings["languages"])
    return findings


def run_self_tests():
    import tempfile
    with tempfile.TemporaryDirectory() as td:
        td_path = Path(td)
        (td_path / "Cargo.toml").write_text('[package]\nname = "test"', encoding="utf-8")
        (td_path / "LICENSE").write_text('MIT License', encoding="utf-8")
        (td_path / "src").mkdir()
        (td_path / "src" / "main.rs").write_text('fn main() {}', encoding="utf-8")

        res = scout_local_repo(td_path)
        assert "Rust" in res["languages"], f"Failed to detect Rust: {res}"
        assert res["has_license"], "Failed to detect LICENSE"
        assert "src/main.rs" in res["entrypoints"], "Failed to detect entrypoint"
        print("[PASS] repo_scout self-tests passed successfully.")
    return 0


def main():
    parser = argparse.ArgumentParser(description="Repository Scout & Archaeology Tool")
    parser.add_argument("repo_path", nargs="?", default=".", help="Path to repository")
    parser.add_argument("--test", action="store_true", help="Run self-tests")
    parser.add_argument("--json", action="store_true", help="Output pure JSON")

    args = parser.parse_args()
    if args.test:
        sys.exit(run_self_tests())

    path = Path(args.repo_path).resolve()
    res = scout_local_repo(path)

    if args.json:
        print(json.dumps(res, indent=2))
    else:
        print(f"[repo_scout] Scouting analysis for '{path.name}':")
        print(f"  - Languages: {', '.join(res['languages']) if res['languages'] else 'None detected'}")
        print(f"  - Manifests: {', '.join(res['manifests'])}")
        print(f"  - License: {'Found' if res['has_license'] else 'None detected'}")
        print(f"  - Entrypoints: {', '.join(res['entrypoints'])}")


if __name__ == "__main__":
    main()
