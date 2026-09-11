#!/usr/bin/env node
/**
 * @docs ARCHITECTURE:Infrastructure:Execution
 *
 * ### AI Context Alignment
 * - **Subsystem**: Release Governance / Artifact Provenance
 * - **Primary Entrypoints**: `generateManifest`, `main`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - The release tag must equal `v<product version>`.
 * - Private and sanitized-public commit identities are recorded separately.
 * - Every mirrored artifact is hashed before the public release is published.
 *
 * ### Debugging & Observability
 * - **Local Errors**: `[PROVENANCE:INVALID]`
 * - **Telemetry Targets**: output manifest path and artifact count
 */

'use strict';

const crypto = require('node:crypto');
const fs = require('node:fs');
const path = require('node:path');

function sha256(file) {
  return crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex');
}

function parseArgs(argv) {
  const result = {};
  for (let index = 0; index < argv.length; index += 2) {
    const key = argv[index];
    const value = argv[index + 1];
    if (!key?.startsWith('--') || value === undefined) throw new Error(`invalid argument near ${key || '<end>'}`);
    result[key.slice(2)] = value;
  }
  for (const required of ['version', 'private-sha', 'public-sha', 'run-id', 'tag', 'sanitizer', 'artifacts', 'output']) {
    if (!result[required]) throw new Error(`missing --${required}`);
  }
  if (result.tag !== `v${result.version}`) throw new Error(`tag ${result.tag} does not match version ${result.version}`);
  for (const field of ['private-sha', 'public-sha']) {
    if (!/^[0-9a-f]{40}$/i.test(result[field])) throw new Error(`${field} must be a full Git commit SHA`);
  }
  return result;
}

function generateManifest(options) {
  const artifactDir = path.resolve(options.artifacts);
  const output = path.resolve(options.output);
  const sanitizer = path.resolve(options.sanitizer);
  if (!fs.statSync(artifactDir).isDirectory()) throw new Error('artifacts must be a directory');
  if (!fs.statSync(sanitizer).isFile()) throw new Error('sanitizer must be a file');

  const artifacts = fs.readdirSync(artifactDir, { withFileTypes: true })
    .filter((entry) => entry.isFile() && path.resolve(artifactDir, entry.name) !== output)
    .map((entry) => {
      const file = path.join(artifactDir, entry.name);
      return { name: entry.name, size_bytes: fs.statSync(file).size, sha256: sha256(file) };
    })
    .sort((left, right) => left.name.localeCompare(right.name));

  const manifest = {
    schema_version: 1,
    product: 'Tadpole OS',
    version: options.version,
    tag: options.tag,
    private_source_sha: options['private-sha'].toLowerCase(),
    sanitized_public_sha: options['public-sha'].toLowerCase(),
    github_run_id: String(options['run-id']),
    generated_at: new Date().toISOString(),
    sanitizer: {
      path: options.sanitizer.replace(/\\/g, '/'),
      sha256: sha256(sanitizer),
    },
    artifacts,
  };
  fs.mkdirSync(path.dirname(output), { recursive: true });
  fs.writeFileSync(output, `${JSON.stringify(manifest, null, 2)}\n`, 'utf8');
  return manifest;
}

function main() {
  try {
    const options = parseArgs(process.argv.slice(2));
    const manifest = generateManifest(options);
    console.log(`[PROVENANCE:OK] ${options.output}; ${manifest.artifacts.length} artifact(s) hashed`);
  } catch (error) {
    console.error(`[PROVENANCE:INVALID] ${error.message}`);
    process.exitCode = 1;
  }
}

if (require.main === module) main();

module.exports = { generateManifest, parseArgs };
