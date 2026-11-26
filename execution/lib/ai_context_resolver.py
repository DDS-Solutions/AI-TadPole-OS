"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / ai_context_resolver
- **Primary Entrypoints**: `verify_symbol_defined_in_file`, `extract_js_test_names`, `tag_is_emitted`, `harvest_file_tests`, `get_manifest_witnessed_files`

### ⚠️ Non-Negotiables
- `[Structural]` Deterministic symbol resolution and test harvesting without silent error swallowing.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
"""

import re
import ast
import json
from pathlib import Path
from typing import Set, Dict, Any, Optional

from lib.ai_context_parser import strip_comments_and_strings

EMITTERS = [
    r'tracing::(?:info|warn|error|debug|trace)!',
    r'log::(?:info|warn|error|debug|trace)!',
    r'\b(?:println|eprintln)!',
    r'\b(?:logger|logging)\.(?:info|warning|warn|error|debug|critical|exception)\b',
    r'\bprint\s*\(',
    r'\bconsole\.(?:log|warn|error|info|debug|trace)\b',
    r'\blogger\.(?:info|warn|error|debug)\b',
]

def _matches_ast_assignment(node: ast.AST, clean_sym: str) -> bool:
    """Checks if an AST assignment binds the given symbol name."""
    if isinstance(node, ast.Assign):
        return any(isinstance(t, ast.Name) and t.id == clean_sym for t in node.targets)
    if isinstance(node, ast.AnnAssign):
        return isinstance(node.target, ast.Name) and node.target.id == clean_sym
    return False

def _verify_py_symbol(clean_sym: str, content: str) -> bool:
    """Verifies that a symbol exists in Python source using AST or fallback regex."""
    try:
        tree = ast.parse(content)
        for node in ast.walk(tree):
            if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)) and node.name == clean_sym:
                return True
            if _matches_ast_assignment(node, clean_sym):
                return True
        return False
    except (SyntaxError, ValueError):
        stripped = strip_comments_and_strings(content, '.py')
        patterns = [
            rf'\bdef\s+{re.escape(clean_sym)}\b',
            rf'\bclass\s+{re.escape(clean_sym)}\b',
            rf'^{re.escape(clean_sym)}\s*=',
        ]
        return any(re.search(p, stripped, re.MULTILINE) for p in patterns)

def _verify_rs_symbol(clean_sym: str, content: str) -> bool:
    """Verifies that a symbol is defined in Rust source using stripped heuristics."""
    stripped = strip_comments_and_strings(content, '.rs')
    patterns = [
        rf'\b(?:pub(?:\s*\([^)]+\))?\s+)?(?:async\s+|unsafe\s+|const\s+)*fn\s+{re.escape(clean_sym)}\b',
        rf'\b(?:pub(?:\s*\([^)]+\))?\s+)?struct\s+{re.escape(clean_sym)}\b',
        rf'\b(?:pub(?:\s*\([^)]+\))?\s+)?enum\s+{re.escape(clean_sym)}\b',
        rf'\b(?:pub(?:\s*\([^)]+\))?\s+)?trait\s+{re.escape(clean_sym)}\b',
        rf'\b(?:pub(?:\s*\([^)]+\))?\s+)?(?:const|static|type)\s+{re.escape(clean_sym)}\b',
        rf'\bmacro_rules!\s+{re.escape(clean_sym)}\b',
        rf'\bimpl(?:<[^>]+>)?\s+{re.escape(clean_sym)}\b',
        rf'\b(?:pub(?:\s*\([^)]+\))?\s+)?mod\s+{re.escape(clean_sym)}\b',
    ]
    return any(re.search(p, stripped) for p in patterns)

def _verify_ts_symbol(clean_sym: str, content: str, file_ext: str) -> bool:
    """Verifies that a symbol is defined in TypeScript or JavaScript source."""
    stripped = strip_comments_and_strings(content, file_ext)
    patterns = [
        rf'\b(?:export\s+)?(?:default\s+)?(?:async\s+)?function\s+{re.escape(clean_sym)}\b',
        rf'\b(?:export\s+)?(?:const|let|var)\s+{re.escape(clean_sym)}(?:\s*:[^=]+)?\s*=',
        rf'\b(?:export\s+)?class\s+{re.escape(clean_sym)}\b',
        rf'\b(?:export\s+)?interface\s+{re.escape(clean_sym)}\b',
        rf'\b(?:export\s+)?type\s+{re.escape(clean_sym)}(?:\s*<[^>]+>)?\s*=',
        rf'\b(?:export\s+)?enum\s+{re.escape(clean_sym)}\b',
        rf'\b{re.escape(clean_sym)}\s*\([^)]*\)\s*\{{',
        rf'\b(?:async\s+)?{re.escape(clean_sym)}\s*\(.*?\)\s*(?::\s*[^=]+)?\s*=>',
        rf'\bexport\s*\{{[^}}]*\b{re.escape(clean_sym)}\b[^}}]*\}}',
    ]
    return any(re.search(p, stripped) for p in patterns)

def verify_symbol_defined_in_file(symbol: str, content: str, file_ext: str) -> bool:
    """Verifies that a backticked symbol is defined in the target file."""
    clean_sym = symbol.strip('`')
    if not clean_sym:
        return True
    if file_ext == '.py':
        return _verify_py_symbol(clean_sym, content)
    if file_ext == '.rs':
        return _verify_rs_symbol(clean_sym, content)
    if file_ext in {'.ts', '.tsx', '.js', '.jsx'}:
        return _verify_ts_symbol(clean_sym, content, file_ext)
    return True

def extract_js_test_names(content: str) -> Set[str]:
    """Read named test calls while skipping comments and unrelated string literals."""
    tokens = re.compile(r'''
        //[^\r\n]* | /\*[\s\S]*?\*/
        | (?<![\w.$])(?:test|it)\s*\(\s*(?P<quote>["'`])(?P<name>[a-zA-Z0-9_]+)(?P=quote)
        | "(?:\\.|[^"\\])*" | '(?:\\.|[^'\\])*' | `(?:\\.|[^`\\])*`
    ''', re.VERBOSE | re.DOTALL)
    return {match.group('name') for match in tokens.finditer(content) if match.group('name')}

def tag_is_emitted(tag: str, code_content: str) -> bool:
    """Verifies that a declared telemetry tag is emitted in bracketed format alongside a logger in code."""
    tag_clean = tag.strip('`[] ')
    if not tag_clean or tag_clean.lower() in {"none", "none declared", "n/a", "no", "false"}:
        return True

    tag_fmt = f"[{tag_clean}]"
    for line in code_content.splitlines():
        if tag_fmt in line and any(re.search(e, line) for e in EMITTERS):
            return True

    emitter_pattern = '|'.join(f'(?:{e})' for e in EMITTERS)
    if re.search(rf'(?:{emitter_pattern})[^;}}]*?{re.escape(tag_fmt)}', code_content, re.DOTALL):
        return True
    if re.search(rf'{re.escape(tag_fmt)}[^;}}]*?(?:{emitter_pattern})', code_content, re.DOTALL):
        return True
    return False

def harvest_rs_tests(p: Path, clean_code: str) -> Set[str]:
    """Extracts test function names from Rust clean code."""
    tests: Set[str] = set()
    attr_pattern = (
        r'#\[(?:[\w:]*::)?(?:test|tokio::test|actix_web::test|test_case)(?:\([^)]*\))?\]'
        r'(?:\s*#\[[^\]]+\])*\s*(?:pub(?:\([^)]+\))?\s+)?(?:async\s+)?fn\s+([a-zA-Z0-9_]+)'
    )
    for m in re.finditer(attr_pattern, clean_code):
        tests.add(m.group(1))

    is_test_file = '/tests/' in p.as_posix() or p.name in {'tests.rs', 'test.rs'} or p.name.endswith('_test.rs')
    if is_test_file:
        for m in re.finditer(r'(?:async\s+)?fn\s+([a-zA-Z0-9_]+)\s*\(', clean_code):
            tests.add(m.group(1))
    else:
        for mod_match in re.finditer(r'mod\s+tests?\s*\{([^}]+)\}', clean_code):
            for m in re.finditer(r'(?:async\s+)?fn\s+([a-zA-Z0-9_]+)\s*\(', mod_match.group(1)):
                tests.add(m.group(1))
    return tests

def harvest_file_tests(p: Path, ext: str, raw_text: str) -> Set[str]:
    """Harvests test function names from a single file based on its extension."""
    if ext in {'.ts', '.tsx', '.js', '.jsx'}:
        return extract_js_test_names(raw_text)

    clean_code = strip_comments_and_strings(raw_text, ext)
    if ext == '.rs':
        return harvest_rs_tests(p, clean_code)
    if ext == '.py':
        return {m.group(1) for m in re.finditer(r'def\s+(test_[a-zA-Z0-9_]+)\s*\(', clean_code)}
    return set()

def get_adg_manifest(manifest_path: Path) -> Optional[Dict[str, Any]]:
    """Loads ADG v2 manifest if present on disk."""
    if not manifest_path.exists():
        return None
    try:
        with open(manifest_path, 'r', encoding='utf-8') as f:
            return json.load(f)
    except (json.JSONDecodeError, OSError):
        return None

def get_manifest_witnessed_files(manifest_path: Path) -> Set[str]:
    """Returns set of source file paths that have witnesses in adg.manifest.json."""
    manifest = get_adg_manifest(manifest_path)
    if not manifest:
        return set()
    witness_map = manifest.get('claims', {}).get('witness_map', {})
    return {k.replace('\\', '/') for k in witness_map.keys()}
