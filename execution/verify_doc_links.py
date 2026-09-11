"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / verify_doc_links
- **Primary Entrypoints**: `print_result`, `slugify_heading`, `extract_headings`, `resolve_target`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
- **Witness Tests**: none declared
"""

import os
import re
import sys
import io
from pathlib import Path
from typing import Dict, List, Set, Tuple

# Ensure stdout handles UTF-8 on Windows
if sys.platform == "win32":
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
    sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8')

def print_result(check: str, status: bool, message: str):
    icon = "[OK]" if status else "[FAIL]"
    print(f"{icon} [{check}] {message}")

def slugify_heading(heading: str) -> str:
    """Standard GitHub Flavored Markdown heading anchor generation."""
    h = re.sub(r'\[([^\]]+)\]\([^)]+\)', r'\1', heading)
    h = re.sub(r'[`*~#:\(\)\/\+]', '', h)
    h = h.strip().lower()
    # In GFM, whitespace is replaced by dash, but underscores are preserved
    h = re.sub(r'\s+', '-', h)
    # Remove punctuation characters (preserve alphanumeric, dash, underscore)
    h = re.sub(r'[^\w\-]', '', h, flags=re.UNICODE)
    return h

def extract_headings(content: str) -> Set[str]:
    """Extracts all slugified heading anchors from markdown text."""
    headings = set()
    for line in content.splitlines():
        line = line.strip()
        m = re.match(r'^(#{1,6})\s+(.+)$', line)
        if m:
            raw_title = m.group(2)
            slug = slugify_heading(raw_title)
            if slug:
                headings.add(slug)
                # Also add normalized variants (e.g. dash instead of underscore or vice versa)
                headings.add(slug.replace('_', '-'))
                headings.add(slug.replace('-', '_'))
                headings.add(re.sub(r'[^\w\-]', '', raw_title.lower().replace(" ", "-")))
    return headings

def resolve_target(src_file: Path, target_str: str, root: Path) -> Tuple[Path, str]:
    """Resolves link target into an absolute file Path and optional anchor."""
    anchor = ""
    clean_target = target_str.strip()

    if "#" in clean_target:
        parts = clean_target.split("#", 1)
        clean_target = parts[0]
        anchor = parts[1]

    if not clean_target:
        return src_file, anchor

    if clean_target.startswith("file:///"):
        path_str = clean_target[8:]
        if path_str.startswith("/") and len(path_str) > 2 and path_str[2] == ":":
            path_str = path_str[1:]
        
        # Cross-platform normalization for workspace paths (e.g. d:/TadpoleOS-Dev/...)
        normalized_lower = path_str.replace("\\", "/").lower()
        matched = False
        for prefix in ["d:/tadpoleos-dev/", "c:/tadpoleos-dev/", "/d:/tadpoleos-dev/", "/c:/tadpoleos-dev/"]:
            if normalized_lower.startswith(prefix):
                rel = path_str.replace("\\", "/")[len(prefix):].lstrip("/")
                path_str = str(root / rel)
                matched = True
                break
        
        if not matched and path_str.startswith(("/.agent", "/docs", "/server-rs", "/src", ".agent", "docs", "server-rs", "src")):
            path_str = str(root / path_str.lstrip("/"))
            
        resolved = Path(path_str)
    elif clean_target.startswith("/"):
        resolved = (root / clean_target.lstrip("/")).resolve()
    else:
        resolved = (src_file.parent / clean_target).resolve()

    return resolved, anchor

def verify_markdown_links(root: Path) -> int:
    """Scans .agent/skills/, .agent/workflows/, and docs/ for valid relative and absolute links."""
    print("--- Tadpole OS Cross-Reference & Link Guard ---\n")

    scan_dirs = [
        root / ".agent" / "skills",
        root / ".agent" / "workflows",
        root / "docs",
    ]

    md_files: List[Path] = []
    for d in scan_dirs:
        if d.exists():
            for f in d.rglob("*.md"):
                if "archive" in f.parts:
                    continue
                md_files.append(f)

    for root_doc in ["AGENTS.md", "GEMINI.md", "CLAUDE.md", "README.md"]:
        f = root / root_doc
        if f.exists():
            md_files.append(f)

    file_headings_cache: Dict[Path, Set[str]] = {}

    total_links = 0
    errors = 0

    link_pattern = re.compile(r'\[([^\]]+)\]\(([^)]+)\)')

    PLACEHOLDER_TARGETS = {
        "subtopic-file.md",
        "other.md",
        "link",
        "[[home]]",
        "url",
        "example.com",
    }

    for src_file in md_files:
        try:
            content = src_file.read_text(encoding="utf-8", errors="ignore")
        except Exception as e:
            print_result("LINK-READ", False, f"Could not read {src_file.relative_to(root)}: {e}")
            errors += 1
            continue

        matches = link_pattern.findall(content)
        rel_src = src_file.relative_to(root)

        for label, raw_target in matches:
            raw_target = raw_target.strip()
            if raw_target.lower() in PLACEHOLDER_TARGETS or raw_target.startswith("[["):
                continue
            if raw_target.startswith(("http://", "https://", "mailto:", "javascript:", "tel:", "@")):
                continue

            total_links += 1
            target_path, anchor = resolve_target(src_file, raw_target, root)

            if not target_path.exists():
                print_result("DEAD-LINK", False, f"{rel_src}: '[{label}]({raw_target})' -> Target missing: {target_path}")
                errors += 1
                continue

            if anchor and target_path.suffix.lower() in [".md", ".markdown"]:
                if target_path not in file_headings_cache:
                    try:
                        target_content = target_path.read_text(encoding="utf-8", errors="ignore")
                        file_headings_cache[target_path] = extract_headings(target_content)
                    except Exception:
                        file_headings_cache[target_path] = set()

                valid_headings = file_headings_cache[target_path]
                clean_anchor = anchor.lower().strip()

                if valid_headings:
                    matched = (
                        clean_anchor in valid_headings
                        or clean_anchor.replace('_', '-') in valid_headings
                        or clean_anchor.replace('-', '_') in valid_headings
                        or any(clean_anchor in h or h in clean_anchor for h in valid_headings)
                    )
                    if not matched:
                        target_text = target_path.read_text(encoding="utf-8", errors="ignore")
                        if f'id="{anchor}"' not in target_text and f'"{anchor}"' not in target_text:
                            print_result("DEAD-ANCHOR", False, f"{rel_src}: '[{label}]({raw_target})' -> Anchor #{anchor} not found in {target_path.name}")
                            errors += 1
                            continue

    print(f"\nScanned {len(md_files)} active markdown documents, verified {total_links} links.")
    if errors == 0:
        print_result("LINK-INTEGRITY", True, f"All {total_links} internal cross-references healthy.")
        return 0
    else:
        print_result("LINK-INTEGRITY", False, f"Found {errors} broken cross-references.")
        return errors

if __name__ == "__main__":
    project_root = sys.argv[1] if len(sys.argv) > 1 else "."
    sys.exit(verify_markdown_links(Path(project_root).resolve()))
