"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Assist Note
**Sovereign UI Quality Guard (Zero-LLM UI Linter)**
Advanced agentic logic and tool orchestration for the AI-Tadpole-OS swarm.

### 🔍 Debugging & Observability
- **Failure Path**: Script error, API failure, or logic drift in the 3-layer architecture.
- **Telemetry Link**: Search `[ui_quality_guard]` in system logs.
"""

import json
import re
import sys
from pathlib import Path
from typing import List, Dict, Any

REPO_ROOT = Path(__file__).parent.parent.resolve()
SRC_DIR = REPO_ROOT / "src"
TMP_DIR = REPO_ROOT / ".tmp"

# Patterns to scan
PURE_BLACK_PATTERN = re.compile(r'(?:#000(?:000)?\b|bg-black\b|text-black\b)', re.IGNORECASE)
CARD_INCEPTION_PATTERN = re.compile(r'<div[^>]*className=["\'][^"\']*(?:bg-slate-900|bg-zinc-900)[^"\']*["\'][^>]*>\s*<div[^>]*className=["\'][^"\']*(?:bg-slate-900|bg-zinc-900)[^"\']*["\']', re.DOTALL)
UNLABELLED_BUTTON_PATTERN = re.compile(r'<button(?![^>]*aria-label=)(?![^>]*title=)[^>]*>\s*<[A-Z][a-zA-Z0-9]*\s*(?:className|size|color|width|height)=[^>]*/>\s*</button>', re.DOTALL)

def scan_ui_quality() -> Dict[str, Any]:
    findings: List[Dict[str, Any]] = []
    scanned_files = 0

    if not SRC_DIR.exists():
        print(f"[ERROR] [UI Guard] Directory not found: {SRC_DIR}")
        sys.exit(1)

    for filepath in SRC_DIR.rglob("*.tsx"):
        if ".test." in filepath.name or "vite-env" in filepath.name:
            continue

        scanned_files += 1
        content = filepath.read_text(encoding="utf-8", errors="ignore")
        rel_path = str(filepath.relative_to(REPO_ROOT)).replace("\\", "/")

        # 1. Check for pure black literals
        for match in PURE_BLACK_PATTERN.finditer(content):
            line_no = content[:match.start()].count("\n") + 1
            findings.append({
                "rule": "PURE_BLACK_USAGE",
                "file": rel_path,
                "line": line_no,
                "snippet": match.group(0),
                "severity": "WARNING",
                "message": "Use HSL-tinted dark surface (e.g. #090d16 or slate-950) instead of pure black"
            })

        # 2. Check for nested identical cards
        for match in CARD_INCEPTION_PATTERN.finditer(content):
            line_no = content[:match.start()].count("\n") + 1
            findings.append({
                "rule": "CARD_INCEPTION",
                "file": rel_path,
                "line": line_no,
                "snippet": "nested card container",
                "severity": "INFO",
                "message": "Card container nested inside identical card container; consider dividers or spatial padding"
            })

        # 3. Check for unlabelled icon-only buttons
        for match in UNLABELLED_BUTTON_PATTERN.finditer(content):
            line_no = content[:match.start()].count("\n") + 1
            findings.append({
                "rule": "UNLABELLED_ICON_BUTTON",
                "file": rel_path,
                "line": line_no,
                "snippet": match.group(0)[:60] + "...",
                "severity": "WARNING",
                "message": "Icon-only button is missing aria-label or title attribute for screen readers"
            })

    report = {
        "status": "PASS" if not any(f["severity"] == "ERROR" for f in findings) else "FAIL",
        "scanned_files": scanned_files,
        "total_findings": len(findings),
        "findings": findings
    }

    TMP_DIR.mkdir(parents=True, exist_ok=True)
    report_file = TMP_DIR / "ui_quality_report.json"
    report_file.write_text(json.dumps(report, indent=2), encoding="utf-8")

    print(f"[OK] [UI Guard] Scanned {scanned_files} TSX files.")
    print(f"[INFO] [UI Guard] Findings: {len(findings)} ({sum(1 for f in findings if f['severity'] == 'WARNING')} warnings).")
    print(f"[SAVE] [UI Guard] Report written to .tmp/ui_quality_report.json")

    return report

if __name__ == "__main__":
    scan_ui_quality()

# Metadata: [ui_quality_guard]
