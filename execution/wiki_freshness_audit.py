"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Assist Note
**@docs ARCHITECTURE:Infrastructure:Execution**
Advanced agentic logic and tool orchestration for the Tadpole OS swarm.

### 🔍 Debugging & Observability
- **Failure Path**: Script error, API failure, or logic drift in the 3-layer architecture.
- **Telemetry Link**: Search `[wiki_freshness_audit]` in system logs.
"""

#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Assist Note
**Wiki Freshness Audit**: Detects stale wiki pages by comparing their last-modified
timestamp against the most recent feature commit in git. Flags pages older than
a configurable threshold relative to code churn.

### Debugging and Observability
- **Failure Path**: git log unavailable, or wiki pages not updated after major features.
- **Telemetry Link**: Search [wiki_freshness] in audit logs.
"""

import sys
import io
import os
import subprocess
from pathlib import Path
from datetime import datetime, timezone

if sys.platform == "win32":
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8")

ROOT = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(".")
WIKI_DIR = ROOT / "docs" / "wiki"
STALE_THRESHOLD_DAYS = int(sys.argv[2]) if len(sys.argv) > 2 else 60
SKIP_FILES = {"_Footer.md", "_Sidebar.md", "log.md"}

def get_file_last_commit_time(fpath, root):
    try:
        result = subprocess.run(
            ["git", "log", "-1", "--format=%ct", "--", str(fpath.relative_to(root))],
            capture_output=True, text=True, cwd=root
        )
        if result.returncode == 0 and result.stdout.strip():
            return int(result.stdout.strip())
    except Exception:
        pass
    return None

def main():
    if not WIKI_DIR.exists():
        print("No wiki directory found. Skipping.")
        sys.exit(0)

    print("=" * 70)
    print("  TADPOLE OS -- WIKI FRESHNESS AUDIT")
    print("=" * 70)
    now_ts = datetime.now(timezone.utc).timestamp()
    STALE = []
    FRESH = []
    UNTRACKED = []

    for fpath in sorted(WIKI_DIR.rglob("*.md")):
        if fpath.name in SKIP_FILES:
            continue
        rel = fpath.relative_to(ROOT / "docs")
        file_commit_ts = get_file_last_commit_time(fpath, ROOT)
        if file_commit_ts:
            age_days = (now_ts - file_commit_ts) / 86400
            file_dt = datetime.fromtimestamp(file_commit_ts, tz=timezone.utc)
            if age_days > STALE_THRESHOLD_DAYS:
                status = "STALE"
                STALE.append((str(rel), age_days, file_dt))
            else:
                status = "FRESH"
                FRESH.append(str(rel))
            print(f"[{status}] [{age_days:.0f}d] {rel} (last: {file_dt.strftime('%Y-%m-%d')})")
        else:
            mtime = fpath.stat().st_mtime
            age_days = (now_ts - mtime) / 86400
            UNTRACKED.append((str(rel), age_days))
            print(f"[UNTRACKED] [{age_days:.0f}d mtime] {rel}")

    print("\n" + "=" * 70)
    total = len(FRESH) + len(STALE) + len(UNTRACKED)
    print(f"Total: {total} | Fresh: {len(FRESH)} | Stale: {len(STALE)} | Untracked: {len(UNTRACKED)}")
    if STALE:
        print("STALE PAGES:")
        for name, age, dt in sorted(STALE, key=lambda x: -x[1]):
            print(f"  - {name} [{age:.0f}d old]")
        sys.exit(1)
    sys.exit(0)

if __name__ == "__main__":
    main()

# Metadata: [wiki_freshness]

# Metadata: [wiki_freshness_audit]
