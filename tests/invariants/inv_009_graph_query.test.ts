/**
 * @docs ARCHITECTURE:CodeBaseIntelligence
 *
 * ### AI Context Alignment
 * - **Subsystem**: Invariant Verification Suite / inv_009_graph_query.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 *
 * INV-009: Code Symbol Graph & Active Documentation Guard (ADG-SG) Invariant Suite
 *
 * Asserts structural & behavioral guarantees in server-rs/src/bin/graph_query/:
 *   1. Path Traversal Oracle Elimination: filter_symbol rejects `..`, drive colons, root slashes, and null bytes
 *   2. Strict Containment in disk_candidate_exists: starts_with(canonical_root) and is_file()
 *   3. Fix Loop CAS Staleness Guard: content hash verification and atomic file rename (.docguard.tmp)
 *   4. Occurrence-Aware Symbol Accounting: eliminates false green clearance on multi-occurrence symbols
 *   5. Unconditional Keyword Allowlist: BUILTIN_KEYWORDS never shadowed by globals.json
 *   6. Subdirectory Diff Rebase: rev-parse --show-prefix stripping and strict empty-changeset error
 *   7. Bounded & Sandboxed Git Execution: --no-optional-locks, -c core.fsmonitor=false, timeout reaping
 *   8. Visualizer Security: Cytoscape SRI hash integrity pinning and U+2028/U+2029 JS escaping
 *   9. PageRank Isolation: segment-based boost matching and total_cmp sorting
 */

import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

describe('INV-009: Graph Query & Active Documentation Guard Invariants', () => {
    const docGuardSource = readFileSync(
        resolve('server-rs/src/bin/graph_query/doc_guard.rs'),
        'utf-8'
    );
    const pathUtilsSource = readFileSync(
        resolve('server-rs/src/bin/graph_query/path_utils.rs'),
        'utf-8'
    );
    const visualizerSource = readFileSync(
        resolve('server-rs/src/bin/graph_query/visualizer.rs'),
        'utf-8'
    );
    const queryManagerSource = readFileSync(
        resolve('server-rs/src/bin/graph_query/query_manager.rs'),
        'utf-8'
    );

    it('asserts path traversal oracle defense in filter_symbol and disk_candidate_exists', () => {
        // filter_symbol checks
        expect(docGuardSource).toContain("s_clean.contains('\\0') || s_clean.contains(':')");
        expect(docGuardSource).toContain("s_clean.starts_with('/') || s_clean.starts_with('\\\\') || Path::new(s_clean).is_absolute()");
        expect(docGuardSource).toContain("seg == \"..\"");

        // disk_candidate_exists containment
        expect(docGuardSource).toContain("pub fn disk_candidate_exists(");
        expect(docGuardSource).toContain(".starts_with(&canonical_root)");
        expect(docGuardSource).toContain(".is_file()");
    });

    it('asserts CAS staleness verification and atomic rename in auto-fix loop', () => {
        expect(docGuardSource).toContain("content changed on disk after scan; refusing to auto-fix");
        expect(docGuardSource).toContain(".with_extension(\"docguard.tmp\")");
        expect(docGuardSource).toContain("fs::rename(&tmp_path, &full_path)");
        // Must compute SHA256 digest
        expect(docGuardSource).toContain("Sha256::new()");
    });

    it('asserts occurrence-aware accounting for fixed symbols', () => {
        expect(docGuardSource).toContain("applied_occurrences: HashMap<String, usize>");
        expect(docGuardSource).toContain("*count -= 1;");
        // Must NOT use blanket HashSet contains for retention
        expect(docGuardSource).not.toContain("missing_symbols.retain(|s| !applied_fixed_symbols.contains(s));");
    });

    it('asserts unconditional inclusion of BUILTIN_KEYWORDS regardless of globals.json', () => {
        expect(docGuardSource).toContain("pub const BUILTIN_KEYWORDS: &[&str]");
        expect(docGuardSource).toContain("for kw in BUILTIN_KEYWORDS {");
        expect(docGuardSource).toContain("whitelist.insert(kw.to_string());");
        // Must not gate keyword fallback behind whitelist.is_empty()
        expect(docGuardSource).not.toContain("if whitelist.is_empty() {");
    });

    it('asserts git diff rebase and strict error on empty changeset', () => {
        expect(pathUtilsSource).toContain("pub fn get_git_repo_prefix(root: &Path)");
        expect(pathUtilsSource).toContain("rev-parse");
        expect(pathUtilsSource).toContain("--show-prefix");
        expect(pathUtilsSource).toContain("path.strip_prefix(&prefix)");

        // Diff mode refusal on 0 files
        expect(docGuardSource).toContain("options.diff_only && total_files == 0");
        expect(docGuardSource).toContain("Diff mode matched 0 files in repository changeset");
    });

    it('asserts bounded, sandboxed git subprocess execution', () => {
        expect(pathUtilsSource).toContain("--no-optional-locks");
        expect(pathUtilsSource).toContain("core.fsmonitor=false");
        expect(pathUtilsSource).toContain("core.hooksPath=");
        expect(pathUtilsSource).toContain("GIT_TIMEOUT");
        expect(pathUtilsSource).toContain("child.kill()");
        expect(pathUtilsSource).toContain("child.wait()");
    });

    it('asserts SRI integrity hash and U+2028/U+2029 escaping in visualizer', () => {
        expect(visualizerSource).toContain("integrity=\"sha384-");
        expect(visualizerSource).toContain(".replace('\\u{2028}', \"\\\\u2028\")");
        expect(visualizerSource).toContain(".replace('\\u{2029}', \"\\\\u2029\")");
        expect(visualizerSource).toContain("pub fn mermaid_escape(");
    });

    it('asserts segment-based contract boost and total_cmp sorting in PageRank', () => {
        expect(queryManagerSource).toContain(".split(['/', '\\\\'])");
        expect(queryManagerSource).toContain("seg == \"routes\" || seg == \"commands\" || seg == \"schemas\"");
        expect(queryManagerSource).toContain("ranked.sort_by(|a, b| b.1.total_cmp(&a.1));");
        // Must not use loose substring contains on whole path
        expect(queryManagerSource).not.toContain("real_path.contains(\"routes\")");
    });
});
