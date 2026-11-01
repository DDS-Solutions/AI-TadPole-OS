#!/usr/bin/env node
/**
 * @docs ARCHITECTURE:Infrastructure:Build
 *
 * ### AI Assist Note
 * Cross-platform Linux Light artifact builder.
 * Invokes Docker to build .deb and .AppImage bundles without requiring Windows PowerShell.
 * Works uniformly on Linux, macOS, and Windows.
 *
 * ### 🔍 Debugging & Observability
 * - Telemetry Link: Search `[docker-light]` in build output.
 */

import { execSync, spawnSync } from 'child_process';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..');

const imageName = 'tadpole-os-linux-light-builder';
const containerName = 'tadpole-os-light-temp';
const outputDist = path.join(rootDir, 'dist', 'linux-light');

const isUpload = process.argv.includes('--upload') || process.argv.includes('-Upload');

function run(cmd, args, opts = {}) {
    const result = spawnSync(cmd, args, {
        cwd: rootDir,
        stdio: 'inherit',
        shell: process.platform === 'win32',
        ...opts,
    });
    if (result.error) {
        console.error(`[docker-light] Failed to execute ${cmd}: ${result.error.message}`);
        process.exit(1);
    }
    if (result.status !== 0) {
        process.exit(result.status ?? 1);
    }
}

console.log('[docker-light] Checking if Docker is available...');
try {
    execSync('docker version', { stdio: 'ignore' });
} catch {
    console.error('❌ ERROR: Docker is not running or not installed. Please start Docker first.');
    process.exit(1);
}

console.log('[docker-light] Building Lightweight Linux artifacts inside container...');
run('docker', ['build', '-t', imageName, '-f', 'Dockerfile.linux', '.']);

console.log(`[docker-light] Cleaning up ${outputDist}...`);
if (fs.existsSync(outputDist)) {
    fs.rmSync(outputDist, { recursive: true, force: true });
}
fs.mkdirSync(outputDist, { recursive: true });

console.log('[docker-light] Extracting bundles from container...');
// Remove any existing container with the same name before creating
try {
    execSync(`docker rm -f ${containerName}`, { stdio: 'ignore' });
} catch {
    // Ignore if container does not exist
}

run('docker', ['create', '--name', containerName, imageName]);

// Copy bundles out of container
try {
    run('docker', ['cp', `${containerName}:/app/src-tauri/target/release/bundle/appimage/.`, outputDist]);
} catch (e) {
    console.warn('[docker-light] Note: AppImage bundle extraction notice:', e.message);
}

try {
    run('docker', ['cp', `${containerName}:/app/src-tauri/target/release/bundle/deb/.`, outputDist]);
} catch (e) {
    console.warn('[docker-light] Note: Debian bundle extraction notice:', e.message);
}

// Clean up container
try {
    execSync(`docker rm -f ${containerName}`, { stdio: 'ignore' });
} catch {
    // Ignore cleanup errors
}

console.log(`\n✅ [DONE] Linux Light build complete! Files available in: dist/linux-light\n`);

if (isUpload) {
    if (process.platform === 'win32') {
        console.log('[docker-light] Delegating upload step to PowerShell on Windows...');
        run('powershell', ['-ExecutionPolicy', 'Bypass', '-File', 'scripts/build-linux-light.ps1', '-Upload']);
    } else {
        console.log('[docker-light] For non-Windows platforms, set GITHUB_TOKEN and use gh release upload or the CI pipeline.');
    }
}
