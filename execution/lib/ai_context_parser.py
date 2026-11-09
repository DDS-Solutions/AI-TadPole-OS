"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / ai_context_parser
- **Primary Entrypoints**: `strip_comments_and_strings`, `extract_header_block`, `extract_code_body`, `extract_metadata`

### ⚠️ Non-Negotiables
- `[Structural]` Deterministic parsing of AI context headers and code stripping across languages.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
"""

import re
from typing import Dict, List, Any, Optional

def strip_rs_comments_and_strings(text: str) -> str:
    """Strips comments and string literals from Rust source."""
    # 1. Strip Rust multi-hash raw strings r#*""*#
    text = re.sub(r'(?<![a-zA-Z0-9_])r(#*)"[\s\S]*?"\1', '""', text)
    # 2. Strip standard strings (allowing multiline strings in Rust)
    text = re.sub(r'"(?:\\.|[^"\\])*?"', '""', text, flags=re.DOTALL)
    # 3. Strip character literals
    text = re.sub(r"'(?:\\.|[^'\\])'", "''", text)
    # 4. Strip block comments
    text = re.sub(r'/\*[\s\S]*?\*/', '', text)
    # 5. Strip line comments
    return re.sub(r'//.*$', '', text, flags=re.MULTILINE)

def strip_js_comments_and_strings(text: str) -> str:
    """Strips comments and string literals from JavaScript and TypeScript source."""
    pattern = re.compile(
        r'(/\*[\s\S]*?\*/)|'          # Group 1: block comments
        r'(//[^\r\n]*)|'              # Group 2: line comments
        r'(`(?:\\.|[^`\\])*`)|'       # Group 3: template literals
        r'("(?:\\.|[^"\r\n\\])*")|'   # Group 4: double-quoted strings
        r"('(?:\\.|[^'\r\n\\])*')",   # Group 5: single-quoted strings
        re.DOTALL
    )
    def _js_replacer(m: re.Match) -> str:
        if m.group(1) or m.group(2):
            return ' '
        return '""'
    return pattern.sub(_js_replacer, text)

def strip_py_comments_and_strings(text: str) -> str:
    """Strips comments and string literals from Python source."""
    text = re.sub(r'"""[\s\S]*?"""', '""', text)
    text = re.sub(r"'''[\s\S]*?'''", "''", text)
    text = re.sub(r'#.*$', '', text, flags=re.MULTILINE)
    text = re.sub(r'"(?:\\.|[^"\r\n\\])*"', '""', text)
    return re.sub(r"'(?:\\.|[^'\r\n\\])*'", "''", text)

def strip_comments_and_strings(content: str, file_ext: str) -> str:
    """Strips string literals and comments safely according to language-specific grammar rules."""
    text = content.replace('\r\n', '\n')
    if file_ext == '.rs':
        return strip_rs_comments_and_strings(text)
    if file_ext in {'.ts', '.tsx', '.js', '.jsx'}:
        return strip_js_comments_and_strings(text)
    if file_ext == '.py':
        return strip_py_comments_and_strings(text)
    return text

def extract_header_block(content: str, file_ext: str = "") -> str:
    """Extracts only the top header block (docstring or leading comments)."""
    norm = content.replace('\r\n', '\n')
    if file_ext == '.py' or norm.strip().startswith('"""') or norm.strip().startswith("'''"):
        m = re.match(r'^(?:\s*#[^\n]*\n)*\s*(?:"""[\s\S]*?"""|\'\'\'[\s\S]*?\'\'\')', norm)
        if m:
            return m.group(0)
    elif file_ext in {'.rs', '.ts', '.tsx', '.js', '.jsx'}:
        prefix_set = ('//', '/*', '*', '*/', '#!')
        header_lines = []
        for line in norm.split('\n'):
            s = line.strip()
            if any(s.startswith(p) for p in prefix_set) or not s:
                header_lines.append(line)
            else:
                break
        return '\n'.join(header_lines)

    lines = norm.split('\n')
    return '\n'.join(lines[:120])

def extract_code_body(content: str, file_ext: str) -> str:
    """Returns the code content excluding the leading AI Context header/docstring."""
    norm = content.replace('\r\n', '\n')
    if file_ext == '.py':
        m = re.match(r'^(?:\s*#[^\n]*\n)*\s*(?:"""[\s\S]*?"""|\'\'\'[\s\S]*?\'\'\')', norm)
        if m:
            return norm[m.end():]
    elif file_ext == '.rs':
        lines = norm.split('\n')
        idx = 0
        prefix_set = ('//', '/*', '*', '*/')
        while idx < len(lines) and (any(lines[idx].strip().startswith(p) for p in prefix_set) or not lines[idx].strip()):
            idx += 1
        return '\n'.join(lines[idx:])
    elif file_ext in {'.ts', '.tsx', '.js', '.jsx'}:
        m = re.match(r'^(?:\s*#![^\n]*\n)?(?:\s*//[^\n]*\n|\s*/\*[\s\S]*?\*/\s*)+', norm)
        if m:
            return norm[m.end():]
    return norm

def parse_raw_tag_list(line_val: str) -> List[str]:
    """Parses bracketed and backticked telemetry tags from a line value."""
    if re.match(r'^(?:none\b|n/a\b|none declared\b)', line_val, re.IGNORECASE):
        return []
    bracket_tags = re.findall(r'\[([a-zA-Z0-9_:\-]+)\]', line_val)
    backtick_tags = re.findall(r'`([a-zA-Z0-9_:\-]+)`', line_val)
    tags: List[str] = []
    for t in bracket_tags + backtick_tags:
        clean = t.strip('`[] ')
        if clean and clean.lower() not in {"none", "none declared", "n/a"} and clean not in tags:
            tags.append(clean)
    return tags

def extract_telemetry_tags(clean_text: str) -> List[str]:
    """Extracts declared telemetry tags from clean header text."""
    tags: List[str] = []
    tele_section = re.search(r'-\s+\*\*Telemetry (?:Targets|Link)\*\*:\s*([^\n]+)', clean_text)
    if tele_section:
        tags.extend(parse_raw_tag_list(tele_section.group(1).strip()))

    tele_matches = re.finditer(
        r'Search (?:for |`)\[?([a-zA-Z0-9_:\-]+)\]?`? in (?:system|audit|test|log)',
        clean_text,
        re.IGNORECASE
    )
    for tm in tele_matches:
        tag = tm.group(1).strip('`[] ')
        if tag and tag.lower() not in {"none", "none declared", "n/a"} and tag not in tags:
            tags.append(tag)
    return tags

def extract_symbols(clean_text: str) -> List[str]:
    """Extracts primary entrypoint symbols from clean header text."""
    symbols: List[str] = []
    matches = re.finditer(r'-\s+\*\*(?:Primary Entrypoints|Execution Symbols|Core Symbols)\*\*:\s*([^\n]+)', clean_text)
    for sm in matches:
        for sym in re.findall(r'`([a-zA-Z0-9_]+)`', sm.group(1)):
            if sym not in symbols:
                symbols.append(sym)
    return symbols

def extract_invariants(clean_text: str) -> List[Dict[str, Any]]:
    """Extracts invariant taxonomy bullets and optional witness links from clean header text."""
    invariants: List[Dict[str, Any]] = []
    match = re.search(r'#{1,4}\s+.*?\s*Invariants\s+&\s+Non-Negotiables\s*\n(.*?)(?=\n#{1,4}|\Z)', clean_text, re.DOTALL)
    if not match:
        return invariants

    for b in re.split(r'\n(?=-\s+)', match.group(1)):
        b_str = b.strip()
        if not b_str.startswith('-'):
            continue
        tag_match = re.search(r'-\s+`?\[(Structural|Behavioral|Advisory:\s*UNVERIFIED)\]`?', b_str)
        tag = tag_match.group(1) if tag_match else None
        enf_match = re.search(r'enforced_by:\s*`?([a-zA-Z0-9_]+)`?', b_str)
        witness = enf_match.group(1) if enf_match else None
        invariants.append({
            "tag": tag,
            "text": b_str.split('\n')[0],
            "witness": witness,
        })
    return invariants

def extract_witness_tests(clean_text: str) -> List[str]:
    """Extracts witness test names from clean header text."""
    witness_match = re.search(r'-\s+\*\*Witness Tests\*\*:\s*([^\n]+)', clean_text)
    if not witness_match:
        return []
    witness_tests: List[str] = []
    for test in re.findall(r'`([a-zA-Z0-9_]+)`', witness_match.group(1)):
        if test not in witness_tests:
            witness_tests.append(test)
    return witness_tests

def extract_metadata(content: str, file_ext: str = "") -> Dict[str, Any]:
    """Extracts structured metadata and verification targets from header comments or docstrings."""
    header_raw = extract_header_block(content, file_ext)
    clean_lines = [re.sub(r'^\s*(?://[!/]?|\*(?!/)|#(?!##))\s?', '', line) for line in header_raw.split('\n')]
    clean_text = '\n'.join(clean_lines)

    docs_match = re.search(r'@docs\s+([A-Za-z0-9_:\-]+)', clean_text)
    return {
        "has_note": bool(re.search(r'#{1,4}\s+AI\s+(?:Assist\s+Note|Context\s+Alignment)', clean_text, re.IGNORECASE)),
        "has_debugging": bool(re.search(r'#{1,4}\s+.*?\s*Debugging\s+&\s+Observability', clean_text, re.IGNORECASE)),
        "has_docs": bool(docs_match),
        "has_invariants": bool(re.search(r'#{1,4}\s+.*?\s*Invariants\s+&\s+Non-Negotiables', clean_text, re.IGNORECASE)),
        "telemetry_tags": extract_telemetry_tags(clean_text),
        "docs_link": docs_match.group(1) if docs_match else None,
        "symbols": extract_symbols(clean_text),
        "invariants": extract_invariants(clean_text),
        "witness_tests": extract_witness_tests(clean_text),
        "local_errors": [],
    }
