#!/usr/bin/env tsx
/**
 * @docs ADG_V2
 *
 * ### AI Context Alignment
 * - **Subsystem**: Developer Scripts / verify_witness_tests
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic execution without side effects outside declared scope.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: [ADG]
 *
 * verify_witness_tests.ts
 * CI enforcement for ADG v2 Witness Test mappings.
 *
 * Validates that all witness tests registered in `adg.manifest.json` exist on disk,
 * or verifies declared witness headers if running in legacy mode.
 *
 * Usage: npx tsx scripts/verify_witness_tests.ts
 */

import { readFileSync, existsSync } from 'node:fs';
import { resolve, relative, dirname } from 'node:path';
import { globSync } from 'node:fs';

const MANIFEST_PATH = resolve('adg.manifest.json');
const SRC_ROOT = resolve('src');

if (existsSync(MANIFEST_PATH)) {
    try {
        const manifest = JSON.parse(readFileSync(MANIFEST_PATH, 'utf-8'));
        const witness_map = manifest?.claims?.witness_map || {};
        const witnessed_entries = Object.entries(witness_map) as [string, string[]][];

        let missing_count = 0;
        let test_count = 0;

        for (const [src_file, test_files] of witnessed_entries) {
            for (const tf of test_files) {
                test_count++;
                const abs_path = resolve(tf);
                if (!existsSync(abs_path)) {
                    console.error(`[ADG] FAIL: Witness test does not exist on disk: ${tf} (for ${src_file})`);
                    missing_count++;
                }
            }
        }

        if (missing_count > 0) {
            console.error(`\n[ADG] FAIL: ${missing_count} witness test(s) missing on disk.`);
            process.exit(1);
        }

        const witnessed_files = Object.keys(witness_map).length;
        console.log(`[ADG] OK: ${test_count} witness tests verified across ${witnessed_files} source files via adg.manifest.json.`);
        process.exit(0);
    } catch (err) {
        console.error('[ADG] Error reading adg.manifest.json:', err);
        process.exit(1);
    }
}

// Fallback legacy header scanning
const WITNESS_PATTERN = /Witness Tests\*{0,2}:\s*`([^`]+)`/g;
const source_files = globSync('src/**/*.{ts,tsx}').filter(f => !f.includes('.test.') && !f.includes('.spec.'));
const test_files_on_disk = new Set(
    globSync('src/**/*.{test,spec}.{ts,tsx}').map(f => resolve(f))
);

const declared_tests = new Map<string, string[]>();
const missing_tests: string[] = [];

for (const src_file of source_files) {
    const content = readFileSync(src_file, 'utf-8');
    let match: RegExpExecArray | null;
    WITNESS_PATTERN.lastIndex = 0;

    while ((match = WITNESS_PATTERN.exec(content)) !== null) {
        const raw_paths = match[1].split(',').map(p => p.trim()).filter(Boolean);
        for (const raw_path of raw_paths) {
            let abs_path = resolve(dirname(src_file), raw_path);
            if (!existsSync(abs_path)) {
                const root_path = resolve(raw_path);
                const src_path = resolve('src', raw_path);
                if (existsSync(root_path)) {
                    abs_path = root_path;
                } else if (existsSync(src_path)) {
                    abs_path = src_path;
                } else {
                    for (const test_file of test_files_on_disk) {
                        if (test_file.endsWith(`/${raw_path}`) || test_file.endsWith(`\\${raw_path}`) || test_file === raw_path) {
                            abs_path = test_file;
                            break;
                        }
                    }
                }
            }

            if (existsSync(abs_path)) {
                if (!declared_tests.has(abs_path)) declared_tests.set(abs_path, []);
                declared_tests.get(abs_path)!.push(src_file);
            } else if (raw_path.endsWith('.ts') || raw_path.endsWith('.tsx') || raw_path.includes('/')) {
                missing_tests.push(`  MISSING: ${raw_path} (declared in ${relative(SRC_ROOT, src_file)})`);
            }
        }
    }
}

if (missing_tests.length > 0) {
    console.error('\n[ADG] FAIL: Declared Witness Tests that do not exist on disk:');
    missing_tests.forEach(m => console.error(m));
    process.exit(1);
}

console.log(`\n[ADG] OK: ${declared_tests.size} Witness Test(s) verified across ${source_files.length} source files.`);
process.exit(0);
