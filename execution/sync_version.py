"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Assist Note
**Sovereign Version Synchronizer**
Advanced agentic logic and tool orchestration for the Tadpole OS swarm.

### 🔍 Debugging & Observability
- **Failure Path**: Script error, API failure, or logic drift in the 3-layer architecture.
- **Telemetry Link**: Search `[sync_version]` in system logs.
"""

"""
Sovereign Version Synchronizer

### AI Assist Note
**Release Gatekeeper**: Enforces codebase-wide version parity by 
propagating the `version.json` source-of-truth across 20+ manifests, 
Cargo.toml files, and documentation blocks. Uses a strictly typed 
SemVer parser to perform major/minor/patch bumps.

### 🔍 Debugging & Observability
- **Failure Path**: Regex capture group mismatch (ROUT-05). Occurs if 
  a manifest format changes without updating the global `PATHS` map.
- **Verification**: Run `python execution/sync_version.py` before 
  any production deployment or audit lock.
"""
import json
import os
import re
import sys
import argparse
from pathlib import Path
from typing import List, Dict, Any

# Paths to sync
# Key: path relative to repo root
# Value: regex pattern to find the version string. 
# The first capturing group MUST be the version string.
PATHS = {
    "package.json": r'"version":\s*"([^"]+)"',
    "src-tauri/tauri.conf.json": r'"version":\s*"([^"]+)"',
    "server-rs/Cargo.toml": r'^version\s*=\s*"([^"]+)"',
    "src-tauri/Cargo.toml": r'^version\s*=\s*"([^"]+)"',
    "server-rs/src/state/mod.rs": r'TadpoleOS/([0-9.]+)',
    "server-rs/src/adapter/discord.rs": r'TadpoleOS/([0-9.]+)',
    "server-rs/src/agent/skill_manifest.rs": r'\bversion:\s*"([^"]+)"',
    "directives/IDENTITY.md": [
        r'TadpoleOS/([0-9.]+)',
        r'\*\*System Version\*\*:\s*([0-9.]+)',
        r'- \*\*Version\*\*:\s*([0-9.]+)'
    ],
    "docs/TROUBLESHOOTING.md": r'\*\*Version\*\*:\s*([0-9.]+)',
    "docs/openapi.yaml": r'version:\s*([0-9.]+)',
    "CHANGELOG.md": r'## \[([0-9.]+)\]',
    "README.md": r'\*\*Version\*\*:\s*([0-9.]+)',
    "docs/API_CONTRACT.md": r'Version[-:]\s*([0-9.]+)',
    "docs/OPERATIONS_MANUAL.md": r'\*\*Version\*\*:\s*([0-9.]+)',
    "docs/GETTING_STARTED.md": r'\*\*Version\*\*:\s*([0-9.]+)',
    "src/components/Command_Table.tsx": r"ver:\s*'([^']+)'",
    "src/components/Command_Table.test.tsx": r"Version:\s*([0-9.]+)",
    "docs/API_REFERENCE.md": r'\*\*Version\*\*:\s*([0-9.]+)',
    "index.html": r'"softwareVersion":\s*"([^"]+)"',
    "docs/wiki/_Sidebar.md": r'\*Version:\s*([0-9.]+)',
    "docs/wiki/_Footer.md": r'• Version:\s*([0-9.]+)',
}

def bump_version(current_version, part):
    """Increments the specified part of a SemVer string."""
    try:
        major, minor, patch = map(int, current_version.split('.'))
        if part == 'major':
            major += 1
            minor = 0
            patch = 0
        elif part == 'minor':
            minor += 1
            patch = 0
        elif part == 'patch':
            patch += 1
        return f"{major}.{minor}.{patch}"
    except Exception as e:
        print(f"[-] Error parsing version '{current_version}': {e}")
        sys.exit(1)

def sync():
    parser = argparse.ArgumentParser(description="Sovereign Version Synchronizer")
    parser.add_argument("--bump", choices=["major", "minor", "patch"], help="Bump the version before syncing")
    args = parser.parse_args()

    # Load Source of Truth
    if not os.path.exists("version.json"):
        print("[-] Error: version.json not found")
        sys.exit(1)

    with open("version.json", "r") as f:
        data = json.load(f)
    
    current_version = data.get("version")
    if not current_version:
        print("[-] Error: No version found in version.json")
        sys.exit(1)

    # Handle Bump
    if args.bump:
        old_version = current_version
        current_version = bump_version(current_version, args.bump)
        data["version"] = current_version
        with open("version.json", "w") as f:
            json.dump(data, f, indent=2)
        print(f"[+] Bumped version: {old_version} -> {current_version}")

    print(f"[*] Synchronizing to version: {current_version}")

    success_count = 0
    fail_count = 0

    for path, pattern in PATHS.items():
        file_path = Path(path)
        if not file_path.exists():
            print(f"[-] Warning: {path} not found, skipping...")
            continue

        content = file_path.read_text(encoding='utf-8')

        # Update content using regex
        # We replace the content of the first capturing group with the target version
        patterns = [pattern] if isinstance(pattern, str) else pattern
        new_content = content
        for pat in patterns:
            new_content = re.sub(pat, lambda m: m.group(0).replace(m.group(1), current_version), new_content, flags=re.MULTILINE)

        if new_content != content:
            file_path.write_text(new_content, encoding='utf-8')
            print(f"[+] Updated: {path}")
            success_count += 1
        else:
            # Check if it's already in sync
            all_in_sync = True
            for pat in patterns:
                if not re.search(pat, content, flags=re.MULTILINE):
                    all_in_sync = False
                    break
            
            if all_in_sync:
                print(f"[=] Already in sync: {path}")
                success_count += 1
            else:
                print(f"[-] Failed to find pattern in: {path}")
                fail_count += 1

    # Regenerate API Reference documentation so it stays in absolute sync with openapi.yaml
    try:
        from generate_api_reference import generate_reference
        generate_reference()
        print("[+] Regenerated docs/API_REFERENCE.md")
    except Exception as e:
        print(f"[-] Error regenerating API Reference: {e}")

    # Custom post-sync for docs/wiki/_Footer.md to update SHA and DATE
    footer_path = Path("docs/wiki/_Footer.md")
    if footer_path.exists():
        try:
            import subprocess
            import datetime
            sha = subprocess.check_output(["git", "rev-parse", "--short", "HEAD"]).decode("utf-8").strip()
            date_str = datetime.date.today().isoformat()
            
            content = footer_path.read_text(encoding='utf-8')
            # Replace date and SHA
            # Last Synced: 2026-06-11 (commit b1c347b1)
            updated_content = re.sub(
                r'Last Synced:\s*[0-9\-]+\s*\(commit\s*[a-f0-9]+\)',
                f'Last Synced: {date_str} (commit {sha})',
                content
            )
            # Also make sure Status: verified is set
            updated_content = re.sub(
                r'Status:\s*\w+',
                'Status: verified',
                updated_content
            )
            if updated_content != content:
                footer_path.write_text(updated_content, encoding='utf-8')
                print("[+] Updated wiki _Footer.md with current commit SHA and date.")
        except Exception as e:
            print(f"[-] Error updating _Footer.md: {e}")

    print(f"\n[*] Sync finished: {success_count} success, {fail_count} failed.")
    if fail_count > 0:
        sys.exit(1)

if __name__ == "__main__":
    sync()

# Metadata: [sync_version]
