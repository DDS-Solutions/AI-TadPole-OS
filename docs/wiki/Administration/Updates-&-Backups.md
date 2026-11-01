---
title: "Updates & Backups"
tier: "2"
status: "verified"
version: "1.2.0"
last-verified: "2026-06-17"
commit: "b1c347b1"
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
| `data/state/backups/` | Default encrypted snapshot destination (`.tadpole-snap.enc`) | **High** — recovery archives |
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

This prevents out-of-memory crashes on typical office hardware (8–16 GB VRAM). Quoted in the source code at [openai.rs:L336-361](https://github.com/DDS-Solutions/Tadpole-OS/blob/main/server-rs/src/agent/openai.rs#L336-L361) and [openai.rs:L596-614](https://github.com/DDS-Solutions/Tadpole-OS/blob/main/server-rs/src/agent/openai.rs#L596-L614).

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

**[Command] Backup LanceDB Vector Store**
```powershell
xcopy /E /I data\memory.lance data\memory.lance.backup
```

### 3.3 Automated Encrypted Backup Daemon (`StateBackupDaemonService`)

Tadpole OS includes an integrated background backup service executing non-blocking SQLite `VACUUM INTO` snapshots and ChaCha20 authenticated envelope encryption (`TADPOLE_SNAP_V1`).

Key capabilities:
- **Zero-Knowledge Client-Side Encryption**: Derived via PBKDF2-HMAC-SHA512 (100,000 rounds) and constant-time HMAC-SHA256 authentication.
- **Target Selection**: Operators can select between **Local Storage (Preferred)** (e.g. NAS or dedicated storage drive) or a **Cloud Sync Folder** (OneDrive, Google Drive, Dropbox, Nextcloud local mount points). Cloud providers only ever receive ciphertext.
- **Hourly Automated Runs**: The daemon triggers an hourly background snapshot without locking the active database.
- **48-Hour Rolling Retention**: Automatically prunes snapshots older than 48 hours to conserve local disk space while recording audit states in `system_backups`.
- **Disaster Recovery Key**: Operators can export a standalone `.tadpole-recovery.key` JSON manifest containing the KDF parameters and verification canary.

### 3.4 Backup User Interface (Settings -> Backup Settings)

Configurable directly from the Core Settings panel:
- **[Toggle] Enable Automated Backups**: Activates or pauses the hourly daemon cycle.
- **[Selector] Destination Type**: Toggle between `Local Storage (Preferred)` and `Cloud Sync Folder`.
- **[Input] Backup Destination Path**: Directory picker / path input with active directory creation and tooltip instructions.
- **[Input] Encryption Passphrase**: Maskable passphrase field with password visibility toggle.
- **[Button] Export Recovery Key**: Downloads `.tadpole-recovery.key` file for emergency offline restoration.
- **[Button] Backup Now**: Triggers an immediate point-in-time snapshot, updating the live telemetry ribbon (status, timestamp, total snapshots, and storage footprint).
- **[Table] Historical Snapshots**: Audits past snapshot records with SHA-256 checksums, SQLite size, and LanceDB bytes.

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

New variables may have been added. See [[Configuration|§1]] for the complete reference.

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

→ *See [[Configuration|§5]] for persistence and storage path settings.*
→ *See [[Installation|§3]] for compilation after updates.*

---

**Complete Lexicon**: For the authoritative technical breakdown, see the main repository [`GLOSSARY.md`](https://github.com/DDS-Solutions/Tadpole-OS/blob/main/docs/GLOSSARY.md). Every `[[Glossary#term|term]]` link on this page resolves to an entry there.

<!-- Last verified against commit b1c347b1 on 2026-06-11 -->
[//]: # (wiki-page: Updates-&-Backups)


[//]: # (Metadata: [wiki-admin])
