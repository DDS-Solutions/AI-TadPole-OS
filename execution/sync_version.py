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
    "directives/IDENTITY.md": r'TadpoleOS/([0-9.]+)',
    "docs/TROUBLESHOOTING.md": r'\*\*Version\*\*:\s*([0-9.]+)',
    "docs/openapi.yaml": r'version:\s*([0-9.]+)',
    "CHANGELOG.md": r'## \[([0-9.]+)\]',
    "README.md": r'\*\*Version\*\*:\s*([0-9.]+)',
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
        new_content = re.sub(pattern, lambda m: m.group(0).replace(m.group(1), current_version), content, flags=re.MULTILINE)

        if new_content != content:
            file_path.write_text(new_content, encoding='utf-8')
            print(f"[+] Updated: {path}")
            success_count += 1
        else:
            # Check if it's already in sync
            if re.search(pattern, content, flags=re.MULTILINE):
                print(f"[=] Already in sync: {path}")
                success_count += 1
            else:
                print(f"[-] Failed to find pattern in: {path}")
                fail_count += 1

    print(f"\n[*] Sync finished: {success_count} success, {fail_count} failed.")
    if fail_count > 0:
        sys.exit(1)

if __name__ == "__main__":
    sync()
