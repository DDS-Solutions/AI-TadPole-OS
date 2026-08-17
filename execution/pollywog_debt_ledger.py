# execution/pollywog_debt_ledger.py
"""
### AI Assist Note (Knowledge Heritage):
- @docs ARCHITECTURE:Core:Execution
- Failure Path: Rotting shortcuts, unparsed debt comments
- Telemetry Link: Search [pollywog-debt] in audit logs

### AI Assist Note
Automated Technical Debt Ledger script that crawls the workspace, parses pollywog: comments, and generates POLLYWOG-DEBT.md.

### 🔍 Debugging & Observability
Traceability via standard stdout and exit codes.
"""

import os
import re
import sys
import io

# Ensure stdout handles UTF-8 on Windows
if sys.platform == "win32":
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
    sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8')

EXCLUDE_DIRS = {
    ".git",
    ".tmp",
    "__pycache__",
    "coverage",
    "dist",
    "node_modules",
    "target",
}
TEXT_EXTENSIONS = {
    ".cjs",
    ".css",
    ".html",
    ".js",
    ".json",
    ".jsx",
    ".kt",
    ".kts",
    ".md",
    ".mjs",
    ".py",
    ".rs",
    ".scss",
    ".sh",
    ".sql",
    ".toml",
    ".ts",
    ".tsx",
    ".yaml",
    ".yml",
}
COMMENT_PATTERN = re.compile(r"(?://|#|/\*|--|#|;)\s*pollywog:\s*(.+)")

def scan_files(root_dir):
    ledger = []
    for dirpath, dirnames, filenames in os.walk(root_dir):
        # Filter directories in-place to exclude unwanted trees
        dirnames[:] = [d for d in dirnames if d not in EXCLUDE_DIRS and not d.startswith(".")]
        
        for filename in filenames:
            # Scan only known text source/document formats. This prevents compiled
            # bytecode or other binary artifacts from becoming ledger content.
            if os.path.splitext(filename)[1].lower() not in TEXT_EXTENSIONS:
                continue
                
            file_path = os.path.join(dirpath, filename)
            rel_path = os.path.relpath(file_path, root_dir).replace("\\", "/")
            
            # Avoid scanning the generated file itself
            if filename == "POLLYWOG-DEBT.md" or filename == "pollywog_debt_ledger.py":
                continue
                
            try:
                with open(file_path, "r", encoding="utf-8", errors="ignore") as f:
                    for line_num, line in enumerate(f, 1):
                        match = COMMENT_PATTERN.search(line)
                        if match:
                            comment_content = match.group(1).strip()
                            # Clean up trailing comment markers if multi-line comment (e.g. */)
                            if comment_content.endswith("*/"):
                                comment_content = comment_content[:-2].strip()
                            ledger.append({
                                "file": rel_path,
                                "line": line_num,
                                "content": comment_content
                            })
            except Exception:
                continue
    return ledger

def generate_report(ledger):
    if not ledger:
        return "No pollywog: debt. Clean ledger."
        
    report = []
    report.append("# 📋 Tadpole OS: Technical Debt Ledger")
    report.append("\nThis document lists all deliberate shortcuts and architectural ceilings marked with `pollywog:` comments.\n")
    report.append("| File | Line | Shortcut / Ceiling | Upgrade Trigger | Status |")
    report.append("| :--- | :--- | :--- | :--- | :--- |")
    
    rot_count = 0
    for item in ledger:
        # Check if the comment specifies a trigger via comma division
        parts = item["content"].split(",", 1)
        ceiling = parts[0].strip()
        trigger = parts[1].strip() if len(parts) > 1 else ""
        
        if not trigger:
            trigger_display = "⚠️ *No trigger defined*"
            status = "🚨 **Rot Risk**"
            rot_count += 1
        else:
            trigger_display = trigger
            status = "✅ Tracked"
            
        report.append(f"| [{os.path.basename(item['file'])}](file:///{os.path.abspath(item['file'])}#L{item['line']}) | {item['line']} | {ceiling} | {trigger_display} | {status} |")
        
    report.append(f"\n**Summary**: {len(ledger)} markers found, {rot_count} with no upgrade trigger.")
    return "\n".join(report)

def main():
    workspace_root = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
    print(f"Scanning {workspace_root} for pollywog comments...")
    
    found_ledger = scan_files(workspace_root)
    markdown_report = generate_report(found_ledger)
    
    output_path = os.path.join(workspace_root, "POLLYWOG-DEBT.md")
    with open(output_path, "w", encoding="utf-8") as f:
        f.write(markdown_report)
        
    print(f"Ledger updated successfully in {output_path}")
    
    # Check if any comment lacks an upgrade trigger
    invalid_comments = []
    for item in found_ledger:
        if "," not in item["content"]:
            invalid_comments.append(item)
            
    if invalid_comments:
        print("\n🚨 Error: The following pollywog comments are missing an upgrade trigger:")
        for item in invalid_comments:
            print(f"  - {item['file']}:{item['line']} -> '{item['content']}'")
        print("\nAll 'pollywog:' comments must have the format: '// pollywog: <ceiling>, <upgrade trigger>'")
        sys.exit(1)
        
    sys.exit(0)

if __name__ == "__main__":
    main()
