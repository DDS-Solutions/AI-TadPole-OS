# @docs ARCHITECTURE:Core
#
# ### AI Assist Note
# **!/bin/bash**
# Shell-level automation for the Tadpole OS deployment and maintenance.
#
# ### 🔍 Debugging & Observability
# - **Failure Path**: Environment error, permission denied, or command failure.
# - **Telemetry Link**: Search `[backup_db]` in audit logs.

#!/bin/bash
# ================================================================
# Tadpole OS — Automated Database Backup
# ================================================================
# Uses SQLite's .backup command for an atomic, consistent snapshot.
# Safe to run against a live database.
#
# Usage: ./scripts/backup_db.sh
# Cron:  0 */6 * * * /app/scripts/backup_db.sh >> /var/log/tadpole-backup.log 2>&1
# ================================================================

BACKUP_DIR="${BACKUP_DIR:-/app/backups}"
DB_PATH="${DB_PATH:-/app/data/tadpole.db}"
KEEP_DAYS="${KEEP_DAYS:-7}"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="$BACKUP_DIR/tadpole_$TIMESTAMP.db"

# Ensure backup directory exists
mkdir -p "$BACKUP_DIR"

# Verify source database exists
if [ ! -f "$DB_PATH" ]; then
    echo "[BACKUP] ERROR: Database not found at $DB_PATH"
    exit 1
fi

# Create atomic backup
sqlite3 "$DB_PATH" ".backup '$BACKUP_FILE'"

if [ $? -eq 0 ]; then
    SIZE=$(du -h "$BACKUP_FILE" | cut -f1)
    echo "[BACKUP] Created: $BACKUP_FILE ($SIZE)"
else
    echo "[BACKUP] ERROR: Backup failed"
    exit 1
fi

# Prune old backups
PRUNED=$(find "$BACKUP_DIR" -name "tadpole_*.db" -mtime +$KEEP_DAYS -delete -print | wc -l)
if [ "$PRUNED" -gt 0 ]; then
    echo "[BACKUP] Pruned $PRUNED backup(s) older than $KEEP_DAYS days"
fi

# Metadata: [backup_db]
