/**
 * 🩺 Tadpole OS Health & Security Doctor
 * Self-contained, zero-dependency diagnostic utility for environment, engine, and safety verification.
 */

import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { execSync } from 'node:child_process';

const rootDir = process.cwd();
const homeDir = os.homedir();

function sanitizePath(inputPath) {
    if (!inputPath || typeof inputPath !== 'string') return '';
    return inputPath.replaceAll(homeDir, '<USER_HOME>');
}

function checkSignal0Liveness(pid) {
    if (!Number.isInteger(pid) || pid <= 0) return false;
    try {
        process.kill(pid, 0);
        return true;
    } catch {
        return false;
    }
}

const results = {
    passed: 0,
    warnings: 0,
    failed: 0,
};

function logResult(status, category, message) {
    const icons = {
        PASS: '✅',
        WARN: '⚠️ ',
        FAIL: '❌',
    };
    if (status === 'PASS') results.passed++;
    if (status === 'WARN') results.warnings++;
    if (status === 'FAIL') results.failed++;

    console.log(`${icons[status]} [${category}] ${message}`);
}

console.log('\n==================================================');
console.log('      🩺 Tadpole OS Sovereign Health Doctor       ');
console.log('==================================================\n');

// 1. Node.js Engine Check
const nodeVersion = process.version;
const majorVersion = parseInt(nodeVersion.replace(/^v/, '').split('.')[0], 10);
if (majorVersion >= 18) {
    logResult('PASS', 'Node.js', `Runtime version ${nodeVersion} meets requirement (>= 18)`);
} else {
    logResult('FAIL', 'Node.js', `Runtime version ${nodeVersion} is outdated. Require Node.js >= 18`);
}

// 2. Cargo & Rust Toolchain Check
try {
    const cargoVer = execSync('cargo --version', { encoding: 'utf8', stdio: ['ignore', 'pipe', 'ignore'] }).trim();
    const cargoTomlPath = path.join(rootDir, 'server-rs', 'Cargo.toml');
    if (fs.existsSync(cargoTomlPath)) {
        logResult('PASS', 'Rust Engine', `${cargoVer} detected; server-rs/Cargo.toml found`);
    } else {
        logResult('WARN', 'Rust Engine', `${cargoVer} detected, but server-rs/Cargo.toml missing at ${sanitizePath(cargoTomlPath)}`);
    }
} catch {
    logResult('WARN', 'Rust Engine', 'cargo executable not found in system PATH. Rust server engine requires Rust toolchain.');
}

// 3. SQLite Database Check
const dbPath = path.join(rootDir, 'tadpole.db');
if (fs.existsSync(dbPath)) {
    try {
        fs.accessSync(dbPath, fs.constants.R_OK | fs.constants.W_OK);
        logResult('PASS', 'Database', `tadpole.db accessible at ${sanitizePath(dbPath)}`);
    } catch {
        logResult('FAIL', 'Database', `tadpole.db exists but lacks read/write permissions at ${sanitizePath(dbPath)}`);
    }
} else {
    logResult('PASS', 'Database', `tadpole.db not yet created (will initialize automatically on engine boot)`);
}

// 4. Environment Schema Verification
const envPath = path.join(rootDir, '.env');
const envSchemaPath = path.join(rootDir, '.env.schema');
if (fs.existsSync(envPath)) {
    logResult('PASS', 'Environment', `.env configuration file detected`);
    if (fs.existsSync(envSchemaPath)) {
        const envContent = fs.readFileSync(envPath, 'utf8');
        const schemaContent = fs.readFileSync(envSchemaPath, 'utf8');
        const schemaKeys = schemaContent.split('\n')
            .map(line => line.trim())
            .filter(line => line && !line.startsWith('#'))
            .map(line => line.split('=')[0]);

        const missingKeys = schemaKeys.filter(key => key && !envContent.includes(key));
        if (missingKeys.length === 0) {
            logResult('PASS', 'Environment', `.env matches all schema keys in .env.schema`);
        } else {
            logResult('WARN', 'Environment', `.env missing ${missingKeys.length} keys from schema: ${missingKeys.slice(0, 3).join(', ')}${missingKeys.length > 3 ? '...' : ''}`);
        }
    }
} else {
    logResult('WARN', 'Environment', `.env file not found. Copy .env.example or .env.schema to create .env`);
}

// 5. Python SOP Runner Check
try {
    const pythonCmd = process.platform === 'win32' ? 'python' : 'python3';
    const pyVer = execSync(`${pythonCmd} --version`, { encoding: 'utf8', stdio: ['ignore', 'pipe', 'ignore'] }).trim();
    const executionDir = path.join(rootDir, 'execution');
    if (fs.existsSync(executionDir)) {
        logResult('PASS', 'Python SOPs', `${pyVer} detected; execution/ SOP directory present`);
    } else {
        logResult('WARN', 'Python SOPs', `${pyVer} detected, but execution/ directory is missing`);
    }
} catch {
    logResult('WARN', 'Python SOPs', 'Python runtime not detected in PATH. Layer-3 SOP scripts in execution/ require Python 3.');
}

// 6. OS Signal-0 Process Liveness Check
const selfAlive = checkSignal0Liveness(process.pid);
if (selfAlive) {
    logResult('PASS', 'Process Safety', `Signal-0 OS PID check verified (current process PID ${process.pid})`);
} else {
    logResult('FAIL', 'Process Safety', `Signal-0 OS PID check failed for PID ${process.pid}`);
}

console.log('\n--------------------------------------------------');
console.log(` Diagnostic Summary: ${results.passed} Passed | ${results.warnings} Warnings | ${results.failed} Failed`);
console.log('--------------------------------------------------\n');

if (results.failed > 0) {
    process.exit(1);
} else {
    process.exit(0);
}
