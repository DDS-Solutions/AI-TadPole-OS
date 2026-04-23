"""
@docs ARCHITECTURE:Registry:Skills

### AI Assist Note
**Type Coverage Checker - Measures TypeScript/Python type coverage.**
Advanced agentic logic and tool orchestration for the Tadpole OS swarm.

### 🔍 Debugging & Observability
- **Failure Path**: Script error, API failure, or logic drift in the 3-layer architecture.
- **Telemetry Link**: Search `[type_coverage]` in system logs.
"""

#!/usr/bin/env python3
"""
Type Coverage Checker - Measures TypeScript/Python type coverage.
Identifies untyped functions, any usage, and type safety issues.
"""
import sys
import re
import subprocess
from pathlib import Path
from typing import List, Dict, Any

# Fix Windows console encoding for Unicode output
try:
    sys.stdout.reconfigure(encoding='utf-8', errors='replace')
    sys.stderr.reconfigure(encoding='utf-8', errors='replace')
except AttributeError:
    pass  # Python < 3.7

def check_typescript_coverage(project_path: Path) -> dict:
    """Check TypeScript type coverage."""
    issues = []
    passed = []
    stats = {'any_count': 0, 'untyped_functions': 0, 'total_functions': 0}
    # Target core directories only
    core_dirs = ['src', 'execution', 'scripts']
    ts_files = []
    for d in core_dirs:
        dir_path = project_path / d
        if dir_path.exists():
            ts_files.extend(list(dir_path.rglob("*.ts")) + list(dir_path.rglob("*.tsx")))
    
    ts_files = [f for f in ts_files if not any(x in str(f) for x in ['node_modules', '.tmp', '.agent', 'dist', 'build', '.d.ts', 'target', 'ignore', '.test.'])]
    
    if not ts_files:
        return {'type': 'typescript', 'files': 0, 'passed': [], 'issues': ["[!] No TypeScript files found"], 'stats': stats}
    
    for file_path in ts_files[:50]:  # Increase limit for better accuracy
        try:
            content = file_path.read_text(encoding='utf-8', errors='ignore')
            
            # Count 'any' usage
            any_matches = re.findall(r':\s*any\b', content)
            stats['any_count'] += len(any_matches)
            
            # Find functions without return types
            # function name(params) { - no return type
            untyped = re.findall(r'function\s+\w+\s*\([^)]*\)\s*{', content)
            # Arrow functions without types
            untyped += re.findall(r'=\s*\([^:)]*\)\s*=>', content)
            # Class methods without return types: method(params) {
            untyped += re.findall(r'^\s*(?!constructor|if|for|while|switch|catch)\w+\s*\([^:)]*\)\s*{', content, re.MULTILINE)
            stats['untyped_functions'] += len(untyped)
            
            # Count typed functions
            # function name<T>(params): type - improved type regex to handle braces, commas, nested generics
            type_regex = r'[A-Z\w<>[\]|&,:\s{}]+'
            typed = re.findall(r'function\s+\w+\s*(?:<[^>]*>)?\s*\([^)]*\)\s*:\s*' + type_regex, content)
            # (params) : type =>
            typed += re.findall(r'(?::\s*async\s*)?\([^)]*\)\s*:\s*(?:Promise<)?' + type_regex + r'>?\s*=>', content)
            # Class methods with return types: method<T>(params): type {
            typed += re.findall(r'^\s*(?!constructor)\w+\s*(?:<[^>]*>)?\s*\([^)]*\)\s*:\s*' + type_regex, content, re.MULTILINE)
            stats['total_functions'] += len(typed) + len(untyped)
            
        except Exception:
            continue
    
    # Analyze results
    if stats['any_count'] == 0:
        passed.append("[OK] No 'any' types found")
    elif stats['any_count'] <= 5:
        issues.append(f"[!] {stats['any_count']} 'any' types found (acceptable)")
    else:
        issues.append(f"[X] {stats['any_count']} 'any' types found (too many)")
    
    if stats['total_functions'] > 0:
        typed_ratio = (stats['total_functions'] - stats['untyped_functions']) / stats['total_functions'] * 100
        if typed_ratio >= 80:
            passed.append(f"[OK] Type coverage: {typed_ratio:.0f}%")
        elif typed_ratio >= 50:
            issues.append(f"[!] Type coverage: {typed_ratio:.0f}% (improve)")
        else:
            issues.append(f"[X] Type coverage: {typed_ratio:.0f}% (too low)")
    
    passed.append(f"[OK] Analyzed {len(ts_files)} TypeScript files")
    
    return {'type': 'typescript', 'files': len(ts_files), 'passed': passed, 'issues': issues, 'stats': stats}

def check_python_coverage(project_path: Path) -> dict:
    """Check Python type hints coverage."""
    issues = []
    passed = []
    stats = {'untyped_functions': 0, 'typed_functions': 0, 'any_count': 0}
    # Target core directories only
    core_dirs = ['execution', 'scripts', 'src-tauri/src']
    py_files = []
    for d in core_dirs:
        dir_path = project_path / d
        if dir_path.exists():
            py_files.extend(list(dir_path.rglob("*.py")))
    
    py_files = [f for f in py_files if not any(x in str(f) for x in ['venv', '__pycache__', '.git', 'node_modules', '.tmp', '.agent', 'dist', 'build', 'target'])]
    
    if not py_files:
        return {'type': 'python', 'files': 0, 'passed': [], 'issues': ["[!] No Python files found"], 'stats': stats}
    
    for file_path in py_files[:100]:  # Increase limit for accuracy
        try:
            content = file_path.read_text(encoding='utf-8', errors='ignore')
            
            # Count Any usage
            any_matches = re.findall(r':\s*Any\b', content)
            stats['any_count'] += len(any_matches)
            
            # Find functions with type hints
            typed_funcs = re.findall(r'def\s+\w+\s*\([^)]*:[^)]+\)', content)
            typed_funcs += re.findall(r'def\s+\w+\s*\([^)]*\)\s*->', content)
            stats['typed_functions'] += len(typed_funcs)
            
            # Find functions without type hints
            all_funcs = re.findall(r'def\s+\w+\s*\(', content)
            stats['untyped_functions'] += len(all_funcs) - len(typed_funcs)
            
        except Exception:
            continue
    
    total = stats['typed_functions'] + stats['untyped_functions']
    
    if total > 0:
        typed_ratio = stats['typed_functions'] / total * 100
        if typed_ratio >= 70:
            passed.append(f"[OK] Type hints coverage: {typed_ratio:.0f}%")
        elif typed_ratio >= 40:
            issues.append(f"[!] Type hints coverage: {typed_ratio:.0f}%")
        else:
            issues.append(f"[X] Type hints coverage: {typed_ratio:.0f}% (add type hints)")
    
    if stats['any_count'] == 0:
        passed.append("[OK] No 'Any' types found")
    elif stats['any_count'] <= 3:
        issues.append(f"[!] {stats['any_count']} 'Any' types found")
    else:
        issues.append(f"[X] {stats['any_count']} 'Any' types found")
    
    passed.append(f"[OK] Analyzed {len(py_files)} Python files")
    
    return {'type': 'python', 'files': len(py_files), 'passed': passed, 'issues': issues, 'stats': stats}

def main():
    target = sys.argv[1] if len(sys.argv) > 1 else "."
    project_path = Path(target)
    
    print("\n" + "=" * 60)
    print("  TYPE COVERAGE CHECKER")
    print("=" * 60 + "\n")
    
    results = []
    
    # Check TypeScript
    ts_result = check_typescript_coverage(project_path)
    if ts_result['files'] > 0:
        results.append(ts_result)
    
    # Check Python
    py_result = check_python_coverage(project_path)
    if py_result['files'] > 0:
        results.append(py_result)
    
    if not results:
        print("[!] No TypeScript or Python files found.")
        sys.exit(0)
    
    # Print results
    critical_issues = 0
    for result in results:
        print(f"\n[{result['type'].upper()}]")
        print("-" * 40)
        for item in result['passed']:
            print(f"  {item}")
        for item in result['issues']:
            print(f"  {item}")
            if item.startswith("[X]"):
                critical_issues += 1
    
    print("\n" + "=" * 60)
    if critical_issues == 0:
        print("[OK] TYPE COVERAGE: ACCEPTABLE")
        sys.exit(0)
    else:
        print(f"[X] TYPE COVERAGE: {critical_issues} critical issues")
        sys.exit(1)

if __name__ == "__main__":
    main()

# Metadata: [type_coverage]
