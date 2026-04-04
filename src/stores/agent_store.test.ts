/**
 * @file agentStore.test.ts
 * @description Suite for the central Agent management state machine.
 * @module Stores/AgentStore
 * @testedBehavior
 * - Async Orchestration: Fetching agent rosters and handling error events.
 * - Reactive State: Optimistic updates with automatic rollback on persistence failure.
 * - Telemetry: Real-time socket integration and workspace-aware ID resolution.
 * @aiContext
 * - Mocks localStorage to prevent persistence side effects.
 * - Simulates socket 'agent:update' payloads to validate reactive UI updates.
 */
import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';

// Block localStorage BEFORE imports
vi.hoisted(() => {
    const mockLocalStorage = {
        getItem: vi.fn(),
        setItem: vi.fn(),
        clear: vi.fn(),
        removeItem: vi.fn(),
        length: 0,
        key: vi.fn(),
    };
    vi.stubGlobal('localStorage', mockLocalStorage);
});

import { use_agent_store } from './agent_store';
import * as AgentService from '../services/agent_service';
import { event_bus } from '../services/event_bus';
import { tadpole_os_socket } from '../services/socket';
import { use_workspace_store } from './workspace_store';
import type { Agent } from '../types';

// Mock dependencies
vi.mock('../services/agent_service', async (importOriginal) => {
    const actual = await importOriginal<typeof import('../services/agent_service')>();
    return {
        ...actual,
        __esModule: true,
        loadAgents: vi.fn(),
        persistAgentUpdate: vi.fn(),
        normalizeAgent: vi.fn((raw) => ({ ...raw, normalized: true })),
    };
});

vi.mock('../services/event_bus', () => ({
    event_bus: {
        emit: vi.fn(),
    }
}));

vi.mock('../services/socket', () => ({
    tadpole_os_socket: {
        subscribeAgentUpdates: vi.fn(),
    }
}));

vi.mock('./workspace_store', () => ({
    use_workspace_store: {
        getState: vi.fn(),
    }
}));

describe('agent_store', () => {
    const mockAgent1: Agent = {
        id: '1',
        name: 'Test Agent 1',
        role: 'Developer',
        model: 'model-a',
        modelConfig: { provider: 'test', modelId: 'test' },
        department: 'Engineering',
        workspacePath: '/test',
        status: 'idle',
        tokensUsed: 0,
        category: 'user'
    };

    const mockAgent2: Agent = {
        ...mockAgent1,
        id: '2',
        name: 'Test Agent 2'
    };

    beforeEach(() => {
        vi.clearAllMocks();
        
        // Reset state
        use_agent_store.setState({
            agents: [],
            isLoading: false,
            error: null
        });

        (AgentService.loadAgents as Mock).mockResolvedValue([mockAgent1, mockAgent2]);
        (AgentService.persistAgentUpdate as Mock).mockResolvedValue(true);
    });

    describe('fetchAgents', () => {
        it('fetches agents successfully and updates state', async () => {
            const promise = use_agent_store.getState().fetchAgents();
            expect(use_agent_store.getState().isLoading).toBe(true);
            
            await promise;
            
            expect(use_agent_store.getState().isLoading).toBe(false);
            expect(use_agent_store.getState().error).toBeNull();
            expect(use_agent_store.getState().agents).toEqual([mockAgent1, mockAgent2]);
        });

        it('handles fetch errors correctly and emits event', async () => {
            (AgentService.loadAgents as Mock).mockRejectedValue(new Error('Network error'));
            
            await use_agent_store.getState().fetchAgents();
            
            expect(use_agent_store.getState().isLoading).toBe(false);
            expect(use_agent_store.getState().error).toBe('Network error');
            expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({
                severity: 'error',
                text: expect.stringContaining('Network error')
            }));
        });
    });

    describe('update_agent', () => {
        beforeEach(() => {
            use_agent_store.setState({ agents: [mockAgent1, mockAgent2] });
        });

        it('performs an optimistic update and persists successfully', async () => {
            await use_agent_store.getState().update_agent('1', { name: 'Updated Name', status: 'working' });
            
            const agent = use_agent_store.getState().agents.find(a => a.id === '1');
            expect(agent?.name).toBe('Updated Name');
            expect(agent?.status).toBe('working');
            
            expect(AgentService.persistAgentUpdate).toHaveBeenCalledWith('1', { name: 'Updated Name', status: 'working' });
            expect(AgentService.loadAgents).not.toHaveBeenCalled(); // No revert needed
        });

        it('reverts state and emits event on persistence failure', async () => {
            (AgentService.persistAgentUpdate as Mock).mockRejectedValue(new Error('Sync failed'));
            
            // Revert will trigger loadAgents, let's mock it to return original agent
            (AgentService.loadAgents as Mock).mockResolvedValue([mockAgent1, mockAgent2]);
            
            await use_agent_store.getState().update_agent('1', { name: 'Updated Name' });
            
            // It should NOT revert (to prevent flickers)
            const agent = use_agent_store.getState().agents.find(a => a.id === '1');
            expect(agent?.name).toBe('Updated Name'); // kept optimistic
            expect(use_agent_store.getState().error).toBeNull(); // handled silently with warning
            
            expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({
                severity: 'warning',
                text: expect.stringContaining('Sync failed')
            }));
        });
    });

    describe('addAgent', () => {
        beforeEach(() => {
            use_agent_store.setState({ agents: [mockAgent1] });
        });

        it('optimistically adds an agent and persists', async () => {
            await use_agent_store.getState().addAgent(mockAgent2);
            
            expect(use_agent_store.getState().agents).toHaveLength(2);
            expect(use_agent_store.getState().agents[1]).toEqual(mockAgent2);
            
            expect(AgentService.persistAgentUpdate).toHaveBeenCalledWith('2', mockAgent2);
        });

        it('reverts and emits event on persistence failure', async () => {
            (AgentService.persistAgentUpdate as Mock).mockRejectedValue(new Error('Create failed'));
            (AgentService.loadAgents as Mock).mockResolvedValue([mockAgent1]); // Revert state
            
            await use_agent_store.getState().addAgent(mockAgent2);
            
            expect(use_agent_store.getState().agents).toHaveLength(1); // Reverted
            expect(use_agent_store.getState().error).toBe('Create failed');
            
            expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({
                severity: 'error',
                text: expect.stringContaining('Create failed')
            }));
        });
    });

    describe('getAgent', () => {
        it('retrieves an agent by id', () => {
            use_agent_store.setState({ agents: [mockAgent1, mockAgent2] });
            
            expect(use_agent_store.getState().getAgent('2')).toEqual(mockAgent2);
            expect(use_agent_store.getState().getAgent('non-existent')).toBeUndefined();
        });
    });

    describe('initTelemetry', () => {
        let updateCallback: (event: any) => void;

        beforeEach(() => {
            vi.mocked(tadpole_os_socket.subscribeAgentUpdates).mockImplementation((cb: any) => {
                updateCallback = cb;
                return vi.fn(); // mock unsubscribe
            });
            
            (use_workspace_store.getState as Mock).mockReturnValue({
                clusters: [
                    { path: '/org/frontend', collaborators: ['1'] }
                ]
            });

            use_agent_store.setState({ agents: [mockAgent1] });
        });

        it('subscribes to updates and applies them with workspace resolution', () => {
            const unsubscribe = use_agent_store.getState().initTelemetry();
            expect(tadpole_os_socket.subscribeAgentUpdates).toHaveBeenCalled();
            expect(typeof unsubscribe).toBe('function');

            // Simulate incoming socket event
            updateCallback({
                type: 'agent:update',
                agent_id: '1',
                data: { status: 'working', telemetry: { cpu: 50 }, category: 'user' }
            });

            const agent = use_agent_store.getState().agents.find(a => a.id === '1') as any;
            
            // Check that it merged the initial mockAgent1 with the update data
            expect(agent).toMatchObject({
                id: '1',
                status: 'working',
                telemetry: { cpu: 50 },
                normalized: true // our mock normalizeAgent adds this
            });

            // normalizeAgent should have been called with the correct workspace path resolved from workspaceStore
            expect(AgentService.normalizeAgent).toHaveBeenCalledWith(
                expect.objectContaining({ id: '1', status: 'working' }),
                '/org/frontend',
                expect.objectContaining({ id: '1' })
            );
        });
        
        it('resolves silo path if agent is not in any cluster', () => {
            use_agent_store.setState({ agents: [{ ...mockAgent1, id: '99' }] });
            use_agent_store.getState().initTelemetry();
            
            updateCallback({
                type: 'agent:update',
                agent_id: '99', // Not in cluster mock
                data: { status: 'working', category: 'user' }
            });
            
            expect(AgentService.normalizeAgent).toHaveBeenCalledWith(
                expect.objectContaining({ id: '99', status: 'working' }),
                '/workspaces/agent-silo-99',
                expect.objectContaining({ id: '99' })
            );
        });
    });
});


