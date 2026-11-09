#!/usr/bin/env node
/**
 * ADG v2 — Active Documentation Guard CLI
 *
 * Zero-dependency, zero-config. Derives truth from the source tree,
 * test co-location, and CSP grammar — never from human-written prose.
 *
 * Commands:
 *   lint-headers [dir]           Flag forbidden v1 fact-fields in JSDoc headers.
 *   codemod [--write] [dir]     Strip v1 fact-fields. Dry-run by default.
 *   generate [--check] [dir]    Compute adg.manifest.json. --check diffs claims & ratchet.
 *   verify [dir]                Run must-fail fixtures, lint, CSP grammar, manifest integrity.
 *
 * Usage:
 *   node tools/adg/adg.mjs lint-headers src/
 *   node tools/adg/adg.mjs codemod --write src/
 *   node tools/adg/adg.mjs generate
 *   node tools/adg/adg.mjs generate --check
 *   node tools/adg/adg.mjs verify
 */

import { readFileSync, writeFileSync, readdirSync, statSync, existsSync } from 'node:fs';
import { resolve, relative, dirname, basename, extname, join } from 'node:path';
import { execSync } from 'node:child_process';

// ─── Constants ──────────────────────────────────────────────────────────────

const PROJECT_ROOT = resolve(import.meta.dirname, '..', '..');
const MANIFEST_PATH = resolve(PROJECT_ROOT, 'adg.manifest.json');
const TAURI_CONF_PATH = resolve(PROJECT_ROOT, 'src-tauri', 'tauri.conf.json');
const FIXTURES_DIR = resolve(import.meta.dirname, 'fixtures', 'must-fail');
const VALID_SOURCE_EXTS = new Set(['.ts', '.tsx', '.js', '.jsx']);

/**
 * Forbidden v1 fact-fields: lines in JSDoc headers that assert facts
 * which should be derived by the manifest generator, not hand-written.
 *
 * Pattern matches lines like:
 *   * - **Witness Tests**: none declared
 *   * - **Witness Tests**: `path/to/file.test.ts`
 */
const V1_FORBIDDEN_PATTERNS = [
    /^\s*\*\s*-\s*\*\*Witness Tests\*\*:/,
];

/**
 * Codemod target: the exact line pattern to strip from files.
 * Matches the full line including leading whitespace and trailing content.
 */
const V1_CODEMOD_PATTERNS = [
    /^\s*\*\s*-\s*\*\*Witness Tests\*\*:.*\r?$/,
];

// ─── File System Utilities ──────────────────────────────────────────────────

function to_posix(path_str) {
    return path_str.replace(/\\/g, '/');
}

function rel_posix(from, to) {
    return to_posix(relative(from, to));
}

/**
 * Recursively walk a directory tree, yielding absolute paths to files
 * matching the given extensions.
 * @param {string} dir - Absolute path to the root directory.
 * @param {Set<string>} exts - File extensions to include (e.g., new Set(['.ts', '.tsx'])).
 * @returns {string[]} Array of absolute file paths.
 */
const SKIP_DIRS = new Set([
    'node_modules', '.git', 'dist', 'target', '.tmp', 'tmp', 'coverage',
    'build', '.fallow', '.gemini', '.cargo'
]);

function walk_tree(dir, exts) {
    const results = [];
    let entries;
    try {
        entries = readdirSync(dir, { withFileTypes: true });
    } catch {
        return results;
    }
    entries.sort((a, b) => a.name.localeCompare(b.name));
    for (const entry of entries) {
        const full = join(dir, entry.name);
        if (entry.isDirectory()) {
            if (SKIP_DIRS.has(entry.name)) continue;
            results.push(...walk_tree(full, exts));
        } else if (entry.isFile() && exts.has(extname(entry.name))) {
            results.push(full);
        }
    }
    return results;
}

/**
 * Check if a file is a test file based on naming convention.
 * @param {string} filename - Base filename (e.g., 'foo.test.ts').
 * @returns {boolean}
 */
function is_test_file(filename) {
    return /\.(test|spec)\.(ts|tsx|js|jsx)$/.test(filename);
}

/**
 * Get the source file name that a test file witnesses.
 * 'foo.test.ts' → 'foo.ts', 'foo.test.tsx' → 'foo.tsx'
 * @param {string} test_filename - Test file basename.
 * @returns {string|null} Source file basename or null if not a co-located test.
 */
function test_to_source_name(test_filename) {
    const match = test_filename.match(/^(.+)\.(test|spec)\.(ts|tsx|js|jsx)$/);
    if (!match) return null;
    const base = match[1];
    const ext = match[3];
    return `${base}.${ext}`;
}

// ─── lint-headers ───────────────────────────────────────────────────────────

/**
 * Scan source files for forbidden v1 fact-fields in JSDoc headers.
 * @param {string} target_dir - Directory to scan.
 * @returns {{ violations: Array<{file: string, line: number, content: string}>, scanned: number }}
 */
function lint_headers(target_dir) {
    const dir = resolve(target_dir);
    const files = walk_tree(dir, VALID_SOURCE_EXTS);
    const violations = [];

    for (const file_path of files) {
        // Skip meta-test fixtures (they are intentionally designed to fail)
        if (file_path.includes('must-fail')) continue;
        const content = readFileSync(file_path, 'utf-8');
        const lines = content.split('\n');

        // Only scan within the first JSDoc block (/** ... */)
        let in_jsdoc = false;
        for (let i = 0; i < lines.length; i++) {
            const line = lines[i];
            if (line.includes('/**')) in_jsdoc = true;
            if (in_jsdoc) {
                for (const pattern of V1_FORBIDDEN_PATTERNS) {
                    if (pattern.test(line)) {
                        violations.push({
                            file: rel_posix(PROJECT_ROOT, file_path),
                            line: i + 1,
                            content: line.trimStart(),
                        });
                    }
                }
            }
            if (line.includes('*/')) {
                if (in_jsdoc) break; // Only scan the first JSDoc block
                in_jsdoc = false;
            }
        }
    }

    return { violations, scanned: files.length };
}

// ─── codemod ────────────────────────────────────────────────────────────────

/**
 * Strip v1 fact-fields from source file headers.
 * @param {string} target_dir - Directory to scan.
 * @param {boolean} write - If true, write changes to disk. Dry-run if false.
 * @returns {{ modified: Array<{file: string, removed: number}>, scanned: number }}
 */
function codemod(target_dir, write = false) {
    const dir = resolve(target_dir);
    const files = walk_tree(dir, VALID_SOURCE_EXTS);
    const modified = [];

    for (const file_path of files) {
        if (file_path.includes('must-fail')) continue;
        const content = readFileSync(file_path, 'utf-8');
        const lines = content.split('\n');
        const output_lines = [];
        let removed = 0;
        let in_jsdoc = false;
        let jsdoc_ended = false;

        for (let i = 0; i < lines.length; i++) {
            const line = lines[i];
            if (!jsdoc_ended && line.includes('/**')) in_jsdoc = true;

            let should_remove = false;
            if (in_jsdoc && !jsdoc_ended) {
                for (const pattern of V1_CODEMOD_PATTERNS) {
                    if (pattern.test(line)) {
                        should_remove = true;
                        removed++;
                        break;
                    }
                }
            }

            if (!should_remove) {
                output_lines.push(line);
            }

            if (in_jsdoc && line.includes('*/')) {
                jsdoc_ended = true;
                in_jsdoc = false;
            }
        }

        if (removed > 0) {
            modified.push({ file: rel_posix(PROJECT_ROOT, file_path), removed });
            if (write) {
                writeFileSync(file_path, output_lines.join('\n'), 'utf-8');
            }
        }
    }

    return { modified, scanned: files.length };
}

// ─── witnessMap ─────────────────────────────────────────────────────────────

/**
 * Build the witness map: for each source file, find its co-located test files.
 * Resolution strategy:
 *   1. Co-location: `foo.ts` → `foo.test.ts` in the same directory.
 *   2. Extension variants: `foo.ts` → `foo.test.tsx` (and vice versa).
 *
 * @param {string} target_dir - Directory to scan.
 * @returns {{ witnessed: Map<string, string[]>, unattested: string[] }}
 *   - `witnessed`: Map of relative source path → array of relative test paths.
 *   - `unattested`: Source files with no co-located test.
 */
function witness_map(target_dir) {
    const dir = resolve(target_dir);
    const all_files = walk_tree(dir, VALID_SOURCE_EXTS);
    const source_files = all_files.filter(f => !is_test_file(basename(f)));
    const test_files = new Set(all_files.filter(f => is_test_file(basename(f))));

    // Build a lookup: directory → Set of test basenames in that directory
    /** @type {Map<string, Set<string>>} */
    const dir_tests = new Map();
    for (const tf of test_files) {
        const d = dirname(tf);
        if (!dir_tests.has(d)) dir_tests.set(d, new Set());
        dir_tests.get(d).add(basename(tf));
    }

    /** @type {Map<string, string[]>} */
    const witnessed = new Map();
    /** @type {string[]} */
    const unattested = [];

    for (const src_path of source_files) {
        const src_dir = dirname(src_path);
        const src_base = basename(src_path);
        const src_name_no_ext = src_base.replace(/\.(ts|tsx|js|jsx)$/, '');
        const tests_in_dir = dir_tests.get(src_dir) || new Set();
        const matches = [];

        // Check all valid test file patterns for this source
        for (const ext of ['.ts', '.tsx', '.js', '.jsx']) {
            const candidate = `${src_name_no_ext}.test${ext}`;
            if (tests_in_dir.has(candidate)) {
                matches.push(rel_posix(PROJECT_ROOT, join(src_dir, candidate)));
            }
            const spec_candidate = `${src_name_no_ext}.spec${ext}`;
            if (tests_in_dir.has(spec_candidate)) {
                matches.push(rel_posix(PROJECT_ROOT, join(src_dir, spec_candidate)));
            }
        }

        const rel_src = rel_posix(PROJECT_ROOT, src_path);
        if (matches.length > 0) {
            matches.sort();
            witnessed.set(rel_src, matches);
        } else {
            unattested.push(rel_src);
        }
    }

    return { witnessed, unattested };
}

// ─── cspClaims ──────────────────────────────────────────────────────────────

/**
 * Parse the CSP directive from tauri.conf.json and extract structured claims.
 * Also validates against known-dangerous patterns:
 *   - RFC 1918 wildcards (192.168.*, 10.*, 172.16–31.*)
 *   - Unquoted CSP keywords (self, unsafe-inline, etc.)
 *   - Overly-broad wildcards (* in connect-src, script-src, default-src)
 *
 * @returns {{ directives: Record<string, string[]>, violations: Array<{directive: string, value: string, reason: string}>, raw: string|null, devCsp: string|null }}
 */
function csp_claims() {
    if (!existsSync(TAURI_CONF_PATH)) {
        return { directives: {}, violations: [{ directive: 'N/A', value: 'N/A', reason: 'tauri.conf.json not found' }], raw: null, devCsp: null };
    }

    const conf = JSON.parse(readFileSync(TAURI_CONF_PATH, 'utf-8'));
    const raw_csp = conf?.app?.security?.csp || null;
    const dev_csp = conf?.app?.security?.devCsp || null;
    const result = { directives: {}, violations: [], raw: raw_csp, devCsp: dev_csp };

    const csp_strings = [];
    if (raw_csp) csp_strings.push({ label: 'csp', value: raw_csp });
    if (dev_csp) csp_strings.push({ label: 'devCsp', value: dev_csp });

    for (const { label, value } of csp_strings) {
        const parsed = parse_csp_string(value);
        for (const [directive, values] of Object.entries(parsed)) {
            if (label === 'csp') {
                result.directives[directive] = values;
            }
            // Validate each value in the directive
            for (const v of values) {
                // RFC 1918 wildcard check
                if (/^https?:\/\/(192\.168|10\.|172\.(1[6-9]|2[0-9]|3[01]))\.?\*/.test(v)) {
                    result.violations.push({
                        directive: `${label}:${directive}`,
                        value: v,
                        reason: 'RFC 1918 private-range wildcard — allows binding to arbitrary local services',
                    });
                }

                // Unquoted keyword check: CSP keywords must be single-quoted
                const CSP_KEYWORDS = ['self', 'unsafe-inline', 'unsafe-eval', 'none', 'strict-dynamic',
                    'report-sample', 'unsafe-hashes', 'wasm-unsafe-eval'];
                if (CSP_KEYWORDS.includes(v)) {
                    result.violations.push({
                        directive: `${label}:${directive}`,
                        value: v,
                        reason: `CSP keyword must be single-quoted: '${v}'`,
                    });
                }

                // Overly-broad wildcard in sensitive directives
                const SENSITIVE_DIRECTIVES = ['default-src', 'script-src', 'connect-src', 'style-src', 'object-src'];
                if (v === '*' && SENSITIVE_DIRECTIVES.includes(directive)) {
                    result.violations.push({
                        directive: `${label}:${directive}`,
                        value: v,
                        reason: `Unrestricted wildcard in ${directive} — permits loading from any origin`,
                    });
                }
            }
        }
    }

    return result;
}

/**
 * Parse a raw CSP string into directive → values map.
 * @param {string} csp_string
 * @returns {Record<string, string[]>}
 */
function parse_csp_string(csp_string) {
    /** @type {Record<string, string[]>} */
    const result = {};
    const directives = csp_string.split(';').map(d => d.trim()).filter(Boolean);
    for (const directive of directives) {
        const parts = directive.split(/\s+/);
        const name = parts[0];
        result[name] = parts.slice(1);
    }
    return result;
}

// ─── generate ───────────────────────────────────────────────────────────────

/**
 * Compute the ADG manifest — the single source of truth for all derived claims.
 *
 * @param {string} target_dir - Directory to scan (default: src/).
 * @param {boolean} check - If true, diff against existing manifest and fail on drift.
 * @returns {{ manifest: object, drift: object|null, exit_code: number }}
 */
function generate(target_dir, check = false) {
    const dir = resolve(target_dir);

    // 1. Build witness map
    const { witnessed, unattested } = witness_map(dir);

    // 2. Build CSP claims
    const csp = csp_claims();

    // 3. Lint headers (verify no v1 contamination)
    const { violations: header_violations } = lint_headers(dir);

    // 4. Get git SHA for pinning
    let git_sha = 'unknown';
    try {
        git_sha = execSync('git rev-parse HEAD', { cwd: PROJECT_ROOT, encoding: 'utf-8' }).trim();
    } catch {
        // Not in a git repo or git not available
    }

    // Sort witness_map entries alphabetically for determinism
    const sorted_witness_map = Object.fromEntries(
        [...witnessed.entries()].sort(([a], [b]) => a.localeCompare(b))
    );

    // 5. Assemble manifest
    const manifest = {
        $schema: 'adg-manifest-v2',
        generated_at: new Date().toISOString(),
        verified_at: git_sha,
        generator_version: '2.0.0',
        claims: {
            witness_map: sorted_witness_map,
            csp: csp.directives,
            csp_raw: csp.raw,
            csp_dev_raw: csp.devCsp,
            csp_violations: csp.violations,
        },
        unattested: unattested.sort(),
        unattested_ratchet_ceiling: unattested.length,
        v1_contamination: header_violations.map(v => ({ file: to_posix(v.file), line: v.line })),
        stats: {
            witnessed_files: witnessed.size,
            unattested_files: unattested.length,
            total_source_files: witnessed.size + unattested.length,
            csp_directives: Object.keys(csp.directives).length,
            csp_violations: csp.violations.length,
            v1_violations: header_violations.length,
        },
    };

    // 6. If --check, diff against existing manifest
    let drift = null;
    let exit_code = 0;

    if (check) {
        if (!existsSync(MANIFEST_PATH)) {
            drift = { error: 'No existing adg.manifest.json found — run `adg generate` first.' };
            exit_code = 1;
        } else {
            const existing = JSON.parse(readFileSync(MANIFEST_PATH, 'utf-8'));
            drift = diff_manifests(existing, manifest);
            if (drift.has_drift) {
                exit_code = 1;
            }
        }
    } else {
        // Write the manifest
        writeFileSync(MANIFEST_PATH, JSON.stringify(manifest, null, 2) + '\n', 'utf-8');
    }

    return { manifest, drift, exit_code };
}

/**
 * Normalize manifest paths and ordering for cross-platform comparison.
 * @param {object} m
 * @returns {object}
 */
function normalize_manifest_paths(m) {
    if (!m) return m;
    const normalized = JSON.parse(JSON.stringify(m));
    if (normalized.claims?.witness_map) {
        const sorted = {};
        for (const [k, v] of Object.entries(normalized.claims.witness_map)) {
            const norm_k = to_posix(k);
            const norm_v = Array.isArray(v) ? v.map(to_posix).sort() : v;
            sorted[norm_k] = norm_v;
        }
        normalized.claims.witness_map = Object.fromEntries(
            Object.entries(sorted).sort(([a], [b]) => a.localeCompare(b))
        );
    }
    if (Array.isArray(normalized.unattested)) {
        normalized.unattested = normalized.unattested.map(to_posix).sort();
    }
    return normalized;
}

/**
 * Diff two manifests, checking only `claims` and `unattested` (not `verified_at`).
 * Also enforces the ratchet: unattested count must not increase.
 *
 * @param {object} existing - The on-disk manifest.
 * @param {object} fresh - The freshly computed manifest.
 * @returns {{ has_drift: boolean, details: string[] }}
 */
function diff_manifests(existing, fresh) {
    const details = [];
    const norm_existing = normalize_manifest_paths(existing);
    const norm_fresh = normalize_manifest_paths(fresh);

    // Claims diff (witness_map)
    const old_witnesses = JSON.stringify(norm_existing.claims?.witness_map || {});
    const new_witnesses = JSON.stringify(norm_fresh.claims?.witness_map || {});
    if (old_witnesses !== new_witnesses) {
        details.push('DRIFT: witness_map has changed. Re-run `adg generate` to update.');
    }

    // Claims diff (CSP)
    const old_csp = JSON.stringify(norm_existing.claims?.csp || {});
    const new_csp = JSON.stringify(norm_fresh.claims?.csp || {});
    if (old_csp !== new_csp) {
        details.push('DRIFT: CSP directives have changed. Re-run `adg generate` to update.');
    }

    // Unattested diff
    const old_unattested = JSON.stringify(norm_existing.unattested || []);
    const new_unattested = JSON.stringify(norm_fresh.unattested || []);
    if (old_unattested !== new_unattested) {
        details.push('DRIFT: unattested file list has changed. Re-run `adg generate` to update.');
    }

    // Ratchet enforcement: unattested count must not increase
    const old_ceiling = norm_existing.unattested_ratchet_ceiling ?? Infinity;
    const new_count = norm_fresh.unattested?.length ?? 0;
    if (new_count > old_ceiling) {
        details.push(
            `RATCHET: unattested count ${new_count} exceeds ceiling ${old_ceiling}. ` +
            `New source files must have co-located tests.`
        );
    }

    // v1 contamination must be zero
    if (fresh.v1_contamination?.length > 0) {
        details.push(
            `V1_CONTAMINATION: ${fresh.v1_contamination.length} file(s) still contain forbidden v1 fact-fields. ` +
            `Run \`adg codemod --write\` to strip them.`
        );
    }

    return { has_drift: details.length > 0, details };
}

// ─── verify ─────────────────────────────────────────────────────────────────

/**
 * Run all verification gates:
 *   1. Must-fail fixtures (if fixtures dir exists)
 *   2. Live header lint
 *   3. CSP grammar validation
 *   4. Manifest integrity (generate --check)
 *
 * @param {string} target_dir - Source directory.
 * @returns {{ passed: boolean, results: Array<{gate: string, passed: boolean, details: string}> }}
 */
function verify(target_dir) {
    const results = [];
    let all_passed = true;

    // Gate 1: Must-fail fixtures
    if (existsSync(FIXTURES_DIR)) {
        const fixture_result = run_must_fail_fixtures();
        results.push(fixture_result);
        if (!fixture_result.passed) all_passed = false;
    } else {
        results.push({ gate: 'must-fail-fixtures', passed: true, details: 'No fixtures directory found (skipped).' });
    }

    // Gate 2: Live header lint
    const { violations, scanned } = lint_headers(target_dir);
    const lint_passed = violations.length === 0;
    results.push({
        gate: 'lint-headers',
        passed: lint_passed,
        details: lint_passed
            ? `${scanned} files scanned, 0 v1 violations.`
            : `${violations.length} v1 violation(s) in ${scanned} files:\n` +
              violations.map(v => `  ${v.file}:${v.line} → ${v.content}`).join('\n'),
    });
    if (!lint_passed) all_passed = false;

    // Gate 3: CSP grammar
    const csp = csp_claims();
    const csp_passed = csp.violations.length === 0;
    results.push({
        gate: 'csp-grammar',
        passed: csp_passed,
        details: csp_passed
            ? `CSP parsed, ${Object.keys(csp.directives).length} directives, 0 violations.`
            : `${csp.violations.length} CSP violation(s):\n` +
              csp.violations.map(v => `  [${v.directive}] ${v.value} — ${v.reason}`).join('\n'),
    });
    if (!csp_passed) all_passed = false;

    // Gate 4: Manifest integrity (only if manifest exists)
    if (existsSync(MANIFEST_PATH)) {
        const { drift, exit_code } = generate(target_dir, true);
        const manifest_passed = exit_code === 0;
        results.push({
            gate: 'manifest-integrity',
            passed: manifest_passed,
            details: manifest_passed
                ? 'Manifest is current — no drift detected.'
                : `Manifest drift detected:\n` + (drift?.details || []).map(d => `  ${d}`).join('\n'),
        });
        if (!manifest_passed) all_passed = false;
    } else {
        results.push({
            gate: 'manifest-integrity',
            passed: true,
            details: 'No adg.manifest.json found (skipped — run `adg generate` first).',
        });
    }

    return { passed: all_passed, results };
}

/**
 * Run must-fail fixtures. Each fixture file must cause lint-headers to flag violations.
 * If a fixture passes lint (no violations found), the fixture is considered broken.
 *
 * @returns {{ gate: string, passed: boolean, details: string }}
 */
function run_must_fail_fixtures() {
    const fixture_files = walk_tree(FIXTURES_DIR, VALID_SOURCE_EXTS);
    const failures = [];

    for (const fixture_path of fixture_files) {
        const fixture_content = readFileSync(fixture_path, 'utf-8');
        const lines = fixture_content.split('\n');
        let found_violation = false;

        let in_jsdoc = false;
        for (const line of lines) {
            if (line.includes('/**')) in_jsdoc = true;
            if (in_jsdoc) {
                for (const pattern of V1_FORBIDDEN_PATTERNS) {
                    if (pattern.test(line)) {
                        found_violation = true;
                        break;
                    }
                }
            }
            if (found_violation) break;
            if (line.includes('*/')) break;
        }

        if (!found_violation) {
            failures.push(rel_posix(PROJECT_ROOT, fixture_path));
        }
    }

    const passed = failures.length === 0 && fixture_files.length > 0;
    return {
        gate: 'must-fail-fixtures',
        passed,
        details: passed
            ? `${fixture_files.length} fixture(s) correctly detected as violations.`
            : fixture_files.length === 0
                ? 'No fixture files found in fixtures/must-fail/.'
                : `${failures.length} fixture(s) failed to trigger lint violations:\n` +
                  failures.map(f => `  ${f}`).join('\n'),
    };
}

// ─── CLI Entry Point ────────────────────────────────────────────────────────

function main() {
    const args = process.argv.slice(2);
    const command = args[0];

    if (!command || command === '--help' || command === '-h') {
        print_usage();
        process.exit(0);
    }

    const default_dir = resolve(PROJECT_ROOT, 'src');

    switch (command) {
        case 'lint-headers': {
            const target = args[1] ? resolve(args[1]) : default_dir;
            console.log(`[ADG] lint-headers: scanning ${rel_posix(PROJECT_ROOT, target)}/`);
            const { violations, scanned } = lint_headers(target);
            if (violations.length === 0) {
                console.log(`[ADG] ✓ ${scanned} files scanned, 0 v1 violations.`);
                process.exit(0);
            } else {
                console.error(`[ADG] ✗ ${violations.length} v1 violation(s) found:`);
                for (const v of violations) {
                    console.error(`  ${v.file}:${v.line} → ${v.content}`);
                }
                process.exit(1);
            }
            break;
        }

        case 'codemod': {
            const write = args.includes('--write');
            const target = args.filter(a => a !== '--write')[1] || default_dir;
            const resolved_target = resolve(target);
            console.log(`[ADG] codemod${write ? ' (WRITE)' : ' (DRY-RUN)'}: scanning ${rel_posix(PROJECT_ROOT, resolved_target)}/`);
            const { modified, scanned } = codemod(resolved_target, write);
            if (modified.length === 0) {
                console.log(`[ADG] ✓ ${scanned} files scanned, 0 files require changes.`);
            } else {
                const verb = write ? 'modified' : 'would modify';
                console.log(`[ADG] ${write ? '✓' : '⚠'} ${modified.length} file(s) ${verb}:`);
                for (const m of modified) {
                    console.log(`  ${m.file} (${m.removed} line(s) removed)`);
                }
                if (!write) {
                    console.log('\n  Re-run with --write to apply changes.');
                }
            }
            process.exit(0);
            break;
        }

        case 'generate': {
            const check = args.includes('--check');
            const target = args.filter(a => a !== '--check')[1] || default_dir;
            const resolved_target = resolve(target);
            console.log(`[ADG] generate${check ? ' --check' : ''}: scanning ${rel_posix(PROJECT_ROOT, resolved_target)}/`);
            const { manifest, drift, exit_code } = generate(resolved_target, check);
            if (check) {
                if (exit_code === 0) {
                    console.log('[ADG] ✓ Manifest is current — no drift detected.');
                } else {
                    console.error('[ADG] ✗ Manifest drift detected:');
                    for (const d of drift.details) {
                        console.error(`  ${d}`);
                    }
                }
            } else {
                console.log(`[ADG] ✓ Generated adg.manifest.json`);
                console.log(`  Witnessed: ${manifest.stats.witnessed_files} files`);
                console.log(`  Unattested: ${manifest.stats.unattested_files} files`);
                console.log(`  CSP directives: ${manifest.stats.csp_directives}`);
                console.log(`  CSP violations: ${manifest.stats.csp_violations}`);
                console.log(`  V1 contamination: ${manifest.stats.v1_violations}`);
            }
            process.exit(exit_code);
            break;
        }

        case 'verify': {
            const target = args[1] ? resolve(args[1]) : default_dir;
            console.log(`[ADG] verify: running all gates on ${rel_posix(PROJECT_ROOT, target)}/`);
            console.log('');
            const { passed, results } = verify(target);
            for (const r of results) {
                const icon = r.passed ? '✓' : '✗';
                console.log(`  [${icon}] ${r.gate}: ${r.details.split('\n')[0]}`);
                if (!r.passed) {
                    // Print multi-line details for failures
                    const extra_lines = r.details.split('\n').slice(1);
                    for (const line of extra_lines) {
                        console.log(`      ${line}`);
                    }
                }
            }
            console.log('');
            if (passed) {
                console.log('[ADG] ✓ All gates passed.');
            } else {
                console.error('[ADG] ✗ Verification failed — see details above.');
            }
            process.exit(passed ? 0 : 1);
            break;
        }

        default:
            console.error(`[ADG] Unknown command: ${command}`);
            print_usage();
            process.exit(1);
    }
}

function print_usage() {
    console.log(`
ADG v2 — Active Documentation Guard

Usage:
  node tools/adg/adg.mjs <command> [options] [dir]

Commands:
  lint-headers [dir]         Flag forbidden v1 fact-fields in JSDoc headers.
  codemod [--write] [dir]    Strip v1 fact-fields. Dry-run by default.
  generate [--check] [dir]   Compute adg.manifest.json. --check diffs claims & ratchet.
  verify [dir]               Run all verification gates.

Examples:
  node tools/adg/adg.mjs lint-headers src/
  node tools/adg/adg.mjs codemod --write src/
  node tools/adg/adg.mjs generate
  node tools/adg/adg.mjs generate --check
  node tools/adg/adg.mjs verify
`);
}

main();
