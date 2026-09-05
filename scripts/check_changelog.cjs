#!/usr/bin/env node
/**
 * @docs ARCHITECTURE:Infrastructure:Execution
 *
 * ### AI Context Alignment
 * - **Subsystem**: Release Governance / Changelog Integrity
 * - **Primary Entrypoints**: `parseChangelog`, `compareRevisions`, `main`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - A single Unreleased section is always first.
 * - Published release sections are unique and immutable.
 * - A product-version transition introduces exactly one matching release section.
 *
 * ### Debugging & Observability
 * - **Local Errors**: `[CHANGELOG:INVALID]`, `[CHANGELOG:IMMUTABLE]`
 * - **Witness Tests**: `scripts/check_changelog.node-test.cjs`
 */

'use strict';

const fs = require('node:fs');
const path = require('node:path');
const { execFileSync } = require('node:child_process');
const { SEMVER } = require('./bump_version.cjs');

class ChangelogError extends Error {
  constructor(code, message) {
    super(message);
    this.code = code;
  }
}

function compareSemver(left, right) {
  const [leftWithoutBuild] = left.split('+');
  const [rightWithoutBuild] = right.split('+');
  const [leftCore, leftPrerelease] = leftWithoutBuild.split('-', 2);
  const [rightCore, rightPrerelease] = rightWithoutBuild.split('-', 2);
  const a = leftCore.split('.').map(Number);
  const b = rightCore.split('.').map(Number);
  for (let index = 0; index < 3; index += 1) {
    if (a[index] !== b[index]) return b[index] - a[index];
  }
  if (!leftPrerelease && !rightPrerelease) return 0;
  if (!leftPrerelease) return -1;
  if (!rightPrerelease) return 1;

  const leftParts = leftPrerelease.split('.');
  const rightParts = rightPrerelease.split('.');
  const count = Math.max(leftParts.length, rightParts.length);
  for (let index = 0; index < count; index += 1) {
    if (leftParts[index] === undefined) return 1;
    if (rightParts[index] === undefined) return -1;
    if (leftParts[index] === rightParts[index]) continue;
    const leftNumeric = /^\d+$/.test(leftParts[index]);
    const rightNumeric = /^\d+$/.test(rightParts[index]);
    if (leftNumeric && rightNumeric) return Number(rightParts[index]) - Number(leftParts[index]);
    if (leftNumeric) return 1;
    if (rightNumeric) return -1;
    return leftParts[index] > rightParts[index] ? -1 : 1;
  }
  return 0;
}

function parseChangelog(content, label = 'CHANGELOG.md') {
  const header = /^## \[([^\]]+)\](?: - (\d{4}-\d{2}-\d{2}))?\s*$/gm;
  const matches = [...content.matchAll(header)];
  if (!matches.length) throw new ChangelogError('CHANGELOG:INVALID', `${label}: no release sections found`);
  if (matches[0][1] !== 'Unreleased' || matches[0][2]) {
    throw new ChangelogError('CHANGELOG:INVALID', `${label}: [Unreleased] must be the first undated section`);
  }

  const sections = new Map();
  for (let index = 0; index < matches.length; index += 1) {
    const match = matches[index];
    const name = match[1];
    const date = match[2] || null;
    if (sections.has(name)) throw new ChangelogError('CHANGELOG:INVALID', `${label}: duplicate [${name}] section`);
    if (name !== 'Unreleased' && (!SEMVER.test(name) || !date)) {
      throw new ChangelogError('CHANGELOG:INVALID', `${label}: released section [${name}] requires SemVer and a date`);
    }
    const end = index + 1 < matches.length ? matches[index + 1].index : content.length;
    sections.set(name, { date, body: content.slice(match.index, end).trimEnd() });
  }

  const releases = [...sections.keys()].filter((name) => name !== 'Unreleased');
  const sorted = [...releases].sort(compareSemver);
  if (JSON.stringify(releases) !== JSON.stringify(sorted)) {
    throw new ChangelogError('CHANGELOG:INVALID', `${label}: release sections must be newest first`);
  }
  return { sections, releases };
}

function compareRevisions({ baseChangelog, headChangelog, baseVersion, headVersion }) {
  const base = parseChangelog(baseChangelog, 'base CHANGELOG.md');
  const head = parseChangelog(headChangelog, 'head CHANGELOG.md');

  for (const release of base.releases) {
    const prior = base.sections.get(release);
    const current = head.sections.get(release);
    if (!current || current.body !== prior.body) {
      throw new ChangelogError('CHANGELOG:IMMUTABLE', `released section [${release}] was changed or removed`);
    }
  }

  const added = head.releases.filter((release) => !base.sections.has(release));
  if (baseVersion === headVersion) {
    if (added.length) {
      throw new ChangelogError('CHANGELOG:INVALID', `product version is unchanged but released section(s) were added: ${added.join(', ')}`);
    }
  } else if (added.length !== 1 || added[0] !== headVersion) {
    throw new ChangelogError('CHANGELOG:INVALID', `version ${baseVersion} -> ${headVersion} requires exactly one new [${headVersion}] section`);
  }
  return { added };
}

function gitShow(rootDir, revision, file) {
  return execFileSync('git', ['show', `${revision}:${file}`], { cwd: rootDir, encoding: 'utf8' });
}

function baseContainsGuard(rootDir, revision) {
  try {
    execFileSync('git', ['cat-file', '-e', `${revision}:scripts/check_changelog.cjs`], { cwd: rootDir, stdio: 'ignore' });
    return true;
  } catch {
    return false;
  }
}

function parseArgs(argv) {
  const result = { check: false, base: null, head: null, release: null };
  for (let index = 0; index < argv.length; index += 1) {
    if (argv[index] === '--check') result.check = true;
    else if (argv[index] === '--base') result.base = argv[++index];
    else if (argv[index] === '--head') result.head = argv[++index];
    else if (argv[index] === '--release') result.release = argv[++index];
    else throw new ChangelogError('CHANGELOG:INVALID', `unknown argument: ${argv[index]}`);
  }
  if (!result.check && !(result.base && result.head)) {
    throw new ChangelogError('CHANGELOG:INVALID', 'use --check or provide --base SHA --head SHA');
  }
  if (result.release && !SEMVER.test(result.release)) {
    throw new ChangelogError('CHANGELOG:INVALID', '--release must be valid SemVer');
  }
  return result;
}

function main() {
  const rootDir = path.resolve(__dirname, '..');
  try {
    const options = parseArgs(process.argv.slice(2));
    const current = fs.readFileSync(path.join(rootDir, 'CHANGELOG.md'), 'utf8');
    const parsed = parseChangelog(current);
    if (options.release && !parsed.sections.has(options.release)) {
      throw new ChangelogError('CHANGELOG:INVALID', `release [${options.release}] is missing from CHANGELOG.md`);
    }

    if (options.base && options.head && baseContainsGuard(rootDir, options.base)) {
      compareRevisions({
        baseChangelog: gitShow(rootDir, options.base, 'CHANGELOG.md'),
        headChangelog: gitShow(rootDir, options.head, 'CHANGELOG.md'),
        baseVersion: JSON.parse(gitShow(rootDir, options.base, 'version.json')).version,
        headVersion: JSON.parse(gitShow(rootDir, options.head, 'version.json')).version,
      });
    }
    console.log('[CHANGELOG:OK] structure and published-section immutability checks passed');
  } catch (error) {
    console.error(`[${error.code || 'CHANGELOG:INVALID'}] ${error.message}`);
    process.exitCode = 1;
  }
}

if (require.main === module) main();

module.exports = { ChangelogError, compareRevisions, parseChangelog };
