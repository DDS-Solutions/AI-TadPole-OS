/**
 * @docs ARCHITECTURE:Infrastructure:Execution
 *
 * ### AI Assist Note
 * - **Subsystem**: Developer Tooling / Environment Diagnostic Tests
 * - Run the diagnostic with isolated process and filesystem fixtures.
 *
 * ### 🔍 Debugging & Observability
 * - Witness target: scripts/doctor.mjs; no live environment or toolchain probes.
 */
'use strict';

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');
const vm = require('node:vm');
const packageJson = require('../package.json');
const lock = require('../package-lock.json');

// Supply built-in imports as fixtures so the CLI never touches the live system.
const source = fs.readFileSync(path.join(__dirname, 'doctor.mjs'), 'utf8')
    .replace(/^import .+;\r?$/gm, '');

function diagnose(version) {
    const messages = [];
    let exitCode;
    vm.runInNewContext(source, {
        fs: { existsSync: () => false },
        path,
        os: { homedir: () => '/fixture/home' },
        execSync: () => { throw new Error('Toolchain absent in fixture'); },
        process: {
            version, pid: 123, platform: 'linux', cwd: () => '/fixture',
            kill: (pid, signal) => { assert.equal(pid, 123); assert.equal(signal, 0); },
            exit: (code) => { exitCode = code; },
        },
        console: { log: (message) => messages.push(message) },
    });
    return { exitCode, messages };
}

test('doctor enforces supported Node releases at the build and test toolchain boundaries', () => {
    for (const version of ['v18.20.8', 'v20.19.0', 'v22.12.0', 'v22.22.1', 'v23.11.0', 'v24.14.9', 'v25.9.0', 'v26.0.0-rc.1']) {
        const result = diagnose(version);
        assert.equal(result.exitCode, 1, version);
        assert.ok(result.messages.some((message) => message.includes(`[Node.js] Runtime version ${version} is unsupported`)));
    }
    for (const version of ['v22.22.2', 'v22.23.0', 'v24.15.0', 'v24.16.0', 'v26.0.0', 'v27.0.0']) {
        const result = diagnose(version);
        assert.equal(result.exitCode, 0, version);
        assert.ok(result.messages.some((message) => message.includes(`[Node.js] Runtime version ${version} meets requirement`)));
    }
});

test('package metadata and doctor agree with the locked jsdom runtime floor', () => {
    const requirement = lock.packages['node_modules/jsdom'].engines.node;
    assert.equal(packageJson.engines.node, requirement);
    assert.equal(lock.packages[''].engines.node, requirement);
    assert.ok(diagnose('v26.0.0').messages.some((message) => message.includes(`(${requirement})`)));
});
