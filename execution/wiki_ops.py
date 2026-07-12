#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Assist Note
**Wiki Operations Suite (wiki_ops)**: Manages automated indexing, logging, and 
linting of the Tadpole OS local wiki (docs/wiki/). Implements the core 
operations of the "LLM Wiki" design pattern.

### 🔍 Debugging & Observability
- **Failure Path**: Broken links, orphan pages, or stale code-symbol references.
- **Trace Scope**: `execution::wiki_ops`
"""

import os
import re
import sys
import json
from pathlib import Path
from datetime import datetime
from typing import Dict, List, Set, Tuple

# Shared utilities
from py_utils import init_utf8, Colors

# Initialize unbuffered UTF-8 streams for Windows console hosts
init_utf8()

def parse_frontmatter(content: str) -> Dict[str, str]:
    """Parse basic YAML frontmatter from markdown content."""
    metadata = {}
    if content.startswith("---"):
        parts = content.split("---", 2)
        if len(parts) >= 3:
            fm_text = parts[1]
            for line in fm_text.splitlines():
                if ":" in line:
                    k, v = line.split(":", 1)
                    metadata[k.strip()] = v.strip().strip('"').strip("'")
    return metadata

def is_valid_code_identifier(s: str) -> bool:
    """Check if string looks like a programming identifier."""
    if not s or len(s) > 100:
        return False
    # Alphanumeric, underscores, colons, dots, dashes
    if not (s[0].isalpha() or s[0] == '_'):
        return False
    for c in s[1:]:
        if not (c.isalnum() or c in ('_', ':', '.', '-')):
            return False
    return True

def get_wiki_files(wiki_dir: Path) -> List[Path]:
    """Recursively list all wiki markdown files."""
    files = []
    for root, _, filenames in os.walk(wiki_dir):
        for f in filenames:
            if f.endswith(".md"):
                path = Path(root) / f
                # Skip helper files, sidebar, footer, indexes, and logs
                if f in ("_Sidebar.md", "_Footer.md", "index.md", "log.md"):
                    continue
                files.append(path)
    return files

def generate_index(wiki_dir: Path):
    """Scan all wiki files and build docs/wiki/index.md."""
    print(f"📖 {Colors.BOLD}Generating Wiki Index...{Colors.ENDC}")
    files = get_wiki_files(wiki_dir)
    
    # Categories based on Tier frontmatter or directories
    categories = {
        "1": ("📋 Tier 1: Business Operations", []),
        "2": ("⚙️ Tier 2: Systems Administration", []),
        "3": ("🔒 Tier 3: Security & Observability", []),
        "general": ("🛠️ General Documentation & Appendix", [])
    }
    
    for f in files:
        rel_path = f.relative_to(wiki_dir).as_posix()
        stem = f.stem
        
        try:
            content = f.read_text(encoding='utf-8')
        except Exception as e:
            print(f"{Colors.RED}[FAIL] Could not read {rel_path}: {e}{Colors.ENDC}")
            continue
            
        meta = parse_frontmatter(content)
        title = meta.get("title", stem.replace("-", " "))
        tier = meta.get("tier", "")
        status = meta.get("status", "draft")
        
        # Simple description parser (first paragraph after frontmatter)
        desc = ""
        lines = content.split("---", 2)
        body = lines[2] if len(lines) >= 3 else content
        for paragraph in body.split("\n\n"):
            cleaned = paragraph.strip()
            if cleaned and not cleaned.startswith("#") and not cleaned.startswith(">"):
                # Clean links and markdown formatting from summary
                cleaned = re.sub(r'\[\[([^\]|]+)(?:\|[^\]]+)?\]\]', r'\1', cleaned)
                cleaned = re.sub(r'\[([^\]]+)\]\([^\)]+\)', r'\1', cleaned)
                cleaned = cleaned.replace("`", "").replace("*", "").replace("_", "")
                desc = cleaned if len(cleaned) <= 150 else cleaned[:147] + "..."
                break
                
        entry = {
            "title": title,
            "stem": stem,
            "path": rel_path,
            "status": status,
            "desc": desc
        }
        
        if tier in categories:
            categories[tier][1].append(entry)
        else:
            categories["general"][1].append(entry)
            
    # Sort categories alphabetically
    for tier in categories:
        categories[tier][1].sort(key=lambda x: x["title"])
        
    index_content = []
    index_content.append("# 📖 Tadpole OS Wiki Index\n")
    index_content.append("> **Auto-generated Catalog of the Sovereign Wiki.**\n")
    index_content.append("This catalog is updated automatically to prevent navigation drift. Grouped by operational layer:\n")
    
    for tier in ["1", "2", "3", "general"]:
        cat_title, entries = categories[tier]
        if entries:
            index_content.append(f"\n## {cat_title}\n")
            for e in entries:
                badge = f" `[{e['status']}]`" if e['status'] != 'verified' else ""
                index_content.append(f"- **[[{e['stem']}]]**{badge} — {e['desc']}\n")
                
    index_content.append("\n---\n*Last Updated: {} | [Home]([[Home]]) | [Main Repo](https://github.com/DDS-Solutions/AI-TadPole-OS)*\n".format(datetime.now().strftime("%Y-%m-%d %H:%M:%S")))
    
    index_path = wiki_dir / "index.md"
    index_path.write_text("".join(index_content), encoding='utf-8')
    print(f"{Colors.GREEN}[OK]{Colors.ENDC} Wrote {index_path.name}")
    log_event(wiki_dir, "index", f"rebuilt index.md with {len(files)} pages")

def log_event(wiki_dir: Path, event_type: str, details: str):
    """Append a chronological log entry to docs/wiki/log.md."""
    log_path = wiki_dir / "log.md"
    date_str = datetime.now().strftime("%Y-%m-%d")
    entry = f"## [{date_str}] {event_type} | {details}\n"
    
    if not log_path.exists():
        log_path.write_text("# 📅 Tadpole OS Wiki Activity Log\n\n", encoding='utf-8')
        
    try:
        content = log_path.read_text(encoding='utf-8')
        # Check if identical event logged today to prevent duplicate runs in short window
        if entry not in content:
            log_path.write_text(content + entry, encoding='utf-8')
    except Exception as e:
        print(f"{Colors.YELLOW}[WARN] Could not write to log.md: {e}{Colors.ENDC}")

def lint_wiki(wiki_dir: Path, project_root: Path, strict: bool) -> bool:
    """Lint the wiki for broken links, orphan pages, and stale code-symbol references."""
    print(f"🔍 {Colors.BOLD}Linting Wiki Quality...{Colors.ENDC}")
    files = get_wiki_files(wiki_dir)
    
    # Check if symbol graph is stale compared to wiki files
    symbol_graph_path = project_root / "reports" / "intelligence" / "symbol_graph.json"
    if symbol_graph_path.exists():
        try:
            graph_mtime = symbol_graph_path.stat().st_mtime
            most_recent_md_time = 0
            for f in files:
                if f.name not in ("index.md", "log.md"):
                    most_recent_md_time = max(most_recent_md_time, f.stat().st_mtime)
            if most_recent_md_time > graph_mtime:
                print(f"{Colors.YELLOW}[WARN] reports/intelligence/symbol_graph.json is older than some wiki files. "
                      f"Please run 'npm run graph:export' to rebuild the symbol graph and prevent false positives.{Colors.ENDC}\n")
        except Exception:
            pass

    # 1. Map available pages (stems)
    available_pages = {f.stem for f in files}
    # Add manual static files
    available_pages.add("Home")
    available_pages.add("_Sidebar")
    available_pages.add("_Footer")
    available_pages.add("index")
    available_pages.add("log")
    
    # 2. Load codebase symbols
    code_symbols = set()
    if symbol_graph_path.exists():
        try:
            graph_data = json.loads(symbol_graph_path.read_text(encoding='utf-8'))
            for node in graph_data.get("nodes", []):
                if node.get("kind") != "wiki" and node.get("name") != "__module__":
                    code_symbols.add(node.get("name"))
        except Exception as e:
            print(f"{Colors.YELLOW}[WARN] Could not parse symbol_graph.json: {e}. Skipping code-symbol checks.{Colors.ENDC}")
    else:
        print(f"{Colors.YELLOW}[WARN] reports/intelligence/symbol_graph.json not found. Run 'npm run graph:export' first. Skipping code-symbol checks.{Colors.ENDC}")

    # Core whitelist of keywords/types to ignore in backticks
    globals_path = project_root / ".agent" / "globals.json"
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

    # 3. Track links
    inbound_links: Dict[str, Set[str]] = {p: set() for p in available_pages}
    broken_links: List[Tuple[str, str, str]] = [] # (source_file, target_link, raw_inside)
    stale_symbols: List[Tuple[str, str]] = [] # (source_file, symbol)
    
    for f in files:
        rel_path = f.relative_to(wiki_dir).as_posix()
        try:
            content = f.read_text(encoding='utf-8')
        except Exception as e:
            print(f"{Colors.RED}[FAIL] Could not read {rel_path}: {e}{Colors.ENDC}")
            continue
            
        # Parse wiki links [[WikiPageName]]
        wiki_link_matches = re.finditer(r'\[\[([^\]]+)\]\]', content)
        for m in wiki_link_matches:
            raw_inside = m.group(1)
            # Handle aliases and section headers, e.g. [[LLM-Providers|Ollama]] or [[Home#Security]]
            target = raw_inside.split('|')[0].split('#')[0].strip()
            
            if target in available_pages:
                inbound_links[target].add(f.stem)
            else:
                broken_links.append((rel_path, target, raw_inside))
                
        # Parse backticked terms for code drift
        backtick_matches = re.findall(r'`([^`\n]+)`', content)
        for term in backtick_matches:
            term = term.strip()
            if term.endswith("()"):
                term = term[:-2]
            if term in whitelist:
                continue
            if is_valid_code_identifier(term):
                # If symbol graph is available, check if code_symbols has the term
                if code_symbols and term not in code_symbols:
                    stale_symbols.append((rel_path, term))

    # Print results
    has_issues = False
    
    print("\n--- 📖 Wiki Lint Report ---")
    
    # Output Broken Links
    if broken_links:
        has_issues = True
        print(f"\n{Colors.RED}[FAIL] Found {len(broken_links)} Broken Wiki Links:{Colors.ENDC}")
        for src, target, raw in broken_links:
            print(f"  - {src}: Link [[{raw}]] points to non-existent page '{target}'")
            
    # Output Stale Code Symbols
    if stale_symbols:
        has_issues = True
        print(f"\n{Colors.YELLOW}[WARN] Found {len(stale_symbols)} Stale Code-Symbol References (Drift):{Colors.ENDC}")
        for src, sym in stale_symbols:
            print(f"  - {src}: Backticked code reference `{sym}` not found in codebase symbol graph")

    # Output Orphans (Pages with 0 inbound links)
    # Ignore Home, _Sidebar, _Footer, index, log
    orphans = []
    for page, in_links in inbound_links.items():
        if page in ("Home", "_Sidebar", "_Footer", "index", "log"):
            continue
        if len(in_links) == 0:
            orphans.append(page)
            
    if orphans:
        has_issues = True
        print(f"\n{Colors.YELLOW}[WARN] Found {len(orphans)} Orphan Pages (No inbound links):{Colors.ENDC}")
        for page in sorted(orphans):
            # Find file path
            matching_files = [f.relative_to(wiki_dir).as_posix() for f in files if f.stem == page]
            path_str = matching_files[0] if matching_files else page
            print(f"  - {path_str} is orphaned (unlinked)")

    print("--------------------------")
    
    if not has_issues:
        print(f"{Colors.GREEN}[OK] Wiki lint passed. No issues found.{Colors.ENDC}\n")
        log_event(wiki_dir, "lint", "passed with 0 errors/warnings")
        return True
    else:
        log_event(wiki_dir, "lint", f"flagged {len(broken_links)} broken links, {len(stale_symbols)} stale symbols, {len(orphans)} orphans")
        if strict:
            print(f"{Colors.RED}[FAIL] Strict Mode Active — Blocking build/commit.{Colors.ENDC}\n")
            return False
        else:
            print(f"{Colors.YELLOW}[WARN] Warning Mode Only — Build/commit allowed.{Colors.ENDC}\n")
            return True

def main():
    import argparse
    parser = argparse.ArgumentParser(description="Tadpole OS LLM Wiki Operations Tool")
    parser.add_argument("root", nargs="?", default=".", help="Project workspace root path")
    parser.add_argument("--index", action="store_true", help="Generate docs/wiki/index.md")
    parser.add_argument("--lint", action="store_true", help="Lint docs/wiki/ for broken links, orphans, and drift")
    parser.add_argument("--strict", action="store_true", help="Fail with exit 1 if lint issues exist")
    args = parser.parse_args()
    
    project_root = Path(args.root).resolve()
    wiki_dir = project_root / "docs" / "wiki"
    
    if not wiki_dir.exists():
        print(f"Error: Wiki directory does not exist at {wiki_dir}")
        sys.exit(1)
        
    # If no specific action is selected, default to linting
    run_lint = args.lint or not args.index
    
    if args.index:
        generate_index(wiki_dir)
        
    if run_lint:
        passed = lint_wiki(wiki_dir, project_root, args.strict)
        if not passed:
            sys.exit(1)

if __name__ == "__main__":
    main()
