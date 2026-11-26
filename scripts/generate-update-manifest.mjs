#!/usr/bin/env node

/**
 * @docs ARCHITECTURE:DesktopShell
 * @docs RELEASE_PROCESS
 *
 * Generates the Tauri v2 `latest.json` update manifest for GitHub Releases.
 * Usage: node scripts/generate-update-manifest.mjs [--version X.Y.Z] [--notes "Release Notes"] [--output dist/latest.json]
 */

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..');

// Read product release authority
const versionJsonPath = path.join(rootDir, 'version.json');
let version = '1.1.464';
if (fs.existsSync(versionJsonPath)) {
    const vData = JSON.parse(fs.readFileSync(versionJsonPath, 'utf-8'));
    version = vData.product_version || version;
}

// Parse CLI args
const args = process.argv.slice(2);
let customNotes = `Tadpole OS v${version} Sovereign Desktop Release`;
let outputPath = path.join(rootDir, 'dist', 'latest.json');

for (let i = 0; i < args.length; i++) {
    if (args[i] === '--version' && args[i + 1]) {
        version = args[++i];
    } else if (args[i] === '--notes' && args[i + 1]) {
        customNotes = args[++i];
    } else if (args[i] === '--output' && args[i + 1]) {
        outputPath = path.resolve(process.cwd(), args[++i]);
    }
}

const baseUrl = `https://github.com/DDS-Solutions/AI-Tadpole-OS/releases/download/v${version}`;

const manifest = {
    version,
    notes: customNotes,
    pub_date: new Date().toISOString(),
    platforms: {
        'windows-x86_64': {
            signature: '',
            url: `${baseUrl}/AI-Tadpole-OS_${version}_x64-setup.exe`
        },
        'darwin-aarch64': {
            signature: '',
            url: `${baseUrl}/AI-Tadpole-OS_${version}_aarch64.dmg`
        },
        'darwin-x86_64': {
            signature: '',
            url: `${baseUrl}/AI-Tadpole-OS_${version}_x64.dmg`
        },
        'linux-x86_64': {
            signature: '',
            url: `${baseUrl}/ai-tadpole-os_${version}_amd64.AppImage`
        }
    }
};

const outputDir = path.dirname(outputPath);
if (!fs.existsSync(outputDir)) {
    fs.mkdirSync(outputDir, { recursive: true });
}

fs.writeFileSync(outputPath, JSON.stringify(manifest, null, 2) + '\n', 'utf-8');
console.log(`[UPDATER:OK] Generated Tauri v2 update manifest at ${outputPath} for v${version}`);

