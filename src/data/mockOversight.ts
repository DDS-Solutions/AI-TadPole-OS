

export interface ToolCall {
    id: string;
    agentId: string;
    clusterId?: string;
    skill: string;
    description: string;
    params: Record<string, unknown>;
    timestamp: string;
}

export interface OversightEntry extends Partial<Omit<ToolCall, 'id'>> {
    id: string;
    toolCall?: ToolCall;
    decision: 'pending' | 'approved' | 'rejected';
    decidedBy?: string;
    decidedAt?: string;
    createdAt: string;
}

export interface LedgerEntry extends Partial<Omit<ToolCall, 'id'>> {
    id: string;
    toolCall?: ToolCall;
    decision: 'approved' | 'rejected';
    result?: {
        success: boolean;
        output: string;
        error?: string;
        durationMs: number;
    };
    timestamp: string;
}

export const MOCK_PENDING: OversightEntry[] = [
    {
        id: 'ov-1',
        toolCall: {
            id: 'tc-1',
            agentId: '1', // Agent of Nine
            skill: 'Execute Command',
            description: 'Deploying security patch to production gateway.',
            params: { target: 'gateway-01', payload: 'v1.4.2-sec' },
            timestamp: new Date().toISOString()
        },
        decision: 'pending',
        createdAt: new Date(Date.now() - 120000).toISOString()
    },
    {
        id: 'ov-2',
        toolCall: {
            id: 'tc-2',
            agentId: '3', // Strategic Alpha
            skill: 'Modify File',
            description: 'Updating server firewall rules to block suspicious IP range.',
            params: { path: '/etc/iptables.conf', action: 'append', rules: 'DROP 10.0.0.1/32' },
            timestamp: new Date().toISOString()
        },
        decision: 'pending',
        createdAt: new Date(Date.now() - 45000).toISOString()
    }
];

export const MOCK_LEDGER: LedgerEntry[] = [
    {
        id: 'le-1',
        toolCall: {
            id: 'tc-old-1',
            agentId: '7',
            skill: 'Read Logs',
            description: 'Scanning system logs for anomalies.',
            params: { lines: 50, filter: 'error' },
            timestamp: new Date(Date.now() - 500000).toISOString()
        },
        decision: 'approved',
        result: {
            success: true,
            output: 'Scan complete. 0 critical errors found.',
            durationMs: 450
        },
        timestamp: new Date(Date.now() - 480000).toISOString()
    },
    {
        id: 'le-2',
        toolCall: {
            id: 'tc-old-2',
            agentId: '11',
            skill: 'Delete File',
            description: 'Attempting to remove temporary cache directory.',
            params: { path: '/tmp/old_cache' },
            timestamp: new Date(Date.now() - 300000).toISOString()
        },
        decision: 'rejected',
        timestamp: new Date(Date.now() - 290000).toISOString()
    }
];
