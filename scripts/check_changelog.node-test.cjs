/**
 * @docs ARCHITECTURE:Infrastructure:Execution
 *
 * ### AI Context Alignment
 * - **Subsystem**: Release Governance / Changelog Guard Tests
 * - **Primary Entrypoints**: Node test runner
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - Published fixture sections remain byte-for-byte immutable.
 *
 * ### 🔍 Debugging & Observability
 * - **Witness Target**: `scripts/check_changelog.cjs`
 */
'use strict';

const assert = require('node:assert/strict');
const test = require('node:test');
const { compareRevisions, parseChangelog } = require('./check_changelog.cjs');

const base = `# Changelog\n\n## [Unreleased]\n\n### Changed\n- Pending.\n\n## [1.2.0] - 2026-08-01\n\n### Added\n- Stable.\n\n## [1.1.0] - 2026-07-01\n\n### Added\n- Older.\n`;

test('accepts one Unreleased section followed by descending unique releases', () => {
  assert.deepEqual(parseChangelog(base).releases, ['1.2.0', '1.1.0']);
});

test('orders stable releases ahead of their prereleases', () => {
  const prereleases = `# Changelog\n\n## [Unreleased]\n\n## [2.0.0] - 2026-08-02\n\n## [2.0.0-rc.1] - 2026-08-01\n`;
  assert.deepEqual(parseChangelog(prereleases).releases, ['2.0.0', '2.0.0-rc.1']);
});

test('rejects duplicate releases', () => {
  assert.throws(() => parseChangelog(`${base}\n## [1.2.0] - 2026-08-02\n`), /duplicate/);
});

test('allows Unreleased edits while product version is unchanged', () => {
  const head = base.replace('- Pending.', '- Pending work updated.');
  assert.deepEqual(compareRevisions({ baseChangelog: base, headChangelog: head, baseVersion: '1.2.0', headVersion: '1.2.0' }).added, []);
});

test('requires immutable published sections', () => {
  const head = base.replace('- Stable.', '- Rewritten.');
  assert.throws(
    () => compareRevisions({ baseChangelog: base, headChangelog: head, baseVersion: '1.2.0', headVersion: '1.2.0' }),
    (error) => error.code === 'CHANGELOG:IMMUTABLE',
  );
});

test('requires a single matching release section for a product bump', () => {
  const released = base.replace('## [Unreleased]', '## [Unreleased]\n\n## [1.3.0] - 2026-08-25');
  assert.deepEqual(compareRevisions({ baseChangelog: base, headChangelog: released, baseVersion: '1.2.0', headVersion: '1.3.0' }).added, ['1.3.0']);
  assert.throws(
    () => compareRevisions({ baseChangelog: base, headChangelog: released, baseVersion: '1.2.0', headVersion: '1.2.0' }),
    /version is unchanged/,
  );
});
