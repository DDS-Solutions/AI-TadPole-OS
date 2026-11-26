"""
@docs ARCHITECTURE:Registry:Skills

### AI Context Alignment
- **Subsystem**: Developer Scripts / lighthouse_audit
- **Primary Entrypoints**: `run_lighthouse`, `get_summary`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[lighthouse_audit]`
- **Witness Tests**: none declared
"""

import subprocess
import json
import sys
import os
import tempfile

def run_lighthouse(url: str) -> dict:
    """Run Lighthouse audit on URL."""
    try:
        with tempfile.NamedTemporaryFile(suffix='.json', delete=False) as f:
            output_path = f.name

        result = subprocess.run(
            [
                "lighthouse",
                url,
                "--output=json",
                f"--output-path={output_path}",
                "--chrome-flags=--headless",
                "--only-categories=performance,accessibility,best-practices,seo"
            ],
            capture_output=True,
            text=True,
            timeout=120
        )

        if os.path.exists(output_path):
            with open(output_path, 'r') as f:
                report = json.load(f)
            os.unlink(output_path)

            categories = report.get("categories", {})
            return {
                "url": url,
                "scores": {
                    "performance": int(categories.get("performance", {}).get("score", 0) * 100),
                    "accessibility": int(categories.get("accessibility", {}).get("score", 0) * 100),
                    "best_practices": int(categories.get("best-practices", {}).get("score", 0) * 100),
                    "seo": int(categories.get("seo", {}).get("score", 0) * 100)
                },
                "summary": get_summary(categories)
            }
        else:
            return {"error": "Lighthouse failed to generate report", "stderr": result.stderr[:500]}

    except subprocess.TimeoutExpired:
        return {"error": "Lighthouse audit timed out"}
    except FileNotFoundError:
        return {"error": "Lighthouse CLI not found. Install with: npm install -g lighthouse"}

def get_summary(categories: dict) -> str:
    """Generate summary based on scores."""
    perf = categories.get("performance", {}).get("score", 0) * 100
    if perf >= 90:
        return "[OK] Excellent performance"
    elif perf >= 50:
        return "[!] Needs improvement"
    else:
        return "[X] Poor performance"

def run_self_tests() -> bool:
    """Run internal self-tests for lighthouse scoring and summary logic."""
    mock_good = {"performance": {"score": 0.95}}
    mock_mid = {"performance": {"score": 0.72}}
    mock_bad = {"performance": {"score": 0.40}}

    assert "[OK]" in get_summary(mock_good), "High score should return OK"
    assert "[!]" in get_summary(mock_mid), "Medium score should return Needs improvement"
    assert "[X]" in get_summary(mock_bad), "Low score should return Poor performance"

    print("[PASS] lighthouse_audit self-tests passed successfully.")
    return True

if __name__ == "__main__":
    if sys.platform == "win32":
        if hasattr(sys.stdout, 'reconfigure'):
            sys.stdout.reconfigure(encoding='utf-8', errors='replace')
        if hasattr(sys.stderr, 'reconfigure'):
            sys.stderr.reconfigure(encoding='utf-8', errors='replace')

    if "--test" in sys.argv:
        success = run_self_tests()
        sys.exit(0 if success else 1)

    print("[lighthouse_audit] Initializing Lighthouse performance audit...")

    if len(sys.argv) < 2:
        print(json.dumps({"error": "Usage: python lighthouse_audit.py <url> [--test]"}))
        sys.exit(1)

    result = run_lighthouse(sys.argv[1])
    print(json.dumps(result, indent=2))
