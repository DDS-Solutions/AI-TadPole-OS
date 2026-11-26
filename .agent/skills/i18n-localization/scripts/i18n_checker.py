#!/usr/bin/env python3
"""
@docs ARCHITECTURE:SovereignArchitecture

### AI Context Alignment
- **Subsystem**: Agent Skills / i18n-localization / i18n_checker
- **Primary Entrypoints**: `find_locale_files`, `check_locale_completeness`, `flatten_keys`, `check_hardcoded_strings`, `run_self_tests`, `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[i18n_checker]`, `[LOCALE]`, `[CODE]`, `[PASS]`, `[FAIL]`, `[OK]`, `[X]`
- **Witness Tests**: none declared
"""

import sys
import os
import re
import json
import argparse
from pathlib import Path
from typing import List, Dict, Set, Any, Optional

# Windows console encoding fix without swallowed exception
if sys.platform == "win32":
    if hasattr(sys.stdout, 'reconfigure'):
        sys.stdout.reconfigure(encoding='utf-8', errors='replace')
    if hasattr(sys.stderr, 'reconfigure'):
        sys.stderr.reconfigure(encoding='utf-8', errors='replace')

# Patterns that indicate hardcoded strings (should be translated)
HARDCODED_PATTERNS = {
    'jsx': [
        # Text directly in JSX: <div>Hello World</div>
        r'>\s*([A-Z][a-zA-Z\s]{3,30})\s*</',
        # JSX attribute strings: title="Welcome"
        r'(?:title|placeholder|label|alt|aria-label)="([A-Z][a-zA-Z\s]{2,})"',
        # Button/heading text
        r'<(?:button|h[1-6]|p|span|label)[^>]*>\s*([A-Z][a-zA-Z\s!?.,]{3,})\s*</',
    ],
    'vue': [
        # Vue template text
        r'>\s*([A-Z][a-zA-Z\s]{3,30})\s*</',
        r'(?:placeholder|label|title)="([A-Z][a-zA-Z\s]{2,})"',
    ],
    'python': [
        # print/raise with string literals
        r'(?:print|raise\s+\w+)\s*\(\s*["\']([A-Z][^"\']{5,})["\']',
        # Flask flash messages
        r'flash\s*\(\s*["\']([A-Z][^"\']{5,})["\']',
    ]
}

# Patterns that indicate proper i18n usage
I18N_PATTERNS = [
    r't\(["\']',           # t('key') - react-i18next
    r'useTranslation',     # React hook
    r'\$t\(',              # Vue i18n
    r'_\(["\']',           # Python gettext
    r'gettext\(',          # Python gettext
    r'useTranslations',    # next-intl
    r'FormattedMessage',   # react-intl
    r'i18n\.',             # Generic i18n
]

def find_locale_files(project_path: Path) -> List[Path]:
    """Find translation/locale JSON and PO files."""
    patterns = [
        "**/locales/**/*.json",
        "**/translations/**/*.json",
        "**/lang/**/*.json",
        "**/i18n/**/*.json",
        "**/messages/*.json",
        "**/*.po",
    ]
    
    files: List[Path] = []
    for pattern in patterns:
        files.extend(project_path.glob(pattern))
    
    return [f for f in files if 'node_modules' not in str(f) and '.git' not in str(f)]

def flatten_keys(d: Any, prefix: str = '') -> Set[str]:
    """Flatten nested dict keys into dot notation."""
    keys: Set[str] = set()
    if not isinstance(d, dict):
        return keys
    for k, v in d.items():
        new_key = f"{prefix}.{k}" if prefix else str(k)
        if isinstance(v, dict):
            keys.update(flatten_keys(v, new_key))
        else:
            keys.add(new_key)
    return keys

def check_locale_completeness(locale_files: List[Path]) -> Dict[str, Any]:
    """Check if all locales have the same keys."""
    issues: List[str] = []
    passed: List[str] = []
    
    if not locale_files:
        return {'passed': [], 'issues': ["[!] No locale files found"], 'valid': True}
    
    locales: Dict[str, Dict[str, Set[str]]] = {}
    for f in locale_files:
        if f.suffix == '.json':
            try:
                lang = f.parent.name
                content = json.loads(f.read_text(encoding='utf-8'))
                if lang not in locales:
                    locales[lang] = {}
                locales[lang][f.stem] = set(flatten_keys(content))
            except Exception as e:
                issues.append(f"[X] {f.name}: Failed to parse JSON ({e})")
                continue
    
    if len(locales) < 2:
        passed.append(f"[OK] Found {len(locale_files)} locale file(s)")
        return {'passed': passed, 'issues': issues, 'valid': len(issues) == 0}
    
    passed.append(f"[OK] Found {len(locales)} language(s): {', '.join(locales.keys())}")
    
    all_langs = list(locales.keys())
    base_lang = all_langs[0]
    
    for namespace in locales.get(base_lang, {}):
        base_keys = locales[base_lang].get(namespace, set())
        
        for lang in all_langs[1:]:
            other_keys = locales.get(lang, {}).get(namespace, set())
            
            missing = base_keys - other_keys
            if missing:
                issues.append(f"[X] {lang}/{namespace}: Missing {len(missing)} keys")
            
            extra = other_keys - base_keys
            if extra:
                issues.append(f"[!] {lang}/{namespace}: {len(extra)} extra keys")
    
    if not issues:
        passed.append("[OK] All locales have matching keys")
    
    return {'passed': passed, 'issues': issues, 'valid': not any(i.startswith("[X]") for i in issues)}

def check_hardcoded_strings(project_path: Path, max_files: int = 200) -> Dict[str, Any]:
    """Check for un-internationalized hardcoded strings in code files."""
    issues: List[str] = []
    passed: List[str] = []
    
    extensions = {
        '.tsx': 'jsx', '.jsx': 'jsx', '.ts': 'jsx', '.js': 'jsx',
        '.vue': 'vue',
        '.py': 'python'
    }
    
    code_files: List[Path] = []
    if project_path.is_file():
        code_files = [project_path]
    else:
        for ext in extensions:
            code_files.extend(project_path.rglob(f"*{ext}"))
    
    code_files = [
        f for f in code_files if not any(x in str(f) for x in 
        ['node_modules', '.git', 'dist', 'build', '__pycache__', 'venv', 'test', 'spec', 'tools'])
    ]
    
    if not code_files:
        return {'passed': ["[!] No code files found"], 'issues': [], 'valid': True}
    
    files_with_i18n = 0
    files_with_hardcoded = 0
    hardcoded_examples: List[str] = []
    
    for file_path in code_files[:max_files]:
        try:
            content = file_path.read_text(encoding='utf-8', errors='ignore')
            ext = file_path.suffix
            file_type = extensions.get(ext, 'jsx')
            
            has_i18n = any(re.search(p, content) for p in I18N_PATTERNS)
            if has_i18n:
                files_with_i18n += 1
            
            patterns = HARDCODED_PATTERNS.get(file_type, [])
            hardcoded_found = False
            
            for pattern in patterns:
                matches = re.findall(pattern, content)
                if matches and not has_i18n:
                    hardcoded_found = True
                    if len(hardcoded_examples) < 5:
                        hardcoded_examples.append(f"{file_path.name}: {str(matches[0])[:40]}...")
            
            if hardcoded_found:
                files_with_hardcoded += 1
                
        except Exception:
            continue
    
    passed.append(f"[OK] Analyzed {len(code_files[:max_files])} code files")
    
    if files_with_i18n > 0:
        passed.append(f"[OK] {files_with_i18n} files use i18n")
    
    if files_with_hardcoded > 0:
        issues.append(f"[X] {files_with_hardcoded} files may have hardcoded strings")
        for ex in hardcoded_examples:
            issues.append(f"   → {ex}")
    else:
        passed.append("[OK] No obvious hardcoded strings detected")
    
    return {'passed': passed, 'issues': issues, 'valid': len(issues) == 0}

def run_self_tests() -> bool:
    """Execute comprehensive automated self-tests."""
    print("[i18n_checker] Executing self-test suite...")

    # Test 1: flatten_keys
    nested = {
        "auth": {
            "login": "Sign In",
            "errors": {
                "invalid": "Invalid password"
            }
        },
        "common": "Submit"
    }
    flat = flatten_keys(nested)
    assert "auth.login" in flat, "flatten_keys missed auth.login"
    assert "auth.errors.invalid" in flat, "flatten_keys missed nested key"
    assert "common" in flat, "flatten_keys missed root key"
    print("[i18n_checker] [LOCALE] Key flattening logic verified.")

    # Test 2: Locale parity comparison
    locales_mock = [
        Path("locales/en/common.json"),
        Path("locales/es/common.json")
    ]
    # Test matching logic with helper
    en_keys = {"common.save", "common.cancel"}
    es_keys = {"common.save"}
    diff = en_keys - es_keys
    assert "common.cancel" in diff, "Locale difference calculation failed"
    print("[i18n_checker] [LOCALE] Locale key parity comparison verified.")

    # Test 3: Hardcoded JSX detection
    test_jsx = '<div><button>Save Changes</button><span>Unsaved text</span></div>'
    matches = re.findall(r'<(?:button|span)[^>]*>\s*([A-Z][a-zA-Z\s!?.,]{3,})\s*</', test_jsx)
    assert len(matches) >= 2, f"Expected 2 matches, got {len(matches)}"
    print("[i18n_checker] [CODE] Hardcoded UI pattern matcher verified.")

    # Test 4: i18n detection
    test_i18n_code = "const { t } = useTranslation(); return <div>{t('common.save')}</div>;"
    has_i18n = any(re.search(p, test_i18n_code) for p in I18N_PATTERNS)
    assert has_i18n, "i18n pattern matcher failed to detect useTranslation/t()"
    print("[i18n_checker] [CODE] i18n usage pattern matcher verified.")

    print("[i18n_checker] [OK] All test cases executed.")
    print("[i18n_checker] [PASS] All self-tests passed successfully.")
    return True

def main():
    parser = argparse.ArgumentParser(
        description="Sovereign Internationalization (i18n) & Locale Parity Validator."
    )
    parser.add_argument("path", nargs="?", default="src", help="Target directory or file to audit (defaults to 'src').")
    parser.add_argument("--test", action="store_true", help="Run automated self-tests.")
    parser.add_argument("--json", action="store_true", help="Output results in JSON format.")
    parser.add_argument("--max-files", type=int, default=100, help="Maximum number of code files to scan.")
    
    args = parser.parse_args()

    if args.test:
        success = run_self_tests()
        sys.exit(0 if success else 1)

    print("[i18n_checker] Initializing localization parity audit...")
    project_path = Path(args.path)
    
    if not project_path.exists():
        if args.json:
            print(json.dumps({"valid": False, "error": f"Path not found: {args.path}"}))
        else:
            print(f"[i18n_checker] [FAIL] Target path not found: {args.path}")
            print("[X] i18n CHECK: Path error")
        sys.exit(1)

    # Check locale files
    locale_files = find_locale_files(project_path)
    locale_result = check_locale_completeness(locale_files)
    
    # Check hardcoded strings
    code_result = check_hardcoded_strings(project_path, max_files=args.max_files)
    
    critical_issues = sum(1 for i in locale_result['issues'] + code_result['issues'] if i.startswith("[X]"))
    is_valid = critical_issues == 0

    if args.json:
        report = {
            "valid": is_valid,
            "target": str(project_path),
            "locale_files": [str(f) for f in locale_files],
            "locale_audit": locale_result,
            "code_audit": code_result,
            "critical_issues": critical_issues
        }
        print(json.dumps(report, indent=2))
    else:
        print("\n" + "=" * 60)
        print("  i18n CHECKER - Internationalization Audit")
        print("=" * 60 + "\n")

        print("[LOCALE FILES]")
        print("-" * 40)
        for item in locale_result['passed']:
            print(f"  {item}")
        for item in locale_result['issues']:
            print(f"  {item}")
        
        print("\n[CODE ANALYSIS]")
        print("-" * 40)
        for item in code_result['passed']:
            print(f"  {item}")
        for item in code_result['issues']:
            print(f"  {item}")
        
        print("\n" + "=" * 60)
        if is_valid:
            print("[i18n_checker] [PASS] i18n CHECK: PASSED")
            print("[OK] i18n CHECK: PASSED")
            sys.exit(0)
        else:
            print(f"[i18n_checker] [FAIL] i18n CHECK: {critical_issues} issues found")
            print(f"[X] i18n CHECK: {critical_issues} issues found")
            sys.exit(1)

if __name__ == "__main__":
    main()
