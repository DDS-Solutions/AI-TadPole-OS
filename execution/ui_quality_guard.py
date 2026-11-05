"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / ui_quality_guard
- **Primary Entrypoints**: `scan_ui_quality`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[ERROR]`, `[OK]`, `[INFO]`, `[SAVE]`
"""

import argparse
import json
import re
import sys
from pathlib import Path
from typing import List, Dict, Any

REPO_ROOT = Path(__file__).parent.parent.resolve()
SRC_DIR = REPO_ROOT / "src"
TMP_DIR = REPO_ROOT / ".tmp"

# DESIGN.md Rule 1: No Pure #000000 or Raw Untinted Grays
PURE_BLACK_PATTERN = re.compile(r'(?:#000(?:000)?\b|bg-black\b|text-black\b)', re.IGNORECASE)
UNTINTED_GRAY_SURFACE_PATTERN = re.compile(r'\bbg-(?:gray|neutral)-(?:800|900|950)\b')

# DESIGN.md Rule 2: No Card Inception (Nested Cards with identical background/borders)
CARD_INCEPTION_PATTERN = re.compile(
    r'<div[^>]*className=["\'][^"\']*(?:bg-slate-900|bg-zinc-900)[^"\']*["\'][^>]*>\s*'
    r'<div[^>]*className=["\'][^"\']*(?:bg-slate-900|bg-zinc-900)[^"\']*["\']',
    re.DOTALL,
)

# DESIGN.md Rule 4: No Low-Contrast Text on Badges (gray text on colored accent background within same static class token string)
LOW_CONTRAST_BADGE_PATTERN = re.compile(
    r'["\']([^"\']*\bbg-(?:emerald|cyan|amber|red|rose|purple|indigo|blue)-(?:400|500|600)(?:/\d+)?\b[^"\']*)'
    r'["\']'
)
MUTED_GRAY_TEXT_RE = re.compile(r'\btext-(?:zinc|slate|gray|neutral)-(?:400|500|600)\b')
ACCENT_TEXT_RE = re.compile(
    r'\btext-(?:emerald|cyan|amber|red|rose|purple|indigo|blue|white|zinc-100|zinc-200|slate-100|slate-200)\b'
)

# DESIGN.md Rule 5: No Generic Icon Tiles stacked directly above section headings
GENERIC_ICON_TILE_PATTERN = re.compile(
    r'<div[^>]*className=["\'][^"\']*\brounded-(?:lg|xl|2xl)\b[^"\']*\b(?:p-\d|w-\d+\s+h-\d+)\b[^"\']*["\'][^>]*>\s*'
    r'<[A-Z][a-zA-Z0-9]*[^>]*/>\s*</div>\s*<h[1-4]\b',
    re.DOTALL,
)

# DESIGN.md Rule 6: Mandatory Icon Accessibility (icon-only <button> or interactive <svg> must have aria-label or title)
UNLABELLED_BUTTON_PATTERN = re.compile(
    r'<button(?![^>]*aria-label=)(?![^>]*title=)[^>]*>\s*<[A-Z][a-zA-Z0-9]*\s*(?:className|size|color|width|height)=[^>]*/>\s*</button>',
    re.DOTALL,
)
UNLABELLED_INTERACTIVE_SVG_PATTERN = re.compile(
    r'<svg(?=[^>]*\bonClick=)(?![^>]*\baria-label=)(?![^>]*\btitle=)(?![^>]*\baria-hidden=["\']true["\'])[^>]*>(?!\s*<title\b)',
    re.DOTALL,
)

# DESIGN.md Shared UI Primitives: Avoid raw browser confirm()/alert() instead of Confirm_Dialog / Toast_Center
RAW_DIALOG_PATTERN = re.compile(r'\bwindow\.(?:confirm|alert)\s*\(')


def scan_ui_quality(strict: bool = False) -> Dict[str, Any]:
    findings: List[Dict[str, Any]] = []
    scanned_files = 0

    if not SRC_DIR.exists():
        print(f"[ERROR] [UI Guard] Directory not found: {SRC_DIR}")
        sys.exit(1)

    for filepath in sorted(SRC_DIR.rglob("*.tsx")):
        if ".test." in filepath.name or ".spec." in filepath.name or "vite-env" in filepath.name:
            continue

        scanned_files += 1
        content = filepath.read_text(encoding="utf-8", errors="ignore")
        rel_path = str(filepath.relative_to(REPO_ROOT)).replace("\\", "/")

        # 1a. DESIGN.md Rule 1: Pure black literals
        for match in PURE_BLACK_PATTERN.finditer(content):
            line_no = content[:match.start()].count("\n") + 1
            findings.append({
                "rule": "PURE_BLACK_USAGE",
                "file": rel_path,
                "line": line_no,
                "snippet": match.group(0),
                "severity": "WARNING",
                "message": "DESIGN.md #1: Use HSL-tinted dark surface (e.g. #090d16, zinc-950, or slate-950) instead of pure black"
            })

        # 1b. DESIGN.md Rule 1: Raw untinted gray surfaces
        for match in UNTINTED_GRAY_SURFACE_PATTERN.finditer(content):
            line_no = content[:match.start()].count("\n") + 1
            findings.append({
                "rule": "UNTINTED_GRAY_SURFACE",
                "file": rel_path,
                "line": line_no,
                "snippet": match.group(0),
                "severity": "INFO",
                "message": "DESIGN.md #1: Prefer HSL-tinted dark surfaces (slate-900/950 or zinc-900/950) over untinted gray/neutral surfaces"
            })

        # 2. DESIGN.md Rule 2: Nested identical cards (Card Inception)
        for match in CARD_INCEPTION_PATTERN.finditer(content):
            line_no = content[:match.start()].count("\n") + 1
            findings.append({
                "rule": "CARD_INCEPTION",
                "file": rel_path,
                "line": line_no,
                "snippet": "nested card container",
                "severity": "INFO",
                "message": "DESIGN.md #2: Card container nested inside identical card container; consider dividers or spatial depth shifts"
            })

        # 3. DESIGN.md Rule 3: Font Uniformity check on standalone top-level Pages
        if rel_path.startswith("src/pages/") and content.count("\n") > 120:
            if (
                "font-mono" not in content
                and "--font-mono" not in content
                and "<code" not in content
                and "../components/" not in content
            ):
                findings.append({
                    "rule": "FONT_UNIFORMITY",
                    "file": rel_path,
                    "line": 1,
                    "snippet": "missing font-mono in page view",
                    "severity": "INFO",
                    "message": "DESIGN.md #3: Pair Sans headers with JetBrains Mono (font-mono) for system metrics, trace IDs, and telemetry"
                })

        # 4. DESIGN.md Rule 4: Low-contrast gray text on colored badge backgrounds
        for match in LOW_CONTRAST_BADGE_PATTERN.finditer(content):
            cls_tokens = match.group(1)
            static_tokens = " ".join(
                tok for tok in cls_tokens.split()
                if not tok.startswith(("hover:", "focus:", "group-hover:", "disabled:", "active:", "placeholder:"))
            )
            if (
                re.search(r'\bbg-(?:emerald|cyan|amber|red|rose|purple|indigo|blue)-(?:400|500|600)(?:/\d+)?\b', static_tokens)
                and MUTED_GRAY_TEXT_RE.search(static_tokens)
                and not ACCENT_TEXT_RE.search(static_tokens)
            ):
                line_no = content[:match.start()].count("\n") + 1
                findings.append({
                    "rule": "LOW_CONTRAST_BADGE_TEXT",
                    "file": rel_path,
                    "line": line_no,
                    "snippet": static_tokens[:80],
                    "severity": "WARNING",
                    "message": "DESIGN.md #4: Avoid muted gray text on colored badge backgrounds; use matching accent text (e.g. text-emerald-400 on bg-emerald-500/10)"
                })

        # 5. DESIGN.md Rule 5: Generic icon tiles stacked above headings
        for match in GENERIC_ICON_TILE_PATTERN.finditer(content):
            line_no = content[:match.start()].count("\n") + 1
            findings.append({
                "rule": "GENERIC_ICON_TILE",
                "file": rel_path,
                "line": line_no,
                "snippet": match.group(0)[:60].replace("\n", " ") + "...",
                "severity": "INFO",
                "message": "DESIGN.md #5: Avoid placing rounded-square icon tiles above section headings; use inline icons with clean typography"
            })

        # 6a. DESIGN.md Rule 6: Unlabelled icon-only buttons
        for match in UNLABELLED_BUTTON_PATTERN.finditer(content):
            line_no = content[:match.start()].count("\n") + 1
            findings.append({
                "rule": "UNLABELLED_ICON_BUTTON",
                "file": rel_path,
                "line": line_no,
                "snippet": match.group(0)[:60].replace("\n", " ") + "...",
                "severity": "WARNING",
                "message": "DESIGN.md #6: Icon-only <button> is missing aria-label or title attribute for screen readers"
            })

        # 6b. DESIGN.md Rule 6: Unlabelled interactive SVGs
        for match in UNLABELLED_INTERACTIVE_SVG_PATTERN.finditer(content):
            line_no = content[:match.start()].count("\n") + 1
            findings.append({
                "rule": "UNLABELLED_INTERACTIVE_SVG",
                "file": rel_path,
                "line": line_no,
                "snippet": match.group(0)[:60].replace("\n", " ") + "...",
                "severity": "WARNING",
                "message": "DESIGN.md #6: Interactive <svg> is missing aria-label or title attribute"
            })

        # 7. DESIGN.md Shared UI Primitives: Raw window.confirm / window.alert
        for match in RAW_DIALOG_PATTERN.finditer(content):
            line_no = content[:match.start()].count("\n") + 1
            findings.append({
                "rule": "RAW_BROWSER_DIALOG",
                "file": rel_path,
                "line": line_no,
                "snippet": match.group(0),
                "severity": "INFO",
                "message": "DESIGN.md Primitives: Use shared Confirm_Dialog or Toast_Center from src/components/ui instead of window.confirm/alert"
            })

    warning_count = sum(1 for f in findings if f["severity"] == "WARNING")
    error_count = sum(1 for f in findings if f["severity"] == "ERROR")
    info_count = sum(1 for f in findings if f["severity"] == "INFO")

    failed = error_count > 0 or (strict and warning_count > 0)
    report = {
        "status": "FAIL" if failed else "PASS",
        "scanned_files": scanned_files,
        "total_findings": len(findings),
        "summary": {
            "errors": error_count,
            "warnings": warning_count,
            "info": info_count,
        },
        "findings": findings,
    }

    TMP_DIR.mkdir(parents=True, exist_ok=True)
    report_file = TMP_DIR / "ui_quality_report.json"
    report_file.write_text(json.dumps(report, indent=2), encoding="utf-8")

    print(f"[OK] [UI Guard] Scanned {scanned_files} TSX files against DESIGN.md contract.")
    print(f"[INFO] [UI Guard] Findings: {len(findings)} ({error_count} errors, {warning_count} warnings, {info_count} info).")
    print(f"[SAVE] [UI Guard] Report written to .tmp/ui_quality_report.json")

    if failed:
        sys.exit(1)

    return report


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Tadpole OS UI Quality Guard (DESIGN.md Enforcer)")
    parser.add_argument("--strict", action="store_true", help="Exit with code 1 if any WARNING findings are detected")
    args = parser.parse_args()
    scan_ui_quality(strict=args.strict)
