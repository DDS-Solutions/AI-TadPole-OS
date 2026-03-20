import { describe, it, expect, vi, beforeEach } from 'vitest';
import { Gateway } from '../../server/gateway.js';
import { OversightGate } from '../../server/oversight.js';
import { randomUUID } from 'node:crypto';
import type { ToolCall } from '../../server/types.js';

// Setup Mocks for dependencies
vi.mock('express', () => {
    const mockApp = {
        use: vi.fn(),
        get: vi.fn(),
        post: vi.fn(),
        listen: vi.fn(),
    };
    const expressMock = vi.fn(() => mockApp);
    (expressMock as any).json = vi.fn(() => (req: any, res: any, next: any) => next());
    (expressMock as any).urlencoded = vi.fn(() => (req: any, res: any, next: any) => next());

    return {
        default: expressMock,
        json: (expressMock as any).json,
        urlencoded: (expressMock as any).urlencoded,
    };
});

vi.mock('ws', () => {
    return {
        WebSocketServer: vi.fn().mockImplementation(function () {
            return {
                on: vi.fn(),
                clients: new Set(),
                close: vi.fn()
            };
        })
    };
});

vi.mock('http', () => {
    const mockServer = {
        listen: vi.fn(),
        on: vi.fn(),
        close: vi.fn()
    };
    return {
        createServer: vi.fn().mockReturnValue(mockServer),
        default: {
            createServer: vi.fn().mockReturnValue(mockServer)
        }
    };
});

// We do NOT mock OversightGate here because we want to test its real logic
// We DO mock AgentRunner to avoid starting the actual AI engine
vi.mock('../../server/runner', () => ({
    AgentRunner: class {
        constructor() { }
        run = vi.fn();
    }
}));

describe('E2E: Oversight Safety Loop', () => {
    let gateway: Gateway;
    let oversight: OversightGate;

    beforeEach(() => {
        vi.clearAllMocks();
        // Instantiate Gateway which initializes the real OversightGate
        gateway = new Gateway();
        // Access the private oversight instance for testing
        oversight = (gateway as any).oversight;

        // Disable auto-approve to force manual intervention
        oversight.setGovernance({ autoApproveSafeSkills: false });
    });

    it('should pause execution for unsafe skills and resume after approval', async () => {
        // 1. Simulate an Agent trying to use a "dangerous" tool
        const toolCall: ToolCall = {
            id: randomUUID(),
            agentId: 'Agent-007',
            department: 'Operations',
            skill: 'system_reset', // Unsafe skill
            description: 'Factory Reset',
            params: { confirm: true },
            timestamp: new Date().toISOString()
        };

        // 2. Submit to Oversight (Async - will hang until approved)
        const approvalPromise = oversight.submit(toolCall);

        // 3. Verify it's pending
        const pending = oversight.getPending();
        expect(pending).toHaveLength(1);
        expect(pending[0].toolCall.skill).toBe('system_reset');
        expect(pending[0].status).toBe('pending');

        // 4. Simulate User Logic: Find the entry and Approve it
        const entryId = pending[0].id;
        console.debug(`[Test] User approving task ${entryId}`);

        // In a real app, this would be a POST /oversight/approve call
        // Here we call the method directly to simulate the API handler's action
        const resultEntry = oversight.decide(entryId, 'approved');

        expect(resultEntry).toBeDefined();
        expect(resultEntry?.status).toBe('approved');

        // 5. Verify the original promise resolves to TRUE
        const isApproved = await approvalPromise;
        expect(isApproved).toBe(true);

        // 6. Verify Queue is empty
        expect(oversight.getPending()).toHaveLength(0);
    });

    it('should reject execution when denied by user', async () => {
        const toolCall: ToolCall = {
            id: randomUUID(),
            agentId: 'Agent-007',
            department: 'Operations',
            skill: 'delete_database',
            description: 'Drop all tables',
            params: {},
            timestamp: new Date().toISOString()
        };

        const approvalPromise = oversight.submit(toolCall);

        const pending = oversight.getPending();
        const entryId = pending[0].id;

        // User REJECTS
        oversight.decide(entryId, 'rejected');

        // Promise should resolve to FALSE
        const isApproved = await approvalPromise;
        expect(isApproved).toBe(false);
    });
});
