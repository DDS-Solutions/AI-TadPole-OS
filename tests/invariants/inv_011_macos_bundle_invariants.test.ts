/**
 * @docs ARCHITECTURE:DesktopShell
 *
 * ### AI Context Alignment
 * - **Subsystem**: Invariant Verification Suite / inv_011_macos_bundle_invariants.test
 *
 * ### AI Assist Note
 * Configuration wiring checks complement executable packaging/data-directory tests.
 * Native compilation and launch must still be verified on macOS.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 *
 * INV-011: Universal macOS Bundle Invariants
 * Checks that:
 * 1. tauri.conf.json enforces macOS minimumSystemVersion >= 11.0 and icns icon.
 * 2. server-rs/deny.toml covers both dual-slice targets (aarch64-apple-darwin and x86_64-apple-darwin).
 * 3. .github/workflows/build.yml contains macos-universal with lipo fat binary splicing.
 * 4. src-tauri/src/main.rs defines resolve_data_dir with macOS Application Support fallback.
 */

import { describe, it, expect } from 'vitest';
import { readFileSync, existsSync } from 'node:fs';
import { resolve } from 'node:path';

const ROOT_DIR = resolve(__dirname, '../..');

describe('INV-011: Universal macOS Build & Packaging Invariants', () => {
    it('INV-MAC-001: tauri.conf.json contains macOS bundle configuration and valid minimumSystemVersion', () => {
        const confPath = resolve(ROOT_DIR, 'src-tauri/tauri.conf.json');
        expect(existsSync(confPath)).toBe(true);

        const config = JSON.parse(readFileSync(confPath, 'utf-8'));
        expect(config.bundle).toBeDefined();
        expect(config.bundle.macOS).toBeDefined();
        expect(config.bundle.macOS.minimumSystemVersion).toBeDefined();

        const minVer = parseFloat(config.bundle.macOS.minimumSystemVersion);
        expect(minVer).toBeGreaterThanOrEqual(11.0);
        expect(config.bundle.macOS.signingIdentity).toBe('-');
    });

    it('INV-MAC-002: tauri.conf.json bundle icons include an Apple icns file', () => {
        const confPath = resolve(ROOT_DIR, 'src-tauri/tauri.conf.json');
        const config = JSON.parse(readFileSync(confPath, 'utf-8'));

        expect(Array.isArray(config.bundle?.icon)).toBe(true);
        const hasIcns = config.bundle.icon.some((iconPath: string) =>
            iconPath.endsWith('.icns') && existsSync(resolve(ROOT_DIR, 'src-tauri', iconPath)));
        expect(hasIcns).toBe(true);
    });

    it('INV-MAC-003: server-rs/deny.toml audits dual macOS slices (aarch64 and x86_64)', () => {
        const denyPath = resolve(ROOT_DIR, 'server-rs/deny.toml');
        expect(existsSync(denyPath)).toBe(true);

        const denyContent = readFileSync(denyPath, 'utf-8');
        expect(denyContent).toContain('aarch64-apple-darwin');
        expect(denyContent).toContain('x86_64-apple-darwin');
    });

    it('INV-MAC-004: .github/workflows/build.yml configures macos-universal with lipo splice', () => {
        const workflowPath = resolve(ROOT_DIR, '.github/workflows/build.yml');
        expect(existsSync(workflowPath)).toBe(true);

        const content = readFileSync(workflowPath, 'utf-8');
        expect(content).toContain('macos-universal');
        expect(content).toContain('macos-14');
        expect(content).toContain('bash scripts/build-macos-universal.sh');
        expect(content).toContain('*.dmg');
        const builder = readFileSync(resolve(ROOT_DIR, 'scripts/build-macos-universal.sh'), 'utf-8');
        expect(builder).toContain('server-rs-universal-apple-darwin');
        expect(builder).toContain('lipo -verify_arch arm64 x86_64');
        expect(builder).toContain('tauri build --target universal-apple-darwin -- --locked');
        expect(builder).toContain('verify-macos-bundle.sh');
    });

    it('INV-MAC-005: src-tauri/src/main.rs implements resolve_data_dir for macOS App Support and TADPOLE_DATA_DIR', () => {
        const mainRsPath = resolve(ROOT_DIR, 'src-tauri/src/main.rs');
        expect(existsSync(mainRsPath)).toBe(true);

        const content = readFileSync(mainRsPath, 'utf-8');
        expect(content).toContain('fn resolve_data_dir');
        expect(content).toContain('TADPOLE_DATA_DIR');
        const paths = readFileSync(resolve(ROOT_DIR, 'src-tauri/src/data_dir.rs'), 'utf-8');
        expect(paths).toContain('Library/Application Support/com.tadpole.os');
        expect(content).toContain('data_dir::prepare_data_dir');
        expect(content).toContain('resolve_data_dir(&install_dir)');
    });
});
