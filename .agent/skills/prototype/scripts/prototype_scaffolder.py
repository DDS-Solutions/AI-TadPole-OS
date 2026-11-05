#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Documentation

### AI Context Alignment
- **Subsystem**: Agent Skills / prototype / prototype_scaffolder
- **Primary Entrypoints**: `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
- **Witness Tests**: none declared
"""

import sys
import os
import re
import argparse
from pathlib import Path
from typing import Dict, List, Any

# Windows console encoding fix
if sys.platform == "win32":
    if hasattr(sys.stdout, 'reconfigure'):
        sys.stdout.reconfigure(encoding='utf-8', errors='replace')
    if hasattr(sys.stderr, 'reconfigure'):
        sys.stderr.reconfigure(encoding='utf-8', errors='replace')

HTML_SPIKE_TEMPLATE = """<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Spike: {title}</title>
  <style>
    :root {{ --bg: #0f172a; --fg: #f8fafc; --card: #1e293b; --accent: #38bdf8; }}
    body {{ background: var(--bg); color: var(--fg); font-family: system-ui, sans-serif; padding: 2rem; margin: 0; }}
    .card {{ background: var(--card); border: 1px solid #334155; border-radius: 8px; padding: 1.5rem; max-width: 520px; margin: 0 auto; box-shadow: 0 4px 6px -1px rgb(0 0 0 / 0.1); }}
    h1 {{ font-size: 1.25rem; margin-top: 0; color: var(--accent); }}
    button {{ background: var(--accent); color: #0f172a; border: none; padding: 0.6rem 1.2rem; border-radius: 4px; font-weight: 600; cursor: pointer; }}
    button:hover {{ filter: brightness(1.1); }}
    .output {{ margin-top: 1rem; padding: 0.75rem; background: #0b1120; border-radius: 4px; font-family: monospace; font-size: 0.9rem; }}
  </style>
</head>
<body>
  <div class="card" id="spike-root">
    <h1>Spike: {title}</h1>
    <p>{description}</p>
    <button id="action-btn">Test Interaction</button>
    <div class="output" id="log-output">Status: Ready</div>
  </div>

  <script>
    const btn = document.getElementById('action-btn');
    const log = document.getElementById('log-output');
    btn.addEventListener('click', () => {{
      log.textContent = `Event fired at: ${{new Date().toLocaleTimeString()}}`;
    }});
  </script>
</body>
</html>
"""

def scaffold_spike(name: str, target_dir: Path, description: str = "Spike interaction prototype") -> Path:
    """Scaffold a minimal self-contained HTML spike in target directory."""
    spike_dir = target_dir / name
    spike_dir.mkdir(parents=True, exist_ok=True)
    index_file = spike_dir / "index.html"

    rendered = HTML_SPIKE_TEMPLATE.format(
        title=name.replace("-", " ").title(),
        description=description
    )
    index_file.write_text(rendered, encoding="utf-8")
    return index_file

def validate_spike(spike_dir: Path) -> Dict[str, Any]:
    """Validate that a prototype directory has a valid runnable entry point."""
    index_file = spike_dir / "index.html"
    if not index_file.exists():
        return {"valid": False, "error": f"Missing index.html in {spike_dir}"}

    content = index_file.read_text(encoding="utf-8", errors="ignore")
    has_root = "spike-root" in content or "id=" in content
    has_script = "<script>" in content or "<script " in content

    return {
        "valid": has_root and has_script,
        "entry": str(index_file),
        "has_root": has_root,
        "has_script": has_script
    }

def run_self_tests() -> bool:
    """Run built-in self-tests."""
    import tempfile

    with tempfile.TemporaryDirectory() as tmp:
        tmp_path = Path(tmp)
        index_path = scaffold_spike("test-widget", tmp_path, "Testing spike generation")
        assert index_path.exists(), "Scaffolded file does not exist"

        validation = validate_spike(tmp_path / "test-widget")
        assert validation["valid"], f"Validation failed: {validation}"

    print("[PASS] prototype_scaffolder self-tests passed successfully.")
    return True

def main():
    parser = argparse.ArgumentParser(description="Prototype Spike Scaffolder")
    parser.add_argument("--test", action="store_true", help="Run internal self-tests")
    parser.add_argument("--scaffold", type=str, help="Name of prototype spike to scaffold")
    parser.add_argument("--output-dir", type=str, default=".tmp/prototypes", help="Output directory")
    parser.add_argument("--desc", type=str, default="Interactive prototype spike", help="Description")
    parser.add_argument("--validate", type=str, help="Path to spike directory to validate")

    args = parser.parse_args()

    if args.test:
        success = run_self_tests()
        sys.exit(0 if success else 1)

    if args.scaffold:
        out_dir = Path(args.output_dir)
        path = scaffold_spike(args.scaffold, out_dir, args.desc)
        print(f"[PASS] Prototype scaffolded at: {path}")
        sys.exit(0)

    if args.validate:
        p = Path(args.validate)
        res = validate_spike(p)
        if res["valid"]:
            print(f"[PASS] Prototype valid at {res['entry']}")
            sys.exit(0)
        else:
            print(f"[FAIL] Prototype invalid: {res}")
            sys.exit(1)

    parser.print_help()
    sys.exit(0)

if __name__ == "__main__":
    main()
