"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / migrate_ai_headers
- **Primary Entrypoints**: `detect_emitted_tags`, `extract_primary_symbols`, `derive_subsystem`, `derive_docs_link`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Invariant taxonomy enforcement: bullets must be marked [Structural], [Behavioral], or [Advisory: UNVERIFIED].
  - enforced_by: `test_invariant_taxonomy_enforcement`
- `[Behavioral]` Behavioral invariants must provide a resolvable `enforced_by` witness test in the workspace test suite.
  - enforced_by: `test_behavioral_witness_enforcement`
- `[Structural]` Backticked entrypoint symbols must resolve to concrete definitions in the target file.
  - enforced_by: `test_symbol_definition_resolution`
- `[Structural]` Telemetry tags declared in header must be emitted via valid logging/printing constructs in the code body.
  - enforced_by: `test_telemetry_emitter_verification`

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[migrate_ai_headers]`
- **Witness Tests**: `test_invariant_taxonomy_enforcement`, `test_behavioral_witness_enforcement`, `test_symbol_definition_resolution`, `test_telemetry_emitter_verification`
"""

#!/usr/bin/env python3
"""
🛡️ Tadpole OS: AI Context Header Migration Engine (Gold Standard Upgrade)
Purpose: Elevates all codebase headers to the rich AI Context Contract v1.
"""

import os
import re
import ast
import sys
import argparse
from pathlib import Path
from typing import List, Dict, Any, Optional, Set, Tuple

ROOT = Path(__file__).resolve().parent.parent
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))

from execution.verify_ai_context import (
    EXTENSIONS,
    SKIP_DIRS,
    EMITTERS,
    extract_metadata,
    extract_code_body,
    extract_header_block,
    strip_comments_and_strings,
    verify_symbol_defined_in_file,
    verify_file,
    get_workspace_test_symbols,
)

def detect_emitted_tags(code_body: str) -> List[str]:
    """Finds all bracketed tags that actually co-occur with log emitters in the code body."""
    emitted = []
    for line in code_body.splitlines():
        if any(re.search(e, line) for e in EMITTERS):
            tags = re.findall(r'\[([a-zA-Z0-9_:\-]+)\]', line)
            for t in tags:
                t_clean = t.strip('`[] ')
                if t_clean and t_clean.lower() not in {"none", "none declared", "n/a"} and t_clean not in emitted:
                    emitted.append(t_clean)
    return emitted

def extract_primary_symbols(code_body: str, file_path: Path) -> List[str]:
    """Extracts defined top-level symbols (functions, classes, components, structs) from the code body."""
    file_ext = file_path.suffix.lower()
    symbols = []

    if file_ext == '.py':
        try:
            tree = ast.parse(code_body)
            for node in ast.iter_child_nodes(tree):
                if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
                    if not node.name.startswith('_') or node.name == '__init__':
                        symbols.append(node.name)
        except Exception:
            for match in re.finditer(r'^(?:async\s+)?def\s+([a-zA-Z0-9_]+)', code_body, re.MULTILINE):
                symbols.append(match.group(1))
            for match in re.finditer(r'^class\s+([a-zA-Z0-9_]+)', code_body, re.MULTILINE):
                symbols.append(match.group(1))

    elif file_ext in {'.ts', '.tsx', '.js', '.jsx'}:
        # Look for exported functions, consts, classes, interfaces
        patterns = [
            r'export\s+(?:default\s+)?(?:async\s+)?function\s+([a-zA-Z0-9_]+)',
            r'export\s+const\s+([a-zA-Z0-9_]+)',
            r'export\s+class\s+([a-zA-Z0-9_]+)',
            r'export\s+interface\s+([a-zA-Z0-9_]+)',
            r'export\s+type\s+([a-zA-Z0-9_]+)',
            r'export\s+enum\s+([a-zA-Z0-9_]+)',
        ]
        for pat in patterns:
            for m in re.finditer(pat, code_body):
                sym = m.group(1)
                if sym not in symbols and sym not in {"Props", "State", "default"}:
                    symbols.append(sym)

        # If no exported symbol found, look for top-level declarations matching file stem
        if not symbols:
            stem = file_path.stem
            if re.search(r'\b(?:const|function|class)\s+' + re.escape(stem) + r'\b', code_body):
                symbols.append(stem)

    elif file_ext == '.rs':
        patterns = [
            r'pub(?:\([^)]+\))?\s+(?:async\s+)?fn\s+([a-zA-Z0-9_]+)',
            r'pub(?:\([^)]+\))?\s+struct\s+([a-zA-Z0-9_]+)',
            r'pub(?:\([^)]+\))?\s+enum\s+([a-zA-Z0-9_]+)',
            r'pub(?:\([^)]+\))?\s+trait\s+([a-zA-Z0-9_]+)',
        ]
        for pat in patterns:
            for m in re.finditer(pat, code_body):
                sym = m.group(1)
    # Filter to only symbols that strictly pass definition verification
    verified_symbols = [s for s in symbols if verify_symbol_defined_in_file(s, code_body, file_ext)]
    # Limit to top 4 most representative symbols
    return verified_symbols[:4]

def derive_subsystem(file_path: Path) -> str:
    """Derives a human-readable subsystem label from the file path."""
    rel = file_path.relative_to(ROOT).as_posix()
    stem = file_path.stem

    if 'src/components' in rel:
        parts = Path(rel).parts
        category = parts[2] if len(parts) > 3 else "General"
        return f"UI Components / {category.title()} / {stem}"
    elif 'src/pages' in rel:
        return f"UI Pages / {stem}"
    elif 'src/stores' in rel:
        return f"Frontend State Store / {stem}"
    elif 'src/services' in rel:
        return f"Frontend Service Layer / {stem}"
    elif 'src/hooks' in rel:
        return f"Frontend React Hooks / {stem}"
    elif 'src/utils' in rel:
        return f"Frontend Utilities / {stem}"
    elif 'server-rs/src/routes' in rel:
        return f"Sovereign Engine / HTTP Routes / {stem}"
    elif 'server-rs/src/services' in rel:
        return f"Sovereign Engine / Core Services / {stem}"
    elif 'server-rs/src/security' in rel:
        return f"Sovereign Engine / Security & Governance / {stem}"
    elif 'server-rs/src/agent' in rel:
        return f"Sovereign Engine / Agent Runner / {stem}"
    elif 'server-rs/src/db' in rel:
        return f"Sovereign Engine / Database & Migrations / {stem}"
    elif 'server-rs' in rel:
        return f"Sovereign Engine / {stem}"
    elif 'execution' in rel:
        return f"Infrastructure Automation / {stem}"
    elif 'scripts' in rel:
        return f"Developer Scripts / {stem}"
    elif 'tests' in rel:
        return f"Test Verification Suite / {stem}"
    elif 'docs' in rel:
        return f"Documentation System / {stem}"
    else:
        return f"System Core / {stem}"

def derive_docs_link(file_path: Path) -> str:
    """Derives the appropriate @docs link."""
    rel = file_path.relative_to(ROOT).as_posix()
    if 'src/components' in rel:
        return "ARCHITECTURE:UI-Components"
    elif 'src/pages' in rel or 'src/layouts' in rel:
        return "ARCHITECTURE:UI"
    elif 'src/stores' in rel:
        return "ARCHITECTURE:State"
    elif 'src/services' in rel or 'src/api' in rel:
        return "ARCHITECTURE:Services"
    elif 'server-rs/src/security' in rel:
        return "SECURITY:Core"
    elif 'server-rs/src/agent' in rel or 'server-rs/src/runner' in rel:
        return "ARCHITECTURE:Runner"
    elif 'server-rs/src/db' in rel or 'server-rs/src/persistence' in rel:
        return "ARCHITECTURE:Persistence"
    elif 'server-rs' in rel:
        return "ARCHITECTURE:Server"
    elif 'execution' in rel or 'scripts' in rel:
        return "ARCHITECTURE:Infrastructure:Execution"
    elif 'tests' in rel:
        return "ARCHITECTURE:Quality:Verification"
    else:
        return "ARCHITECTURE:Core"

def generate_invariants_for_file(file_path: Path) -> List[str]:
    """Generates meaningful structural invariants based on module type."""
    rel = file_path.relative_to(ROOT).as_posix()
    if 'src/components' in rel or 'src/pages' in rel:
        return [
            "- `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings."
        ]
    elif 'src/stores' in rel:
        return [
            "- `[Structural]` Store mutations maintain immutable state transitions and notify subscribers deterministically."
        ]
    elif 'src/services' in rel or 'src/api' in rel:
        return [
            "- `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors."
        ]
    elif 'src/hooks' in rel:
        return [
            "- `[Structural]` React hook lifecycle adheres to Rules of Hooks without conditional execution branches."
        ]
    elif 'server-rs' in rel:
        return [
            "- `[Structural]` Type-safe state handling and bounded execution without unhandled panics."
        ]
    elif 'execution' in rel or 'scripts' in rel:
        return [
            "- `[Structural]` Deterministic execution without side effects outside declared scope."
        ]
    else:
        return [
            "- `[Structural]` Deterministic internal state integrity and strict interface contract compliance."
        ]

def fix_file_header(content: str, file_path: Path) -> Tuple[str, bool]:
    """Upgrades the header in content to the full Gold Standard AI Context Contract."""
    file_ext = file_path.suffix.lower()
    norm = content.replace('\r\n', '\n')
    meta = extract_metadata(norm, file_ext)
    code_body = extract_code_body(norm, file_ext)

    real_emitted = detect_emitted_tags(code_body)
    primary_syms = extract_primary_symbols(code_body, file_path)
    subsystem = derive_subsystem(file_path)
    doc_link = meta["docs_link"] or derive_docs_link(file_path)
    telemetry_target = ', '.join([f"`[{t}]`" for t in real_emitted]) if real_emitted else "none declared"

    # Build gold-standard header lines
    header_comment_type = "slash_star" if file_ext in {'.ts', '.tsx', '.js', '.jsx'} else ("inner_doc" if file_ext == '.rs' else "py_docstring")

    # Invariants
    inv_lines = []
    if meta["invariants"]:
        for inv in meta["invariants"]:
            tag = inv.get("tag", "Structural")
            txt = inv.get("text", "")
            # Ensure proper tag prefix
            if not txt.startswith(f"- `[{tag}]`") and not txt.startswith(f"- [{tag}]"):
                txt = re.sub(r'-\s+(?:`?\[[^\]]+\]`?\s*)?', f'- `[{tag}]` ', txt, count=1)
            inv_lines.append(txt)
            if inv.get("witness"):
                inv_lines.append(f"  - enforced_by: `{inv['witness']}`")
    else:
        inv_lines = generate_invariants_for_file(file_path)

    # Entrypoints line
    entrypoints_line = f"- **Primary Entrypoints**: {', '.join([f'`{s}`' for s in primary_syms])}" if primary_syms else None

    # Witness tests
    witness_line = f"- **Witness Tests**: {', '.join([f'`{w}`' for w in meta['witness_tests']])}" if meta['witness_tests'] else "- **Witness Tests**: none declared"

    # Assemble header text
    if header_comment_type == "slash_star":
        hdr_lines = [
            "/**",
            f" * @docs {doc_link}",
            " *",
            " * ### AI Context Alignment",
            f" * - **Subsystem**: {subsystem}",
        ]
        if entrypoints_line:
            hdr_lines.append(f" * {entrypoints_line}")
        hdr_lines.extend([
            " *",
            " * ### ⚠️ Invariants & Non-Negotiables",
        ])
        for il in inv_lines:
            hdr_lines.append(f" * {il}")
        hdr_lines.extend([
            " *",
            " * ### 🔍 Debugging & Observability",
            " * - **Local Errors**: none",
            f" * - **Telemetry Targets**: {telemetry_target}",
            f" * {witness_line}",
            " */"
        ])
        new_header = '\n'.join(hdr_lines)

    elif header_comment_type == "inner_doc":
        hdr_lines = [
            f"//! @docs {doc_link}",
            "//!",
            "//! ### AI Context Alignment",
            f"//! - **Subsystem**: {subsystem}",
        ]
        if entrypoints_line:
            hdr_lines.append(f"//! {entrypoints_line}")
        hdr_lines.extend([
            "//!",
            "//! ### ⚠️ Invariants & Non-Negotiables",
        ])
        for il in inv_lines:
            hdr_lines.append(f"//! {il}")
        hdr_lines.extend([
            "//!",
            "//! ### 🔍 Debugging & Observability",
            "//! - **Local Errors**: none",
            f"//! - **Telemetry Targets**: {telemetry_target}",
            f"//! {witness_line}",
        ])
        new_header = '\n'.join(hdr_lines)

    else: # py_docstring
        hdr_lines = [
            '"""',
            f"@docs {doc_link}",
            "",
            "### AI Context Alignment",
            f"- **Subsystem**: {subsystem}",
        ]
        if entrypoints_line:
            hdr_lines.append(f"{entrypoints_line}")
        hdr_lines.extend([
            "",
            "### ⚠️ Invariants & Non-Negotiables",
        ])
        for il in inv_lines:
            hdr_lines.append(f"{il}")
        hdr_lines.extend([
            "",
            "### 🔍 Debugging & Observability",
            "- **Local Errors**: none",
            f"- **Telemetry Targets**: {telemetry_target}",
            f"{witness_line}",
            '"""'
        ])
        new_header = '\n'.join(hdr_lines)

    # Strip existing header and footers from code body
    stripped_body = code_body.strip()
    # Remove legacy footer comment like `// Metadata: [...]` or `# Metadata: [...]`
    stripped_body = re.sub(r'(?:\r?\n)?(?://|#)\s*Metadata:\s*\[[^\]]+\]\s*$', '', stripped_body).strip()

    if file_ext == '.py' and norm.startswith('#!'):
        shebang = norm.split('\n', 1)[0]
        final_content = shebang + '\n' + new_header + '\n\n' + stripped_body + '\n'
    else:
        final_content = new_header + '\n\n' + stripped_body + '\n'

    was_modified = (final_content.strip() != norm.strip())
    return final_content, was_modified

def migrate_file(file_path: Path, dry_run: bool = False) -> Dict[str, Any]:
    try:
        with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
            content = f.read()

        new_content, modified = fix_file_header(content, file_path)

        if modified and not dry_run:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(new_content)

        return {
            "file": str(file_path.relative_to(ROOT)),
            "modified": modified,
            "error": None
        }
    except Exception as e:
        return {
            "file": str(file_path),
            "modified": False,
            "error": str(e)
        }

def main():
    if sys.platform == "win32":
        import io
        sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
        sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8')

    parser = argparse.ArgumentParser(description="Tadpole OS AI Context Header Migration Engine (Gold Standard)")
    parser.add_argument("paths", nargs="*", default=["src", "apps", "plugins", "server-rs", "execution", "scripts", "tests"], help="Paths to migrate")
    parser.add_argument("--dry-run", action="store_true", help="Preview changes without writing to disk")
    args = parser.parse_args()

    results = []
    print(f"[migrate_ai_headers] Starting Gold Standard migration on: {args.paths} (dry_run={args.dry_run})")

    for path_str in args.paths:
        target = Path(path_str).resolve()
        if target.is_file():
            if target.suffix.lower() in EXTENSIONS:
                results.append(migrate_file(target, dry_run=args.dry_run))
        else:
            for root, dirs, files in os.walk(target):
                dirs[:] = [d for d in dirs if d not in SKIP_DIRS]
                for file in files:
                    file_path = Path(root) / file
                    if file_path.suffix.lower() in EXTENSIONS:
                        results.append(migrate_file(file_path, dry_run=args.dry_run))

    modified_files = [r for r in results if r["modified"]]
    print(f"[migrate_ai_headers] Processed {len(results)} file(s). Upgraded to Gold Standard: {len(modified_files)}")

if __name__ == "__main__":
    main()
