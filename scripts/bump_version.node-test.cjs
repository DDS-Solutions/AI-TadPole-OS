/**
 * @docs ARCHITECTURE:Infrastructure:Execution
 *
 * ### AI Context Alignment
 * - **Subsystem**: Release Governance / Version Authority Tests
 * - **Primary Entrypoints**: Node test runner
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - Every test operates on an isolated temporary repository fixture.
 *
 * ### 🔍 Debugging & Observability
 * - **Witness Target**: `scripts/bump_version.cjs`
 */

'use strict';

const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const test = require('node:test');

const { TARGET_PATHS, parseArgs, run } = require('./bump_version.cjs');

const REPOSITORY_ROOT = path.resolve(__dirname, '..');

function fixture() {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'tadpole-version-'));
  for (const relativePath of TARGET_PATHS) {
    const destination = path.join(root, relativePath);
    fs.mkdirSync(path.dirname(destination), { recursive: true });
    fs.copyFileSync(path.join(REPOSITORY_ROOT, relativePath), destination);
  }
  return root;
}

function options(overrides = {}) {
  return {
    check: false,
    bump: null,
    androidCode: null,
    apiVersion: null,
    ...overrides,
  };
}

test('the committed version surface is aligned', () => {
  const root = fixture();
  assert.doesNotThrow(() => run({ rootDir: root, options: options({ check: true }) }));
});

test('check mode reports drift without changing files', () => {
  const root = fixture();
  const file = path.join(root, 'package.json');
  const value = JSON.parse(fs.readFileSync(file, 'utf8'));
  value.version = '9.9.9';
  const drifted = `${JSON.stringify(value, null, 2)}\n`;
  fs.writeFileSync(file, drifted);

  assert.throws(
    () => run({ rootDir: root, options: options({ check: true }) }),
    (error) => error.code === 'VERSION:DRIFT' && error.message.includes('package.json'),
  );
  assert.equal(fs.readFileSync(file, 'utf8'), drifted);
});

test('a product bump updates every package surface and increments Android once', () => {
  const root = fixture();
  const result = run({
    rootDir: root,
    options: options({ bump: '1.1.999' }),
    today: '2026-08-25',
  });

  assert.equal(result.authority.version, '1.1.999');
  assert.equal(result.authority.android_version_code, 3);
  assert.equal(result.authority.api_document_version, '1.1.462');
  assert.match(fs.readFileSync(path.join(root, 'apps/mobile-android/app/build.gradle.kts'), 'utf8'), /versionCode = 3[\s\S]*versionName = "1\.1\.999"/);
  assert.equal(JSON.parse(fs.readFileSync(path.join(root, 'package-lock.json'), 'utf8')).packages[''].version, '1.1.999');
  assert.match(fs.readFileSync(path.join(root, 'docs/openapi.yaml'), 'utf8'), /version: 1\.1\.462/);
  assert.doesNotThrow(() => run({ rootDir: root, options: options({ check: true }) }));
});

test('repeating the current version is idempotent', () => {
  const root = fixture();
  const current = JSON.parse(fs.readFileSync(path.join(root, 'version.json'), 'utf8'));
  const result = run({ rootDir: root, options: options({ bump: current.version }), today: '2099-01-01' });

  assert.deepEqual(result.changed, []);
  assert.equal(result.authority.android_version_code, current.android_version_code);
  assert.equal(result.authority.version_updated_at, current.version_updated_at);
});

test('missing or malformed targets fail during preflight with no partial writes', () => {
  const missingRoot = fixture();
  fs.rmSync(path.join(missingRoot, 'README.md'));
  assert.throws(
    () => run({ rootDir: missingRoot, options: options({ bump: '1.1.999' }) }),
    (error) => error.code === 'VERSION:INVALID' && error.message.includes('README.md'),
  );

  const malformedRoot = fixture();
  const packageFile = path.join(malformedRoot, 'package.json');
  const before = fs.readFileSync(packageFile, 'utf8');
  fs.writeFileSync(path.join(malformedRoot, 'README.md'), '# no version marker\n');
  assert.throws(
    () => run({ rootDir: malformedRoot, options: options({ bump: '1.1.999' }) }),
    (error) => error.code === 'VERSION:INVALID' && error.message.includes('README.md'),
  );
  assert.equal(fs.readFileSync(packageFile, 'utf8'), before);
});

test('strict SemVer validation rejects leading zeroes and accepts prereleases', () => {
  assert.throws(() => parseArgs(['--bump', '01.2.3']), /invalid product SemVer/);
  assert.equal(parseArgs(['--bump', '2.0.0-rc.1+build.7']).bump, '2.0.0-rc.1+build.7');
});
