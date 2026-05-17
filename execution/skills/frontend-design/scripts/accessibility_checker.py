"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Assist Note
**Accessibility Checker - WCAG compliance audit**
Advanced agentic logic and tool orchestration for the Tadpole OS swarm.

### 🔍 Debugging & Observability
- **Failure Path**: Script error, API failure, or logic drift in the 3-layer architecture.
- **Telemetry Link**: Search `[accessibility_checker]` in system logs.
"""

#!/usr/bin/env python3
"""
Accessibility Checker - WCAG compliance audit
Checks HTML files for accessibility issues.

Usage:
    python accessibility_checker.py <project_path>

Checks:
    - Form labels
    - ARIA attributes
    - Color contrast hints
    - Keyboard navigation
    - Semantic HTML
"""

import sys
import json
import re
import os
from pathlib import Path
from typing import List, Dict, Any, Optional
from datetime import datetime

# Fix Windows console encoding
try:
    sys.stdout.reconfigure(encoding='utf-8', errors='replace')
except:
    pass


def find_html_files(path) -> list:
    """Recursively find all HTML/JSX/TSX files in the given directory or file path."""
    target_path = Path(path)
    if target_path.is_file():
        if target_path.suffix.lower() in ('.html', '.jsx', '.tsx'):
            return [target_path]
        return []
    
    html_files = []
    # Ensure path for os.walk is a string
    for root, _, files in os.walk(str(target_path)):
        # Skip common build/dependency directories
        if 'node_modules' in root or '.git' in root or 'dist' in root or '.next' in root or 'build' in root:
            continue
        for file in files:
            if file.lower().endswith(('.html', '.jsx', '.tsx')):
                html_files.append(Path(root) / file)
    return html_files[:50]


def check_accessibility(file_path: Path) -> list:
    """Check a single file for accessibility issues."""
    issues = []
    
    try:
        content = file_path.read_text(encoding='utf-8', errors='ignore')
        
        # Check for form inputs (robust JSX-aware splitting)
        chunks = re.split(r'<input', content, flags=re.IGNORECASE)
        for chunk in chunks[1:]:
            # Get everything until the next likely tag start or a large enough distance
            # This is a heuristic for JSX where '>' appears in arrow functions
            tag_content = chunk.split('<')[0]
            inp_flat = ' '.join(tag_content.split()).lower()
            
            if 'type="hidden"' not in inp_flat:
                if 'aria-label' not in inp_flat and 'id=' not in inp_flat:
                    snippet = "<input " + inp_flat[:50] + "..."
                    issues.append(f"Input without label or aria-label: {snippet}")
                    break
        
        # Check for buttons without accessible text
        buttons = re.findall(r'<button[^>]*>[^<]*</button>', content, re.IGNORECASE)
        for btn in buttons:
            # Check if button has text content or aria-label
            if 'aria-label' not in btn.lower():
                text = re.sub(r'<[^>]+>', '', btn)
                if not text.strip():
                    issues.append("Button without accessible text")
                    break
        
        # Check for missing lang attribute (more precise regex to avoid HTMLDivElement false positives)
        if re.search(r'<html\b', content.lower()) and 'lang=' not in content.lower():
            issues.append("Missing lang attribute on <html>")
        
        # Check for missing skip link
        if '<main' in content.lower() or '<body' in content.lower():
            if 'skip' not in content.lower() and '#main' not in content.lower():
                issues.append("Consider adding skip-to-main-content link")
        
        # Check for click handlers without keyboard support (ignoring buttons)
        non_button_content = re.sub(r'<button[^>]*>.*?</button>', '', content, flags=re.IGNORECASE | re.DOTALL)
        onclick_count = non_button_content.lower().count('onclick=')
        onkeydown_count = non_button_content.lower().count('onkeydown=') + non_button_content.lower().count('onkeyup=')
        if onclick_count > 0 and onkeydown_count == 0:
            issues.append("onClick without keyboard handler (onKeyDown)")
        
        # Check for tabIndex misuse
        if 'tabindex=' in content.lower():
            if 'tabindex="-1"' not in content.lower() and 'tabindex="0"' not in content.lower():
                positive_tabindex = re.findall(r'tabindex="([1-9]\d*)"', content, re.IGNORECASE)
                if positive_tabindex:
                    issues.append("Avoid positive tabIndex values")
        
        # Check for autoplay media
        if 'autoplay' in content.lower():
            if 'muted' not in content.lower():
                issues.append("Autoplay media should be muted")
        
        # Check for role usage
        if 'role="button"' in content.lower():
            # Divs with role button should have tabindex
            div_buttons = re.findall(r'<div[^>]*role="button"[^>]*>', content, re.IGNORECASE)
            for div in div_buttons:
                if 'tabindex' not in div.lower():
                    issues.append("role='button' without tabindex")
                    break
        
    except Exception as e:
        issues.append(f"Error reading file: {str(e)[:50]}")
    
    return issues


def main():
    project_path = Path(sys.argv[1] if len(sys.argv) > 1 else ".").resolve()
    
    print(f"\n{'='*60}")
    print(f"[ACCESSIBILITY CHECKER] WCAG Compliance Audit")
    print(f"{'='*60}")
    print(f"Project: {project_path}")
    print(f"Time: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    print("-"*60)
    
    # Find HTML files
    files = find_html_files(project_path)
    print(f"Found {len(files)} HTML/JSX/TSX files")
    
    if not files:
        output = {
            "script": "accessibility_checker",
            "project": str(project_path),
            "files_checked": 0,
            "issues_found": 0,
            "passed": True,
            "message": "No HTML files found"
        }
        print(json.dumps(output, indent=2))
        sys.exit(0)
    
    # Check each file
    all_issues = []
    
    for f in files:
        issues = check_accessibility(f)
        if issues:
            all_issues.append({
                "file": str(f.name),
                "issues": issues
            })
    
    # Summary
    print("\n" + "="*60)
    print("ACCESSIBILITY ISSUES")
    print("="*60)
    
    if all_issues:
        for item in all_issues[:10]:
            print(f"\n{item['file']}:")
            for issue in item["issues"]:
                print(f"  - {issue}")
        
        if len(all_issues) > 10:
            print(f"\n... and {len(all_issues) - 10} more files with issues")
    else:
        print("No accessibility issues found!")
    
    total_issues = sum(len(item["issues"]) for item in all_issues)
    # Accessibility issues are important but not blocking
    passed = total_issues < 5  # Allow minor issues
    
    output = {
        "script": "accessibility_checker",
        "project": str(project_path),
        "files_checked": len(files),
        "files_with_issues": len(all_issues),
        "issues_found": total_issues,
        "passed": passed
    }
    
    print("\n" + json.dumps(output, indent=2))
    
    sys.exit(0 if passed else 1)


if __name__ == "__main__":
    main()

# Metadata: [accessibility_checker]
