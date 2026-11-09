#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Registry:Skills

### AI Context Alignment
- **Subsystem**: Agent Skills / frontend-design / accessibility_checker
- **Primary Entrypoints**: `find_html_files`, `check_accessibility`, `run_self_tests`, `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic WCAG accessibility checks without modifying workspace files.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[accessibility_checker]`, `[PASS]`, `[FAIL]`, `[A11Y]`
- **Witness Tests**: none declared
"""

import sys
import io
import os
import json
import re
import argparse
from pathlib import Path
from datetime import datetime
from typing import List, Dict, Any, Tuple, Optional

# Fix Windows console encoding
if sys.platform == "win32":
    try:
        sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
        sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8')
    except Exception:
        pass


def find_html_files(project_path: Path) -> List[Path]:
    """Find all HTML/JSX/TSX files within the target project scope."""
    skip_dirs = {
        'node_modules', '.next', 'dist', 'build', '.git', 'coverage',
        'target', '.agent', 'docs', '.tmp', 'tmp', 'playwright-report'
    }

    files = []
    for root, dirs, filenames in os.walk(project_path):
        dirs[:] = [d for d in dirs if d not in skip_dirs]
        for filename in filenames:
            ext = os.path.splitext(filename)[1].lower()
            if ext in {'.html', '.jsx', '.tsx'}:
                files.append(Path(root) / filename)
                if len(files) >= 50:
                    return files

    return files


def check_accessibility(file_path: Path) -> List[str]:
    """Check a single file for WCAG accessibility issues."""
    issues = []
    
    try:
        content = file_path.read_text(encoding='utf-8', errors='ignore')
        
        # 1. Check for form inputs without labels or aria-label
        inputs = re.findall(r'<input[^>]*>', content, re.IGNORECASE)
        for inp in inputs:
            if 'type="hidden"' not in inp.lower():
                if 'aria-label' not in inp.lower() and 'id=' not in inp.lower():
                    issues.append("Input without label or aria-label")
                    break
        
        # 2. Check for buttons without accessible text
        buttons = re.findall(r'<button[^>]*>[^<]*</button>', content, re.IGNORECASE)
        for btn in buttons:
            if 'aria-label' not in btn.lower() and 'title=' not in btn.lower():
                text = re.sub(r'<[^>]+>', '', btn)
                if not text.strip():
                    issues.append("Button without accessible text or aria-label")
                    break
        
        # 3. Check for missing lang attribute on <html> in raw HTML
        if file_path.suffix.lower() in ['.html', '.htm']:
            if '<html' in content.lower() and 'lang=' not in content.lower():
                issues.append("Missing lang attribute on <html> element")

        # 4. Check for click handlers without keyboard support on non-button elements
        non_button_onclick = re.findall(
            r'<(?!button|a\s|input)[\w]+[^>]*onclick=', content, re.IGNORECASE
        )
        onkeydown_count = (
            content.lower().count('onkeydown=') +
            content.lower().count('onkeyup=') +
            content.lower().count('onkeypress=')
        )
        if non_button_onclick and onkeydown_count == 0:
            issues.append("onClick on non-interactive element without keyboard handler (onKeyDown)")

        # 5. Check for positive tabIndex misuse
        if 'tabindex=' in content.lower():
            positive_tabindex = re.findall(r'tabindex="([1-9]\d*)"', content, re.IGNORECASE)
            if positive_tabindex:
                issues.append("Avoid positive tabIndex values (use 0 or -1)")
        
        # 6. Check for autoplay media without muted
        if 'autoplay' in content.lower():
            if 'muted' not in content.lower():
                issues.append("Autoplay media must be muted to comply with WCAG")
        
        # 7. Check for role="button" without tabindex
        if 'role="button"' in content.lower():
            div_buttons = re.findall(r'<div[^>]*role="button"[^>]*>', content, re.IGNORECASE)
            for div in div_buttons:
                if 'tabindex' not in div.lower():
                    issues.append("role='button' without tabIndex keyboard accessibility")
                    break
        
    except Exception as e:
        issues.append(f"Error reading file: {str(e)[:50]}")
    
    return issues


def run_self_tests() -> bool:
    """
    Execute self-tests verifying accessibility audit heuristics.
    """
    print("[accessibility_checker] Executing self-test suite...")
    
    temp_dir = Path(".tmp/test_a11y")
    temp_dir.mkdir(parents=True, exist_ok=True)
    
    # 1. Bad HTML sample
    bad_file = temp_dir / "bad.tsx"
    bad_file.write_text(
        '<form><input type="text" name="username" /><button></button><div role="button">Click me</div></form>',
        encoding='utf-8'
    )
    
    # 2. Good HTML sample
    good_file = temp_dir / "good.tsx"
    good_file.write_text(
        '<form><input type="text" id="username" aria-label="Username" /><button aria-label="Submit">Send</button></form>',
        encoding='utf-8'
    )
    
    try:
        bad_issues = check_accessibility(bad_file)
        assert any("Input without label" in i for i in bad_issues), "Failed to detect unlabelled input"
        assert any("Button without accessible text" in i for i in bad_issues), "Failed to detect empty button"
        assert any("role='button'" in i for i in bad_issues), "Failed to detect role='button' without tabindex"
        
        good_issues = check_accessibility(good_file)
        assert len(good_issues) == 0, f"Expected 0 issues for accessible code, got {good_issues}"
    finally:
        if bad_file.exists():
            bad_file.unlink()
        if good_file.exists():
            good_file.unlink()
            
    print("[accessibility_checker] [PASS] All self-tests passed successfully.")
    return True


def main():
    parser = argparse.ArgumentParser(description="Deterministic WCAG Accessibility Checker")
    parser.add_argument("path", nargs="?", default=".", help="Target project root directory or component file")
    parser.add_argument("--test", action="store_true", help="Execute self-test suite and exit")
    parser.add_argument("--json", action="store_true", help="Output machine-readable JSON report")
    
    args = parser.parse_args()
    
    if args.test:
        try:
            success = run_self_tests()
            sys.exit(0 if success else 1)
        except AssertionError as e:
            print(f"[accessibility_checker] [FAIL] Self-test assertion failed: {e}")
            sys.exit(1)
            
    print("[accessibility_checker] Initializing accessibility audit...")
    print("[A11Y] Scanning components for WCAG compliance...")
    project_path = Path(args.path).resolve()
    
    files: List[Path] = []
    if project_path.is_file():
        files.append(project_path)
    else:
        files = find_html_files(project_path)
        
    all_issues = []
    for f in files:
        issues = check_accessibility(f)
        if issues:
            all_issues.append({
                "file": str(f.name),
                "path": str(f),
                "issues": issues
            })
            
    total_issues = sum(len(item["issues"]) for item in all_issues)
    passed = total_issues < 15  # Allow minor non-blocking findings
    
    report = {
        "script": "accessibility_checker",
        "project": str(project_path),
        "files_checked": len(files),
        "files_with_issues": len(all_issues),
        "issues_found": total_issues,
        "passed": passed,
        "findings": all_issues
    }
    
    if args.json:
        print(json.dumps(report, indent=2))
        sys.exit(0 if passed else 1)
        
    print(f"\n{'='*64}")
    print(f"[ACCESSIBILITY CHECKER] WCAG Compliance Audit")
    print(f"{'='*64}")
    print(f"Target Scope     : {project_path}")
    print(f"Files Checked    : {len(files)}")
    print(f"Issues Detected  : {total_issues}")
    print(f"Timestamp        : {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    print(f"{'-'*64}")
    
    if all_issues:
        for item in all_issues[:10]:
            print(f"\n📂 {item['file']}:")
            for issue in item["issues"]:
                print(f"   ⚠️  {issue}")
        if len(all_issues) > 10:
            print(f"\n... and {len(all_issues) - 10} more files with issues")
    else:
        print("🎉 [PASS] No accessibility issues detected!")
        
    print(f"\n{'='*64}")
    if passed:
        print("[accessibility_checker] [PASS] Accessibility checks completed.")
    else:
        print("[accessibility_checker] [FAIL] Excessive accessibility issues detected.")
        
    sys.exit(0 if passed else 1)


if __name__ == "__main__":
    main()
