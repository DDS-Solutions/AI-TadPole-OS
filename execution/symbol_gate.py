#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Quality:Verification

### AI Assist Note
**ADG Symbol Gate (ADG-SG)**: Deterministic validator that parses file headers 
(AI Assist Note and Debugging & Observability blocks) and enforces that any code 
symbols mentioned inside backticks exist in the actual file body.

### 🔍 Debugging & Observability
- **Failure Path**: Missing or mismatched backticked code symbols in headers.
- **Trace Scope**: `execution::symbol_gate`
"""

import os
import re
import sys
from pathlib import Path

# Shared utilities
from py_utils import init_utf8, Colors

# Initialize unbuffered UTF-8 streams for Windows console hosts
init_utf8()

def print_result(file_path: str, passed: bool, message: str):
    icon = f"{Colors.GREEN}[OK]{Colors.ENDC}" if passed else f"{Colors.RED}[FAIL]{Colors.ENDC}"
    print(f"{icon} [ADG-SG] {file_path}: {message}")

def extract_symbols(header_block: str, whitelist: set, is_md: bool = False) -> set:
    """Extract backticked symbols from comment block, filtering out non-code terms."""
    # Find all strings inside backticks
    raw_matches = re.findall(r'`([^`\n]+)`', header_block)
    symbols = set()
    
    for s in raw_matches:
        s = s.strip()
        # Clean trailing function parentheses, e.g. my_func() -> my_func
        if s.endswith('()'):
            s = s[:-2]
            
        if s in whitelist:
            continue
            
        if is_md:
            # For markdown, we allow paths, files, and identifiers.
            if any(char in s for char in (' ', '$', '{', '}', '[', ']', '<', '>', '=', '+', '*', '"', "'")):
                continue
            if s.startswith(('http://', 'https://')):
                continue
            # Ensure it has some valid characters (not just punctuation)
            if not re.search(r'[a-zA-Z0-9]', s):
                continue
            symbols.add(s)
        else:
            # Skip if contains invalid symbol characters (spaces, paths, math, CSS, etc.)
            if any(char in s for char in (' ', '/', '\\', '-', ':', '$', '{', '}', '[', ']', '.', '<', '>')):
                continue
            # Ensure it is a valid programming identifier structure
            if re.match(r'^[a-zA-Z_][a-zA-Z0-9_]*$', s):
                symbols.add(s)
                
    return symbols

def find_workspace_root(start_path: Path) -> Path:
    current = start_path.resolve()
    if current.is_file():
        current = current.parent
    for parent in [current] + list(current.parents):
        if (parent / "server-rs" / ".env.example").exists() or (parent / ".git").exists():
            return parent
    return current

def main():
    print("🔍 Starting Active Documentation Guard - Symbol Gate (ADG-SG)...")
    
    strict_mode = "--strict" in sys.argv
    args = [arg for arg in sys.argv[1:] if arg != "--strict"]
    
    target = args[0] if args else "."
    project_path = Path(target).resolve()
    
    exclude_dirs = {
        'venv', '__pycache__', '.git', 'node_modules', 'target', '.agent', 'docs',
        'execution', '.tmp', 'data', 'archive', 'scratch', 'scripts', 'tmp', 'dist'
    }
    
    import json
    workspace_root = find_workspace_root(project_path)
    globals_path = workspace_root / ".agent" / "globals.json"
    whitelist = set()
    if globals_path.exists():
        try:
            whitelist = set(json.loads(globals_path.read_text(encoding='utf-8')))
        except Exception as e:
            print(f"{Colors.YELLOW}[WARN] Could not parse .agent/globals.json: {e}. Using fallback whitelist.{Colors.ENDC}")
    
    if not whitelist:
        whitelist = {
            'true', 'false', 'any', 'unwrap', 'string', 'number', 'boolean', 'void', 'null', 'undefined',
            'str', 'u8', 'u16', 'u32', 'u64', 'usize', 'i32', 'i64', 'f32', 'f64', 'Self', 'self',
            'Ok', 'Err', 'Option', 'Result', 'Some', 'None', 'Arc', 'Mutex', 'State', 'Body', 'Request',
            'Response', 'StatusCode', 'Next', 'axum', 'tokio', 'std', 'env', 'var', 'cfg', 'test', 'tests',
            'Error', 'props', 'Props', 'interface', 'type', 'const', 'let', 'function', 'class', 'import'
        }
    
    env_example = workspace_root / "server-rs" / ".env.example"
    if env_example.exists():
        try:
            content = env_example.read_text(encoding='utf-8', errors='ignore')
            for line in content.splitlines():
                line = line.strip()
                if not line or line.startswith('#') or '=' not in line:
                    continue
                key = line.split('=')[0].strip()
                whitelist.add(key)
        except Exception as e:
            print(f"[WARN] Failed to load .env.example keys: {e}")

    total_files = 0
    total_symbols_checked = 0
    failed_files = 0
    drift_errors = []

    # Identify files to check (support directory walking or single file checking)
    files_to_check = []
    if project_path.is_file():
        if project_path.suffix in ('.rs', '.ts', '.tsx', '.md'):
            files_to_check.append((project_path, project_path.name))
    else:
        for root, dirs, files in os.walk(project_path):
            # Prune excluded directories on-the-fly
            dirs[:] = [d for d in dirs if d not in exclude_dirs]
            for file in files:
                if file.endswith(('.rs', '.ts', '.tsx', '.md')):
                    file_path = Path(root) / file
                    try:
                        relative_path = file_path.relative_to(project_path).as_posix()
                    except ValueError:
                        relative_path = file_path.as_posix()
                    files_to_check.append((file_path, relative_path))

    for file_path, relative_path in files_to_check:
        try:
            content = file_path.read_text(encoding='utf-8', errors='ignore')
        except Exception:
            continue
            
        if '### AI Assist Note' not in content and '### 🔍 Debugging & Observability' not in content:
            continue
            
        total_files += 1
        
        is_md = file_path.suffix == '.md'
            
        # Split file into header comment block and code body
        lines = content.splitlines()
        header_lines = []
        code_lines = []
        in_header = True
        
        for line in lines:
            trimmed = line.strip()
            if is_md:
                # Markdown header is blockquote or empty line
                is_header_line = trimmed.startswith('>') or trimmed == ''
                if in_header and is_header_line:
                    cleaned = trimmed.lstrip('>').strip()
                    header_lines.append(cleaned)
                else:
                    in_header = False
                    code_lines.append(line)
            else:
                # Classify typical comments in Rust/JS/TS
                is_comment = (
                    trimmed.startswith('//') or 
                    trimmed.startswith('/*') or 
                    trimmed.startswith('*') or 
                    trimmed.endswith('*/') or 
                    trimmed == ''
                )
                if in_header and is_comment:
                    header_lines.append(line)
                else:
                    in_header = False
                    code_lines.append(line)
                
        header_block = '\n'.join(header_lines)
        code_body = '\n'.join(code_lines)
        
        # Extract symbols from header
        symbols = extract_symbols(header_block, whitelist, is_md)
        if not symbols:
            continue
            
        # Verify each symbol is present in the code body (or exists as a local path for markdown)
        file_failed = False
        missing_symbols = []
        
        for s in symbols:
            total_symbols_checked += 1
            if is_md:
                # 1. Case-insensitive search in body
                body_matched = re.search(re.escape(s), code_body, re.IGNORECASE) is not None
                
                # 2. Check if it's a physical path relative to workspace_root
                path_matched = False
                if '/' in s or '\\' in s or '.' in s:
                    test_path = workspace_root / s
                    path_matched = test_path.exists()
                    
                if not body_matched and not path_matched:
                    file_failed = True
                    missing_symbols.append(s)
            else:
                # Search using word boundaries to ensure literal symbol matching
                pattern = r'\b' + re.escape(s) + r'\b'
                if not re.search(pattern, code_body):
                    file_failed = True
                    missing_symbols.append(s)
                    
        if file_failed:
            failed_files += 1
            if is_md:
                msg = f"Mismatched references in header not found in body or disk: {', '.join(f'`{x}`' for x in missing_symbols)}"
            else:
                msg = f"Mismatched symbols in header not found in code: {', '.join(f'`{x}`' for x in missing_symbols)}"
            print_result(relative_path, False, msg)
            drift_errors.append((relative_path, missing_symbols))
        else:
            print_result(relative_path, True, f"Verified {len(symbols)} references" if is_md else f"Verified {len(symbols)} symbols")

    print("\n" + "=" * 60)
    print("  ADG-SG AUDIT SUMMARY")
    print("=" * 60)
    print(f"Total Files Audited: {total_files}")
    print(f"Total Symbols Checked: {total_symbols_checked}")
    print(f"Failed Files: {failed_files}")
    
    if failed_files > 0:
        if strict_mode:
            print(f"\n{Colors.RED}[FAIL] Symbol Gate validation failed. Please align headers with code.{Colors.ENDC}\n")
            sys.exit(1)
        else:
            print(f"\n{Colors.YELLOW}[WARN] Symbol Gate validation failed (Warning Only). Run with --strict to enforce.{Colors.ENDC}\n")
            sys.exit(0)
    else:
        print(f"\n{Colors.GREEN}[OK] 100% Symbol-to-Header Parity Achieved!{Colors.ENDC}\n")
        sys.exit(0)

if __name__ == "__main__":
    main()

# Metadata: [symbol_gate]
