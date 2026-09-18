#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / deduplicate_headers
- **Primary Entrypoints**: `clean_markdown_file`, `clean_code_file`, `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic removal of redundant inner legacy docstrings and duplicate markdown callouts.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[deduplicate_headers]`
- **Witness Tests**: none declared
"""

import os
import re
import sys
from pathlib import Path

# Fix Windows console encoding for Unicode output
if hasattr(sys.stdout, 'reconfigure'):
    sys.stdout.reconfigure(encoding='utf-8', errors='replace')
if hasattr(sys.stderr, 'reconfigure'):
    sys.stderr.reconfigure(encoding='utf-8', errors='replace')

SKIP_DIRS = {
    '.git', 'node_modules', 'dist', 'target', 'build', '__pycache__',
    '.venv', 'venv', '.tmp', 'tmp', 'coverage', '3rdparty', 'scratch',
    'playwright-report', 'test-results', '.system_generated'
}

ROOT = Path(__file__).resolve().parent.parent

EXEMPT_FILES = {
    'tests/test_verify_ai_context.py',
    'execution/migrate_ai_headers.py',
    'execution/verify_ai_context.py',
    'execution/awaken.py',
    'execution/deduplicate_headers.py',
}

def clean_markdown_file(file_path: Path) -> bool:
    rel = file_path.relative_to(ROOT).as_posix()
    if rel in EXEMPT_FILES:
        return False
        
    content = file_path.read_text(encoding='utf-8')
    
    # 1. Remove any secondary > [!IMPORTANT] blocks containing legacy "AI Assist Note" lower down in the file
    # Find all > [!IMPORTANT] blocks
    changed = False
    
    # Check if there are multiple callout blocks
    matches = list(re.finditer(r'>\s*\[!IMPORTANT\][\s\S]*?(?:Traceability via `parity_guard\.py`\.?|\[\w+\] in audit logs\.?|\bparity_guard\.py\b[^\n]*)\s*\r?\n', content))
    
    if len(matches) > 1:
        # Keep the first modernized one if it exists, remove subsequent ones
        for m in reversed(matches[1:]):
            content = content[:m.start()] + content[m.end():]
            changed = True
            
    # Also remove any standalone legacy blocks
    if 'AI Assist Note (Knowledge Heritage)' in content:
        new_content = re.sub(
            r'>\s*\[!IMPORTANT\]\s*\r?\n>\s*\*\*AI Assist Note \(Knowledge Heritage\)[\s\S]*?(?:### 🔍 Debugging & Observability[\s\S]*?Traceability[^\n]*\r?\n|Traceability[^\n]*\r?\n)',
            '',
            content
        )
        if new_content != content:
            content = new_content
            changed = True
            
    if changed:
        file_path.write_text(content, encoding='utf-8')
        return True
        
    return False

def clean_code_file(file_path: Path) -> bool:
    rel = file_path.relative_to(ROOT).as_posix()
    if rel in EXEMPT_FILES:
        return False
        
    content = file_path.read_text(encoding='utf-8')
    ext = file_path.suffix
    changed = False
    
    if ext in {'.ts', '.tsx', '.js', '.jsx'}:
        # Find secondary JSDoc blocks containing '### AI Assist Note'
        new_content = re.sub(
            r'/\*\*[\s\S]*?### AI Assist Note[\s\S]*?\*/\s*',
            '',
            content
        )
        if new_content != content:
            content = new_content
            changed = True
            
    elif ext == '.py':
        # Remove secondary docstrings containing '### AI Assist Note'
        # Keep the very first docstring if it starts at 0 or after #!
        first_doc_end = -1
        if content.startswith('"""'):
            first_doc_end = content.find('"""', 3)
            if first_doc_end != -1:
                first_doc_end += 3
        elif content.startswith('#!'):
            m = re.match(r'^#![^\n]+\n\s*"""', content)
            if m:
                s = content.find('"""', m.end() - 3)
                first_doc_end = content.find('"""', s + 3)
                if first_doc_end != -1:
                    first_doc_end += 3
                    
        if first_doc_end != -1:
            head = content[:first_doc_end]
            body = content[first_doc_end:]
            new_body = re.sub(r'"""[\s\S]*?### AI Assist Note[\s\S]*?"""\s*', '', body)
            new_body = re.sub(r'\n+#!/usr/bin/env python[0-9]*\s*\n', '\n', new_body)
            if new_body != body:
                content = head + new_body
                changed = True
        else:
            # Maybe the file only had legacy header without modern one
            pass
            
    elif ext == '.rs':
        # Rust: strip inner //! comments or /* ... */ containing '### AI Assist Note'
        # Check for //! lines
        lines = content.splitlines(keepends=True)
        new_lines = []
        in_legacy_inner_doc = False
        
        # Check if first lines have //! with AI Context Alignment
        has_top_header = False
        for idx, l in enumerate(lines[:20]):
            if '### AI Context Alignment' in l:
                has_top_header = True
                break
                
        if has_top_header:
            for l in lines:
                if l.startswith('//! @docs') or l.startswith('//! ### AI Assist Note'):
                    # Could be start of legacy block
                    if idx > 15 or '### AI Assist Note' in l:
                        in_legacy_inner_doc = True
                        continue
                if in_legacy_inner_doc:
                    if l.startswith('//!'):
                        continue
                    else:
                        in_legacy_inner_doc = False
                        
                new_lines.append(l)
                
            new_content = "".join(new_lines)
            if new_content != content:
                content = new_content
                changed = True
                
        # Also check for block comments /* ... ### AI Assist Note ... */
        new_content = re.sub(r'/\*[\*!]?[\s\S]*?### AI Assist Note[\s\S]*?\*/\s*', '', content)
        if new_content != content:
            content = new_content
            changed = True
            
    if changed:
        file_path.write_text(content, encoding='utf-8')
        return True
        
    return False

def main():
    target_dir = ROOT
    print(f"🧹 [deduplicate_headers] Scanning {target_dir} for double header blocks...")
    
    code_cleaned = 0
    md_cleaned = 0
    
    for dirpath, dirnames, filenames in os.walk(target_dir):
        dirnames[:] = [d for d in dirnames if d not in SKIP_DIRS]
        for f in filenames:
            fp = Path(dirpath) / f
            ext = fp.suffix
            rel = fp.relative_to(ROOT).as_posix()
            
            if rel in EXEMPT_FILES:
                continue
                
            if ext == '.md':
                if clean_markdown_file(fp):
                    md_cleaned += 1
                    print(f"[CLEANED MD] {rel}")
            elif ext in {'.rs', '.ts', '.tsx', '.js', '.jsx', '.py'}:
                if clean_code_file(fp):
                    code_cleaned += 1
                    print(f"[CLEANED CODE] {rel}")
                    
    print(f"\n📊 Cleanup complete: {code_cleaned} code files cleaned, {md_cleaned} markdown files cleaned.")

if __name__ == "__main__":
    main()
