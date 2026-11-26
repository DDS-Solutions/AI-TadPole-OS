---
title: "Updates & Backups"
tier: "2"
status: "verified"
version: "1.1.464"
last-verified: "2026-11-20"
commit: "3eef5c2d"
network-badge: "none"
risk-tags:
  - "RISK: MEDIUM"
  - "RISK: HIGH"
---

# 💾 Updates & Backups

This page covers backup strategies, hardware recovery, and maintenance procedures for your Tadpole OS local node.

---

## 1. Directory Structure & Backup Paths

Include the following folders in your regular office backup routine:

| Path | Description | Risk if Lost |
|------|-------------|-------------|
| `data/state/tadpole.db` | Default SQLite database with agent profiles, project logs, and configurations (legacy `data/tadpole.db` is copied here on startup when needed; the old file is retained) | **Critical** — all operational data |
| `data/workspaces/<cluster-id>/` | Runner cluster workspaces, including agent memory and mission scope stores when vector memory is enabled | **High** — current mission artifacts |
| `data/state/workspaces/` | Alternate workspace root preferred by memory listing and automated snapshots when present | **High** — back up separately if both workspace roots contain data |
| `backups/` (or `data/backups/` / `TADPOLE_BACKUP_ROOT`) | Default authorized encrypted snapshot destination (`.tadpole-snap.enc`). Paths outside authorized roots are rejected (`403 Forbidden`). | **High** — recovery archives |
| `.env` | Port definitions, API key references, and local path configurations | **Critical** — contains security-sensitive values |
| `data/workflows/` | SOP markdown files for agent enforcement | **Medium** — recreatable from business documentation |
| `data/skills/` | Custom skill manifest JSON files | **Medium** — skill logic can be reconstructed |

> [!IMPORTANT]
> The default SQLite database path is `data/state/tadpole.db`. If that file is absent and a legacy `data/tadpole.db` exists, startup copies it and any WAL/SHM sidecars to the canonical path; the legacy source remains. Automated snapshots include `data/state/workspaces/` when it exists, otherwise `data/workspaces/`. If both roots contain data, the snapshot does not merge them, so back up the other root separately.

> [!CAUTION]
> The `.env` file contains sensitive credentials. Store backups in an encrypted location and never commit to version control.

---

## 2. Automatic Hardware Recovery

If your local computer runs low on GPU memory (VRAM) while loading a local Ollama model, the engine automatically:

1. Detects the out-of-memory condition.
2. Intercepts the error and dynamically down-ranks the active model by appending a `:q4_K_M` quantization suffix.
3. Reroutes the generation request, preserving project progress and resuming the active mission.

This prevents out-of-memory crashes on typical office hardware (8–16 GB VRAM). Implemented in [openai.rs:L427-464](https://github.com/DDS-Solutions/AI-TadPole-OS/blob/main/server-rs/src/agent/openai.rs#L427-L464).

---

## 3. Database Maintenance & Automated Encrypted Backups

### 3.1 SQLite Database Migration & Structure

The primary database (`data/state/tadpole.db`) stores agent configurations, execution logs, and mission states.

**[Command] Manual SQLite Snapshot (Emergency Fallback)**
```powershell
copy data\state\tadpole.db data\state\tadpole.db.backup
```

### 3.2 Vector Database (LanceDB)

Agent and mission vector stores live under their cluster workspace when the optional `vector-memory` feature is enabled. They can be rebuilt from source knowledge or restored from an automated encrypted snapshot of the selected workspace root.

**[Command] Backup Workspace Vector Stores**
```powershell
xcopy /E /I data\workspaces data\workspaces.backup
```

### 3.3 Automated Encrypted Backup Daemon (`StateBackupDaemonService`)

Tadpole OS includes an integrated background backup service (`server-rs/src/services/backup/`) executing non-blocking SQLite `VACUUM INTO` snapshots, symlink-safe `TADPOLE_ARCH_V1` workspace bundling, and ChaCha20 + HMAC-SHA256 authenticated envelope encryption (**`TADPOLE_SNAP_V2`**; legacy `TADPOLE_SNAP_V1` 100k decrypt-compatible).

Key capabilities:
- **Zero-Knowledge Encryption (`TADPOLE_SNAP_V2`)**: Derived via **PBKDF2-HMAC-SHA512 (210,000 rounds, ≥12-character passphrase)** and constant-time HMAC-SHA256 authentication. Passphrases are **never serialized** to `system_backup.json` (sourced from `TADPOLE_BACKUP_PASSPHRASE` or in-memory `Zeroizing<String>`).
- **Admin RBAC (`RequireAdmin`)**: All 5 `/v1/system/backup/*` endpoints require `NEURAL_ADMIN_TOKEN` (enter your **Step-Up Administrative Token** in Settings; operational `NEURAL_TOKEN` returns `403 Forbidden`).
- **Authorized Target Confinement**: Destinations are confined to `backups/` (default), `data/backups/`, or an explicit `TADPOLE_BACKUP_ROOT` (for NAS or Cloud Sync folders such as OneDrive/Google Drive/Dropbox — cloud providers only ever receive ciphertext).
- **Hourly Runs & 48-Hour Rolling Retention**: Automatically prunes snapshots older than 48 hours and updates `system_backups.status = 'pruned'`.
- **Safe Restore & Recovery Manifest**: The daemon writes `.tadpole-recovery.key` (`version: "2.0"`) alongside snapshots. `POST /v1/system/backup/restore` defaults to verify-only dry-run (`apply: false`) with `PRAGMA integrity_check`; passing `apply: true` stages `data/state/tadpole.db.restored` + `workspaces.restored` for rollback-safe promotion on next boot.

### 3.4 Backup User Interface (Settings → Backup Settings)

Configurable directly from the Settings panel (`src/components/settings/Backup_Settings.tsx`, elevated via Step-Up Administrative Token):
- **[Selector] Destination Type**: Toggle between `Local Storage (Preferred)` and `Cloud Sync Folder`.
- **[Input] Backup Destination Path**: Confined backup path input (`backups/`, `data/backups/`, or under `TADPOLE_BACKUP_ROOT`).
- **[Input] Encryption Passphrase**: Maskable ≥12-char passphrase field with visibility toggle (kept in engine memory only; never written to `system_backup.json`).
- **[Button] Save Configuration**: Persists non-secret backup settings via `POST /v1/system/backup/configure`.
- **[Button] Download Recovery Notes**: Exports `tadpole-recovery-notes.json` (v`2.0` KDF/restore instructions; the live `.tadpole-recovery.key` canary manifest is written server-side into the backup folder).
- **[Button] Backup Now**: Synchronously triggers an immediate `TADPOLE_SNAP_V2` snapshot (`POST /v1/system/backup/trigger`), refreshing status, timestamp, total snapshots, and storage footprint.

---

## 4. Update Procedure

### 4.1 Pulling Latest Changes

**[Command] Pull Code & Build Backend**
```powershell
git pull origin main
npm install
cd server-rs
cargo build --release
```

### 4.2 Database Migrations

Database schema upgrades are applied deterministically on boot. The engine executes pending migration scripts inside the `migrations/` directory on startup using `sqlx::migrate!()`. No manual CLI command line flags are required.

### 4.3 Environment Variable Updates

After updates, compare your `.env` against the latest `.env.example`:

**[Command] Check Env Var Differences**
```powershell
diff .env .env.example
```

New variables may have been added. See [[Configuration]] for the complete reference.

---

## 5. Legacy JSON Backup Mode

For backward compatibility, the engine supports a legacy JSON backup mode:

- `LEGACY_JSON_BACKUP` [RISK: MEDIUM] — default `false`. When enabled, keeps old JSON backup files active alongside the SQLite database.

> [!TIP]
> This mode is primarily for migration purposes. Disable it once you've confirmed the SQLite database is operating correctly.

---

## 6. Scheduled Backup Recommendations

| Frequency | What to Back Up | Method |
|-----------|----------------|--------|
| **Daily** | `data/state/tadpole.db` and `data/workspaces/<cluster-id>/` | Automated script or Windows Task Scheduler |
| **Weekly** | `data/state/workspaces/` (if present), `data/workflows/`, `data/skills/` | Folder copy to backup drive; include both workspace roots if both contain data |
| **On Change** | `.env` | Manual copy to encrypted storage |
| **Before Updates** | Full `data/` directory | Complete snapshot before `git pull` |

→ *See [[Configuration]] for persistence and storage path settings.*
→ *See [[Installation]] for compilation after updates.*

---

**Complete Lexicon**: For the authoritative technical breakdown, see the main repository [`GLOSSARY.md`](https://github.com/DDS-Solutions/AI-TadPole-OS/blob/main/docs/GLOSSARY.md). Every `[[term|Glossary#term]]` link on this page resolves to an entry there.

<!-- Last verified against commit 3eef5c2d on 2026-11-02 -->
[//]: # (wiki-page: Updates-&-Backups)


[//]: # (Metadata: [wiki-admin])
