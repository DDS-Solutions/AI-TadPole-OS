/**
 * @docs ARCHITECTURE:Infrastructure
 *
 * ### AI Context Alignment
 * - **Subsystem**: System Core / mock_oversight
 * - **Primary Entrypoints**: `MOCK_PENDING`, `MOCK_LEDGER`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import type { ToolCall, OversightEntry, LedgerEntry } from '../types/oversight';
export type { ToolCall, OversightEntry, LedgerEntry };

export const MOCK_PENDING: OversightEntry[] = [
    {
        id: 'ov-1',
        tool_call: {
            id: 'tc-1',
            agent_id: '1', // Agent of Nine
            skill: 'Execute Command',
            description: 'Deploying security patch to production gateway.',
            params: { target: 'gateway-01', payload: 'v1.4.2-sec' },
            timestamp: new Date().toISOString()
        },
        decision: 'pending',
        created_at: new Date(Date.now() - 120000).toISOString()
    },
    {
        id: 'ov-2',
        tool_call: {
            id: 'tc-2',
            agent_id: '3', // Strategic Alpha
            skill: 'Modify File',
            description: 'Updating server firewall rules to block suspicious IP range.',
            params: { path: '/etc/iptables.conf', action: 'append', rules: 'DROP 10.0.0.1/32' },
            timestamp: new Date().toISOString()
        },
        decision: 'pending',
        created_at: new Date(Date.now() - 45000).toISOString()
    }
];

export const MOCK_LEDGER: LedgerEntry[] = [
    {
        id: 'le-1',
        tool_call: {
            id: 'tc-old-1',
            agent_id: '7',
            skill: 'Read Logs',
            description: 'Scanning system logs for anomalies.',
            params: { lines: 50, filter: 'error' },
            timestamp: new Date(Date.now() - 500000).toISOString()
        },
        decision: 'approved',
        auto_approved: false,
        approval_type: 'hitl',
        requires_oversight: true,
        result: {
            success: true,
            output: 'Scan complete. 0 critical errors found.',
            duration_ms: 450
        },
        timestamp: new Date(Date.now() - 480000).toISOString()
    },
    {
        id: 'le-2',
        tool_call: {
            id: 'tc-old-2',
            agent_id: '11',
            skill: 'Delete File',
            description: 'Attempting to remove temporary cache directory.',
            params: { path: '/tmp/old_cache' },
            timestamp: new Date(Date.now() - 300000).toISOString()
        },
        decision: 'rejected',
        auto_approved: false,
        approval_type: 'hitl',
        requires_oversight: true,
        timestamp: new Date(Date.now() - 290000).toISOString()
    },
    {
        id: 'le-3',
        tool_call: {
            id: 'tc-auto-1',
            agent_id: '2',
            skill: 'Telemetry Sync',
            description: 'Automated background node health telemetry reporting.',
            params: { node: 'swarm-us-east-1', metric: 'heartbeat' },
            timestamp: new Date(Date.now() - 180000).toISOString()
        },
        decision: 'approved',
        auto_approved: true,
        approval_type: 'auto',
        requires_oversight: false,
        result: {
            success: true,
            output: 'Heartbeat acknowledged by control plane.',
            duration_ms: 120
        },
        timestamp: new Date(Date.now() - 175000).toISOString()
    },
    {
        id: 'le-4',
        tool_call: {
            id: 'tc-auto-2',
            agent_id: '5',
            skill: 'Cache Invalidation',
            description: 'Auto-purging stale memory vectors for idle session.',
            params: { namespace: 'agent-vector-mem', threshold_ms: 3600000 },
            timestamp: new Date(Date.now() - 90000).toISOString()
        },
        decision: 'approved',
        auto_approved: true,
        approval_type: 'auto',
        requires_oversight: false,
        result: {
            success: true,
            output: '142 stale vector embeddings evicted.',
            duration_ms: 85
        },
        timestamp: new Date(Date.now() - 88000).toISOString()
    }
];
