/**
 * @docs ARCHITECTURE:Quality:Verification
 *
 * ### AI Context Alignment
 * - **Subsystem**: Invariant Verification Suite / inv_012_durable_state_backups.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 *
 * INV-012: Durable State Separation & Encrypted Backup Invariants.
 *
 * Asserts that:
 * 1. server-rs databases.rs resolves canonical state strictly to data/state/tadpole.db (never .tmp/).
 * 2. server-rs memory.rs workspaces_root resolves to data/state/workspaces.
 * 3. server-rs backup crypto module enforces TADPOLE_SNAP_V1 authenticated envelope.
 * 4. server-rs backup snapshot module blocks directory traversal on archive extraction.
 * 5. server-rs router wires /backup/status, /backup/configure, /backup/trigger, /backup/restore, /backup/history.
 * 6. Backup_Settings.tsx adheres to DESIGN.md with tooltips and dual destination selection.
 */

import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

describe('INV-012: Durable State Separation & Encrypted Backup Invariants', () => {
    it('databases.rs resolves canonical database strictly to data/state/tadpole.db', () => {
        const source = readFileSync(
            resolve('server-rs/src/state/init/databases.rs'),
            'utf-8'
        );

        expect(source).toContain('base_dir.join("data").join("state")');
        expect(source).toContain('data/state/tadpole.db');
        expect(source).not.toContain('.tmp/tadpole.db');
        expect(source).toContain('migrate_legacy_db_if_needed');
    });

    it('memory.rs prioritizes data/state/workspaces for vector memory stores', () => {
        const source = readFileSync(
            resolve('server-rs/src/routes/memory.rs'),
            'utf-8'
        );

        expect(source).toContain('data/state/workspaces');
        expect(source).toContain('legacy_workspaces');
    });

    it('backup crypto module enforces RFC 2898 PBKDF2-HMAC-SHA512 and RFC 2104 HMAC-SHA256 without hardcoded keys', () => {
        const cryptoSource = readFileSync(
            resolve('server-rs/src/services/backup/crypto.rs'),
            'utf-8'
        );
        const daemonSource = readFileSync(
            resolve('server-rs/src/services/backup/daemon.rs'),
            'utf-8'
        );
        const routesSource = readFileSync(
            resolve('server-rs/src/routes/backup.rs'),
            'utf-8'
        );

        expect(cryptoSource).toContain('TADPOLE_SNAP_V2');
        expect(cryptoSource).toContain('TADPOLE_SNAP_V1');
        expect(cryptoSource).toContain('210_000');
        expect(cryptoSource).toContain('pbkdf2_hmac_sha512_64');
        expect(cryptoSource).toContain('hmac_sha256_parts');
        expect(cryptoSource).toContain('derive_backup_keys');
        expect(cryptoSource).toContain('encrypt_backup_payload');
        expect(cryptoSource).toContain('decrypt_backup_payload');
        expect(cryptoSource).toContain('generate_recovery_key_manifest');

        // Fail-closed passphrase invariants: no hardcoded fallback key and no plaintext serialization
        expect(daemonSource).not.toContain('TadpoleSovereignDefaultKey');
        expect(routesSource).not.toContain('TadpoleSovereignDefaultKey');
        expect(daemonSource).toContain('skip_serializing');
        expect(routesSource).toContain('RequireAdmin');
        expect(routesSource).toContain('validate_backup_directory');
        expect(routesSource).toContain('validate_snapshot_file');
    });

    it('backup snapshot module prevents path traversal and Windows drive-letter escapes during archive extraction', () => {
        const source = readFileSync(
            resolve('server-rs/src/services/backup/snapshot.rs'),
            'utf-8'
        );

        expect(source).toContain('rel_path_str.contains("..")');
        expect(source).toContain('has_windows_drive');
        expect(source).toContain('Component::Normal');
        expect(source).toContain('Illegal path in archive');
        expect(source).toContain('create_state_snapshot');
        expect(source).toContain('VACUUM INTO');
    });

    it('router.rs wires all five backup endpoints under /backup/*', () => {
        const source = readFileSync(
            resolve('server-rs/src/router.rs'),
            'utf-8'
        );

        expect(source).toContain('.route("/backup/status", get(routes::backup::get_backup_status))');
        expect(source).toContain('.route("/backup/configure", post(routes::backup::configure_backup))');
        expect(source).toContain('.route("/backup/trigger", post(routes::backup::trigger_backup_now))');
        expect(source).toContain('.route("/backup/restore", post(routes::backup::restore_backup))');
        expect(source).toContain('.route("/backup/history", get(routes::backup::get_backup_history))');
    });

    it('Backup_Settings.tsx adheres to DESIGN.md with tooltips and local/cloud options', () => {
        const source = readFileSync(
            resolve('src/components/settings/Backup_Settings.tsx'),
            'utf-8'
        );

        expect(source).toContain('Tooltip');
        expect(source).toContain('settings.opt_local');
        expect(source).toContain('settings.opt_cloud_sync');
        expect(source).toContain('handleTriggerBackupNow');
        expect(source).toContain('handleDownloadRecoveryKey');
        expect(source).toContain('font-mono');
        expect(source).toContain('bg-[color:var(--color-surface)]');
    });
});
