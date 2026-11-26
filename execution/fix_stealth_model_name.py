#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Infrastructure:Execution

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / fix_stealth_model_name
- **Primary Entrypoints**: `update_sqlite_db`, `update_json_files`, `update_ts_files`, `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: `[-]`
- **Witness Tests**: none declared
"""

import os
import sys
import json
import sqlite3
from pathlib import Path

# UTF-8 stdout setup for Windows
if sys.platform == "win32":
    try:
        sys.stdout.reconfigure(encoding="utf-8")
        sys.stderr.reconfigure(encoding="utf-8")
    except Exception:
        pass

ROOT = Path(__file__).resolve().parent.parent
DB_PATH = ROOT / "data" / "tadpole.db"
STARTER_KITS_DIR = ROOT / "starter_kits"
SRC_DIR = ROOT / "src"

def update_sqlite_db():
    if not DB_PATH.exists():
        print(f"[-] SQLite DB not found at {DB_PATH}")
        return 0

    conn = sqlite3.connect(str(DB_PATH))
    c = conn.cursor()

    # Check rows before
    c.execute("SELECT count(*) FROM agents WHERE model_id = 'stealth/oxalpha' OR model_2 = 'stealth/oxalpha' OR model_3 = 'stealth/oxalpha';")
    count_before = c.fetchone()[0]

    # Update agents table
    c.execute("""
        UPDATE agents 
        SET model_id = 'stealth/ox-alpha' 
        WHERE model_id = 'stealth/oxalpha';
    """)
    c.execute("""
        UPDATE agents 
        SET model_2 = 'stealth/ox-alpha' 
        WHERE model_2 = 'stealth/oxalpha';
    """)
    c.execute("""
        UPDATE agents 
        SET model_3 = 'stealth/ox-alpha' 
        WHERE model_3 = 'stealth/oxalpha';
    """)

    conn.commit()

    # Check rows after
    c.execute("SELECT count(*) FROM agents WHERE model_id = 'stealth/ox-alpha';")
    count_after = c.fetchone()[0]
    conn.close()

    print(f"[+] SQLite: Updated {count_before} agent rows to 'stealth/ox-alpha' (Total with stealth/ox-alpha now: {count_after})")
    return count_before

def update_json_files():
    total_files_updated = 0
    total_replacements = 0

    # Scan starter kits and data dirs
    dirs_to_scan = [STARTER_KITS_DIR, ROOT / "data", ROOT / ".agent"]
    for d in dirs_to_scan:
        if not d.exists():
            continue
        for file_path in d.rglob("*.json"):
            # skip logs or node_modules
            if "logs" in file_path.parts or "node_modules" in file_path.parts:
                continue
            try:
                content = file_path.read_text(encoding="utf-8")
                if "stealth/oxalpha" in content:
                    count = content.count("stealth/oxalpha")
                    new_content = content.replace("stealth/oxalpha", "stealth/ox-alpha")
                    file_path.write_text(new_content, encoding="utf-8")
                    total_files_updated += 1
                    total_replacements += count
                    print(f"  [+] Updated {file_path.relative_to(ROOT)} ({count} replacements)")
            except Exception as e:
                print(f"  [-] Failed to update {file_path}: {e}")

    print(f"[+] JSON Files: Updated {total_files_updated} files with {total_replacements} replacements.")
    return total_files_updated

def update_ts_files():
    files_to_check = [
        SRC_DIR / "data" / "models.ts",
        SRC_DIR / "utils" / "model_utils.ts",
    ]
    for file_path in files_to_check:
        if file_path.exists():
            content = file_path.read_text(encoding="utf-8")
            if "stealth/oxalpha" in content:
                new_content = content.replace("stealth/oxalpha", "stealth/ox-alpha")
                file_path.write_text(new_content, encoding="utf-8")
                print(f"[+] Updated {file_path.relative_to(ROOT)}")

def main():
    print("=" * 70)
    print("[+] STANDARDIZING MODEL NAME TO 'stealth/ox-alpha'")
    print("=" * 70)
    update_sqlite_db()
    update_json_files()
    update_ts_files()
    print("=" * 70)
    print("[+] Model name standardization complete.")

if __name__ == "__main__":
    main()
