/**
 * @docs ARCHITECTURE:Quality:Verification
 *
 * ### AI Assist Note
 * Generates symbol graph context before running audit scripts so coding agents
 * can inspect callers, callees, and blast radius when an audit reports a gap.
 *
 * ### Debugging & Observability
 * - Failure Path: graph_query compile failure, stale Cargo dependencies, or audit script failure.
 * - Telemetry Link: Search `[audit_with_graph]` in command output.
 */

const { spawnSync } = require('child_process');
const path = require('path');

const root = path.resolve(__dirname, '..');
const defaultAudits = ['audit:observability:raw'];
const audits = process.argv.slice(2);
const selectedAudits = (audits.length > 0 ? audits : defaultAudits).map(audit =>
    audit === 'audit:observability' ? 'audit:observability:raw' : audit
);

function run(command, args, options = {}) {
    const executable = process.platform === 'win32' && command === 'npm' ? 'cmd.exe' : command;
    const executableArgs = process.platform === 'win32' && command === 'npm'
        ? ['/d', '/s', '/c', ['npm', ...args].join(' ')]
        : args;
    const result = spawnSync(executable, executableArgs, {
        cwd: root,
        stdio: 'inherit',
        ...options,
    });

    if (result.error) {
        console.error(`[audit_with_graph] Failed to run ${command}: ${result.error.message}`);
        process.exit(1);
    }

    if (result.status !== 0) {
        process.exit(result.status ?? 1);
    }
}

console.log('[audit_with_graph] Generating reports/intelligence/audit_context.json');
run('npm', ['run', 'graph:audit', '--silent']);

for (const audit of selectedAudits) {
    if (audit === 'audit:graph') {
        console.error('[audit_with_graph] Refusing recursive audit:graph invocation.');
        process.exit(1);
    }
    console.log(`[audit_with_graph] Running ${audit}`);
    run('npm', ['run', audit, '--silent']);
}

console.log('[audit_with_graph] Graph-aware audit complete.');

// Metadata: [audit_with_graph]
