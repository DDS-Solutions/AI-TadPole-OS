#!/usr/bin/env node
/**
 * @docs ARCHITECTURE:Infrastructure:Execution
 *
 * ### AI Context Alignment
 * - **Subsystem**: Release Governance / Version Authority
 * - **Primary Entrypoints**: `run`, `buildOutputs`, `parseArgs`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `version.json` is the sole product-version authority.
 * - Every declared target is required and must match exactly one structural rule.
 * - Android build codes are monotonic and only auto-increment for a new product version.
 * - OpenAPI document versioning is independent from the product release version.
 *
 * ### Debugging & Observability
 * - **Local Errors**: `[VERSION:INVALID]`, `[VERSION:DRIFT]`, `[VERSION:WRITE]`
 * - **Telemetry Targets**: concise stdout summary and actionable stderr diagnostics
 * - **Witness Tests**: `scripts/bump_version.node-test.cjs`
 */

'use strict';

const fs = require('node:fs');
const path = require('node:path');

const SEMVER_IDENTIFIER = '(?:0|[1-9]\\d*|[0-9A-Za-z-]*[A-Za-z-][0-9A-Za-z-]*)';
const SEMVER = new RegExp(
  `^(?:0|[1-9]\\d*)\\.(?:0|[1-9]\\d*)\\.(?:0|[1-9]\\d*)` +
  `(?:-${SEMVER_IDENTIFIER}(?:\\.${SEMVER_IDENTIFIER})*)?` +
  '(?:\\+[0-9A-Za-z-]+(?:\\.[0-9A-Za-z-]+)*)?$',
);

const TARGET_PATHS = Object.freeze([
  'version.json',
  'package.json',
  'package-lock.json',
  'server-rs/Cargo.toml',
  'server-rs/Cargo.lock',
  'src-tauri/Cargo.toml',
  'src-tauri/Cargo.lock',
  'src-tauri/tauri.conf.json',
  'apps/mobile-android/app/build.gradle.kts',
  'README.md',
  'directives/IDENTITY.md',
  'docs/.vitepress/config.js',
  'index.html',
  'docs/GETTING_STARTED.md',
  'docs/OPERATIONS_MANUAL.md',
  'docs/TROUBLESHOOTING.md',
  'docs/wiki/Home.md',
  'docs/wiki/_Footer.md',
  'docs/wiki/_Sidebar.md',
  'docs/openapi.yaml',
  'docs/API_REFERENCE.md',
  'docs/API_CONTRACT.md',
]);

class VersionError extends Error {
  constructor(code, message) {
    super(message);
    this.code = code;
  }
}

function detectNewline(content) {
  return content.includes('\r\n') ? '\r\n' : '\n';
}

function formatJson(value, original) {
  const newline = detectNewline(original);
  return `${JSON.stringify(value, null, 2).replace(/\n/g, newline)}${newline}`;
}

function parseJson(content, file) {
  try {
    return JSON.parse(content);
  } catch (error) {
    throw new VersionError('VERSION:INVALID', `${file}: invalid JSON (${error.message})`);
  }
}

function replaceExactly(content, pattern, replacement, file, expected = 1) {
  const flags = pattern.flags.includes('g') ? pattern.flags : `${pattern.flags}g`;
  const matcher = new RegExp(pattern.source, flags);
  const matches = [...content.matchAll(matcher)];
  if (matches.length !== expected) {
    throw new VersionError(
      'VERSION:INVALID',
      `${file}: expected ${expected} match(es) for ${pattern}, found ${matches.length}`,
    );
  }
  return content.replace(matcher, replacement);
}

function validateAuthority(authority, file = 'version.json') {
  if (!authority || typeof authority !== 'object') {
    throw new VersionError('VERSION:INVALID', `${file}: expected a JSON object`);
  }
  for (const field of ['version', 'api_document_version']) {
    if (typeof authority[field] !== 'string' || !SEMVER.test(authority[field])) {
      throw new VersionError('VERSION:INVALID', `${file}: ${field} must be valid SemVer`);
    }
  }
  if (!Number.isSafeInteger(authority.android_version_code) || authority.android_version_code < 1) {
    throw new VersionError('VERSION:INVALID', `${file}: android_version_code must be a positive integer`);
  }
  if (typeof authority.version_updated_at !== 'string' || !/^\d{4}-\d{2}-\d{2}$/.test(authority.version_updated_at)) {
    throw new VersionError('VERSION:INVALID', `${file}: version_updated_at must use YYYY-MM-DD`);
  }
}

function loadAll(rootDir) {
  const files = new Map();
  for (const relativePath of TARGET_PATHS) {
    const absolutePath = path.join(rootDir, relativePath);
    if (!fs.existsSync(absolutePath) || !fs.statSync(absolutePath).isFile()) {
      throw new VersionError('VERSION:INVALID', `${relativePath}: required version target is missing`);
    }
    files.set(relativePath, fs.readFileSync(absolutePath, 'utf8'));
  }
  return files;
}

function updateJsonVersion(content, file, version) {
  const value = parseJson(content, file);
  if (!Object.hasOwn(value, 'version') || typeof value.version !== 'string') {
    throw new VersionError('VERSION:INVALID', `${file}: top-level version string is required`);
  }
  value.version = version;
  return formatJson(value, content);
}

function updatePackageLock(content, version) {
  const file = 'package-lock.json';
  const value = parseJson(content, file);
  if (!value.packages || !value.packages['']) {
    throw new VersionError('VERSION:INVALID', `${file}: packages[\"\"] is required`);
  }
  if (typeof value.version !== 'string' || typeof value.packages[''].version !== 'string') {
    throw new VersionError('VERSION:INVALID', `${file}: root package version strings are required`);
  }
  value.version = version;
  value.packages[''].version = version;
  return formatJson(value, content);
}

function updateCargoToml(content, file, version) {
  return replaceExactly(content, /^version\s*=\s*"[^"]+"/m, `version = "${version}"`, file);
}

function updateCargoLock(content, file, packageName, version) {
  const escapedName = packageName.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const pattern = new RegExp(`(\\[\\[package\\]\\]\\r?\\nname = "${escapedName}"\\r?\\nversion = ")[^"]+(")`);
  return replaceExactly(content, pattern, `$1${version}$2`, file);
}

function updateProductText(content, file, version) {
  const rules = {
    'README.md': [/^(\s*\*\*Version\*\*: )\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?/m, `$1${version}`],
    'docs/.vitepress/config.js': [/("softwareVersion"\s*:\s*")([^"]+)(")/, `$1${version}$3`],
    'index.html': [/("softwareVersion"\s*:\s*")([^"]+)(")/, `$1${version}$3`],
    'docs/GETTING_STARTED.md': [/^(> \*\*Version\*\*: )[^\r\n]+/m, `$1${version}`],
    'docs/OPERATIONS_MANUAL.md': [/^(> \*\*Version\*\*: )[^\r\n]+/m, `$1${version}`],
    'docs/TROUBLESHOOTING.md': [/^(> \*\*Version\*\*: )[^\r\n]+/m, `$1${version}`],
    'docs/wiki/Home.md': [/^(version:\s*")[^"]+(")/m, `$1${version}$2`],
    'docs/wiki/_Sidebar.md': [/(\*Version: )\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?/, `$1${version}`],
  };
  const rule = rules[file];
  if (!rule) {
    throw new VersionError('VERSION:INVALID', `${file}: no product-document rule is registered`);
  }
  return replaceExactly(content, rule[0], rule[1], file);
}

function updateIdentity(content, version) {
  const file = 'directives/IDENTITY.md';
  const rules = [
    [/^(\*\*System Version\*\*: )\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?( \(Hardened\))/m, `$1${version}$2`],
    [/^(\*\*Operational Protocol\*\*: User-Agent: TadpoleOS\/)\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?/m, `$1${version}`],
    [/(identify as `User-Agent: TadpoleOS\/)\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?(`)/, `$1${version}$2`],
    [/^(- \*\*Version\*\*: )\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?/m, `$1${version}`],
    [/^(- \*\*User-Agent Header\*\*: `TadpoleOS\/)\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?(`)/m, `$1${version}$2`],
  ];
  return rules.reduce(
    (next, [pattern, replacement]) => replaceExactly(next, pattern, replacement, file),
    content,
  );
}

function updateWikiFooter(content, version) {
  return replaceExactly(
    content,
    /^\*\*AI-Tadpole-OS Wiki\*\*.*$/m,
    `**AI-Tadpole-OS Wiki** • Deployment: SMB Local Node • Release: ${version}`,
    'docs/wiki/_Footer.md',
  );
}

function updateApiText(content, file, version) {
  const rules = {
    'docs/openapi.yaml': [/^(  version:\s*)[^\r\n]+/m, `$1${version}`],
    'docs/API_REFERENCE.md': [/^(\*\*Version\*\*: )[^\r\n]+/m, `$1${version}`],
    'docs/API_CONTRACT.md': [
      /^!\[Version: [^\]]+\]\(https:\/\/img\.shields\.io\/badge\/Version-[^-\s)]+-blue\)$/m,
      `![Version: ${version}](https://img.shields.io/badge/Version-${version}-blue)`,
    ],
  };
  const rule = rules[file];
  return replaceExactly(content, rule[0], rule[1], file);
}

function buildOutputs(rootDir, authority, sourceFiles = loadAll(rootDir)) {
  validateAuthority(authority);
  const version = authority.version;
  const apiVersion = authority.api_document_version;
  const androidCode = authority.android_version_code;
  const output = new Map(sourceFiles);

  output.set('version.json', formatJson(authority, sourceFiles.get('version.json')));
  output.set('package.json', updateJsonVersion(sourceFiles.get('package.json'), 'package.json', version));
  output.set('package-lock.json', updatePackageLock(sourceFiles.get('package-lock.json'), version));
  output.set('src-tauri/tauri.conf.json', updateJsonVersion(sourceFiles.get('src-tauri/tauri.conf.json'), 'src-tauri/tauri.conf.json', version));
  output.set('server-rs/Cargo.toml', updateCargoToml(sourceFiles.get('server-rs/Cargo.toml'), 'server-rs/Cargo.toml', version));
  output.set('src-tauri/Cargo.toml', updateCargoToml(sourceFiles.get('src-tauri/Cargo.toml'), 'src-tauri/Cargo.toml', version));
  output.set('server-rs/Cargo.lock', updateCargoLock(sourceFiles.get('server-rs/Cargo.lock'), 'server-rs/Cargo.lock', 'server-rs', version));
  output.set('src-tauri/Cargo.lock', updateCargoLock(sourceFiles.get('src-tauri/Cargo.lock'), 'src-tauri/Cargo.lock', 'tadpole-os', version));

  let gradle = sourceFiles.get('apps/mobile-android/app/build.gradle.kts');
  gradle = replaceExactly(gradle, /(versionCode\s*=\s*)\d+/, `$1${androidCode}`, 'apps/mobile-android/app/build.gradle.kts');
  gradle = replaceExactly(gradle, /(versionName\s*=\s*")[^"]+(")/, `$1${version}$2`, 'apps/mobile-android/app/build.gradle.kts');
  output.set('apps/mobile-android/app/build.gradle.kts', gradle);

  for (const file of [
    'README.md',
    'docs/.vitepress/config.js',
    'index.html',
    'docs/GETTING_STARTED.md',
    'docs/OPERATIONS_MANUAL.md',
    'docs/TROUBLESHOOTING.md',
    'docs/wiki/Home.md',
    'docs/wiki/_Sidebar.md',
  ]) {
    output.set(file, updateProductText(sourceFiles.get(file), file, version));
  }
  output.set('directives/IDENTITY.md', updateIdentity(sourceFiles.get('directives/IDENTITY.md'), version));
  output.set('docs/wiki/_Footer.md', updateWikiFooter(sourceFiles.get('docs/wiki/_Footer.md'), version));

  for (const file of ['docs/openapi.yaml', 'docs/API_REFERENCE.md', 'docs/API_CONTRACT.md']) {
    output.set(file, updateApiText(sourceFiles.get(file), file, apiVersion));
  }

  return output;
}

function parseArgs(argv) {
  const options = { check: false, bump: null, androidCode: null, apiVersion: null };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === '--check') options.check = true;
    else if (arg === '--bump') options.bump = argv[++index];
    else if (arg === '--android-code') options.androidCode = Number(argv[++index]);
    else if (arg === '--api-version') options.apiVersion = argv[++index];
    else if (!arg.startsWith('-') && !options.bump) options.bump = arg;
    else throw new VersionError('VERSION:INVALID', `unknown argument: ${arg}`);
  }
  if (options.check && (options.bump || options.androidCode || options.apiVersion)) {
    throw new VersionError('VERSION:INVALID', '--check cannot be combined with mutation options');
  }
  if (!options.check && !options.bump && !options.apiVersion && options.androidCode === null) {
    throw new VersionError('VERSION:INVALID', 'use --check or provide --bump X.Y.Z');
  }
  if (options.bump && !SEMVER.test(options.bump)) {
    throw new VersionError('VERSION:INVALID', `invalid product SemVer: ${options.bump}`);
  }
  if (options.apiVersion && !SEMVER.test(options.apiVersion)) {
    throw new VersionError('VERSION:INVALID', `invalid API document SemVer: ${options.apiVersion}`);
  }
  if (options.androidCode !== null && (!Number.isSafeInteger(options.androidCode) || options.androidCode < 1)) {
    throw new VersionError('VERSION:INVALID', '--android-code must be a positive integer');
  }
  return options;
}

function desiredAuthority(current, options, today = new Date().toISOString().slice(0, 10)) {
  validateAuthority(current);
  const next = { ...current };
  const versionChanged = Boolean(options.bump && options.bump !== current.version);
  if (options.bump) next.version = options.bump;
  if (options.apiVersion) next.api_document_version = options.apiVersion;

  if (options.androidCode !== null) {
    if (options.androidCode < current.android_version_code || (versionChanged && options.androidCode === current.android_version_code)) {
      throw new VersionError('VERSION:INVALID', '--android-code must preserve monotonicity and increase for a new product version');
    }
    next.android_version_code = options.androidCode;
  } else if (versionChanged) {
    next.android_version_code = current.android_version_code + 1;
  }
  if (versionChanged) next.version_updated_at = today;
  return next;
}

function writeAtomically(rootDir, sourceFiles, outputFiles) {
  const changed = [...outputFiles].filter(([file, content]) => sourceFiles.get(file) !== content);
  const written = [];
  try {
    for (const [file, content] of changed) {
      fs.writeFileSync(path.join(rootDir, file), content, 'utf8');
      written.push(file);
    }
  } catch (error) {
    for (const file of written.reverse()) {
      fs.writeFileSync(path.join(rootDir, file), sourceFiles.get(file), 'utf8');
    }
    throw new VersionError('VERSION:WRITE', `write failed; restored ${written.length} file(s): ${error.message}`);
  }
  return changed.map(([file]) => file);
}

function run({ rootDir = path.resolve(__dirname, '..'), options, today } = {}) {
  const parsed = options || parseArgs(process.argv.slice(2));
  const sourceFiles = loadAll(rootDir);
  const current = parseJson(sourceFiles.get('version.json'), 'version.json');
  validateAuthority(current);

  if (parsed.check) {
    const expected = buildOutputs(rootDir, current, sourceFiles);
    const drift = [...expected].filter(([file, content]) => sourceFiles.get(file) !== content).map(([file]) => file);
    if (drift.length) {
      throw new VersionError('VERSION:DRIFT', `version drift detected in: ${drift.join(', ')}`);
    }
    return { mode: 'check', authority: current, changed: [] };
  }

  const desired = desiredAuthority(current, parsed, today);
  const output = buildOutputs(rootDir, desired, sourceFiles);
  const changed = writeAtomically(rootDir, sourceFiles, output);
  return { mode: 'bump', authority: desired, changed };
}

function main() {
  try {
    const result = run();
    if (result.mode === 'check') {
      console.log(`[VERSION:OK] product=${result.authority.version} api=${result.authority.api_document_version} android=${result.authority.android_version_code}; ${TARGET_PATHS.length} targets aligned`);
    } else {
      console.log(`[VERSION:UPDATED] product=${result.authority.version} api=${result.authority.api_document_version} android=${result.authority.android_version_code}; ${result.changed.length} file(s) changed`);
    }
  } catch (error) {
    const code = error.code || 'VERSION:INVALID';
    console.error(`[${code}] ${error.message}`);
    process.exitCode = 1;
  }
}

if (require.main === module) main();

module.exports = {
  SEMVER,
  TARGET_PATHS,
  VersionError,
  buildOutputs,
  desiredAuthority,
  parseArgs,
  run,
};
