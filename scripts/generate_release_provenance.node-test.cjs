/**
 * @docs ARCHITECTURE:Infrastructure:Execution
 *
 * ### AI Context Alignment
 * - **Subsystem**: Release Governance / Provenance Tests
 * - **Primary Entrypoints**: Node test runner
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - Provenance hashes are computed from isolated fixture artifacts.
 *
 * ### 🔍 Debugging & Observability
 * - **Witness Target**: `scripts/generate_release_provenance.cjs`
 */

'use strict';

const assert = require('node:assert/strict');
const crypto = require('node:crypto');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const test = require('node:test');

const { generateManifest, parseArgs } = require('./generate_release_provenance.cjs');

test('provenance binds both source commits and hashes every release artifact', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'tadpole-provenance-'));
  const artifactDir = path.join(root, 'artifacts');
  fs.mkdirSync(artifactDir);
  const artifact = path.join(artifactDir, 'tadpole.deb');
  const sanitizer = path.join(root, 'publish-public.ps1');
  const output = path.join(artifactDir, 'release-provenance.json');
  fs.writeFileSync(artifact, 'artifact-bytes');
  fs.writeFileSync(sanitizer, 'sanitizer-rules');

  const manifest = generateManifest({
    version: '1.2.3',
    tag: 'v1.2.3',
    'private-sha': 'a'.repeat(40),
    'public-sha': 'b'.repeat(40),
    'run-id': '42',
    sanitizer,
    artifacts: artifactDir,
    output,
  });

  assert.equal(manifest.artifacts.length, 1);
  assert.equal(manifest.artifacts[0].sha256, crypto.createHash('sha256').update('artifact-bytes').digest('hex'));
  assert.equal(JSON.parse(fs.readFileSync(output, 'utf8')).sanitized_public_sha, 'b'.repeat(40));
});

test('provenance rejects a tag/version mismatch', () => {
  assert.throws(
    () => parseArgs(['--version', '1.2.3', '--tag', 'v1.2.4', '--private-sha', 'a'.repeat(40), '--public-sha', 'b'.repeat(40), '--run-id', '1', '--sanitizer', 'x', '--artifacts', 'y', '--output', 'z']),
    /does not match/,
  );
});
