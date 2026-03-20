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

import { useAgentStore } from './agentStore';
import * as AgentService from '../services/agentService';
import { EventBus } from '../services/eventBus';
import { TadpoleOSSocket } from '../services/socket';
import { useWorkspaceStore } from './workspaceStore';
import type { Agent } from '../types';

// Mock dependencies
vi.mock('../services/agentService', async (importOriginal) => {
    const actual = await importOriginal<typeof import('../services/agentService')>();
    return {
        ...actual,
        __esModule: true,
        loadAgents: vi.fn(),
        persistAgentUpdate: vi.fn(),
        normalizeAgent: vi.fn((raw) => ({ ...raw, normalized: true })),
    };
});

vi.mock('../services/eventBus', () => ({
    EventBus: {
        emit: vi.fn(),
    }
}));

vi.mock('../services/socket', () => ({
    TadpoleOSSocket: {
        subscribeAgentUpdates: vi.fn(),
    }
}));

vi.mock('./workspaceStore', () => ({
    useWorkspaceStore: {
        getState: vi.fn(),
    }
}));

describe('agentStore', () => {
    const mockAgent1: Agent = {
        id: 'agent-1',
        name: 'Test Agent 1',
        role: 'Developer',
        model: 'model-a',
        modelConfig: { provider: 'test', modelId: 'test' },
        department: 'Engineering',
        workspacePath: '/test',
        status: 'idle',
        tokensUsed: 0
    };

    const mockAgent2: Agent = {
        ...mockAgent1,
        id: 'agent-2',
        name: 'Test Agent 2'
    };

    beforeEach(() => {
        vi.clearAllMocks();
        
        // Reset state
        useAgentStore.setState({
            agents: [],
            isLoading: false,
            error: null
        });

        (AgentService.loadAgents as Mock).mockResolvedValue([mockAgent1, mockAgent2]);
        (AgentService.persistAgentUpdate as Mock).mockResolvedValue(true);
    });

    describe('fetchAgents', () => {
        it('fetches agents successfully and updates state', async () => {
            const promise = useAgentStore.getState().fetchAgents();
            expect(useAgentStore.getState().isLoading).toBe(true);
            
            await promise;
            
            expect(useAgentStore.getState().isLoading).toBe(false);
            expect(useAgentStore.getState().error).toBeNull();
            expect(useAgentStore.getState().agents).toEqual([mockAgent1, mockAgent2]);
        });

        it('handles fetch errors correctly and emits event', async () => {
            (AgentService.loadAgents as Mock).mockRejectedValue(new Error('Network error'));
            
            await useAgentStore.getState().fetchAgents();
            
            expect(useAgentStore.getState().isLoading).toBe(false);
            expect(useAgentStore.getState().error).toBe('Network error');
            expect(EventBus.emit).toHaveBeenCalledWith(expect.objectContaining({
                severity: 'error',
                text: expect.stringContaining('Network error')
            }));
        });
    });

    describe('updateAgent', () => {
        beforeEach(() => {
            useAgentStore.setState({ agents: [mockAgent1, mockAgent2] });
        });

        it('performs an optimistic update and persists successfully', async () => {
            await useAgentStore.getState().updateAgent('agent-1', { name: 'Updated Name', status: 'working' });
            
            const agent = useAgentStore.getState().agents.find(a => a.id === 'agent-1');
            expect(agent?.name).toBe('Updated Name');
            expect(agent?.status).toBe('working');
            
            expect(AgentService.persistAgentUpdate).toHaveBeenCalledWith('agent-1', { name: 'Updated Name', status: 'working' });
            expect(AgentService.loadAgents).not.toHaveBeenCalled(); // No revert needed
        });

        it('reverts state and emits event on persistence failure', async () => {
            (AgentService.persistAgentUpdate as Mock).mockRejectedValue(new Error('Sync failed'));
            
            // Revert will trigger loadAgents, let's mock it to return original agent
            (AgentService.loadAgents as Mock).mockResolvedValue([mockAgent1, mockAgent2]);
            
            await useAgentStore.getState().updateAgent('agent-1', { name: 'Updated Name' });
            
            // It should ultimately revert to the fetch result
            const agent = useAgentStore.getState().agents.find(a => a.id === 'agent-1');
            expect(agent?.name).toBe('Test Agent 1'); // reverted
            expect(useAgentStore.getState().error).toBe('Sync failed');
            
            expect(EventBus.emit).toHaveBeenCalledWith(expect.objectContaining({
                severity: 'warning',
                text: expect.stringContaining('Sync failed')
            }));
        });
    });

    describe('addAgent', () => {
        beforeEach(() => {
            useAgentStore.setState({ agents: [mockAgent1] });
        });

        it('optimistically adds an agent and persists', async () => {
            await useAgentStore.getState().addAgent(mockAgent2);
            
            expect(useAgentStore.getState().agents).toHaveLength(2);
            expect(useAgentStore.getState().agents[1]).toEqual(mockAgent2);
            
            expect(AgentService.persistAgentUpdate).toHaveBeenCalledWith('agent-2', mockAgent2);
        });

        it('reverts and emits event on persistence failure', async () => {
            (AgentService.persistAgentUpdate as Mock).mockRejectedValue(new Error('Create failed'));
            (AgentService.loadAgents as Mock).mockResolvedValue([mockAgent1]); // Revert state
            
            await useAgentStore.getState().addAgent(mockAgent2);
            
            expect(useAgentStore.getState().agents).toHaveLength(1); // Reverted
            expect(useAgentStore.getState().error).toBe('Create failed');
            
            expect(EventBus.emit).toHaveBeenCalledWith(expect.objectContaining({
                severity: 'error',
                text: expect.stringContaining('Create failed')
            }));
        });
    });

    describe('getAgent', () => {
        it('retrieves an agent by id', () => {
            useAgentStore.setState({ agents: [mockAgent1, mockAgent2] });
            
            expect(useAgentStore.getState().getAgent('agent-2')).toEqual(mockAgent2);
            expect(useAgentStore.getState().getAgent('non-existent')).toBeUndefined();
        });
    });

    describe('initTelemetry', () => {
        let updateCallback: (event: any) => void;

        beforeEach(() => {
            vi.mocked(TadpoleOSSocket.subscribeAgentUpdates).mockImplementation((cb: any) => {
                updateCallback = cb;
                return vi.fn(); // mock unsubscribe
            });
            
            (useWorkspaceStore.getState as Mock).mockReturnValue({
                clusters: [
                    { path: '/org/frontend', collaborators: ['agent-1'] }
                ]
            });

            useAgentStore.setState({ agents: [mockAgent1] });
        });

        it('subscribes to updates and applies them with workspace resolution', () => {
            const unsubscribe = useAgentStore.getState().initTelemetry();
            expect(TadpoleOSSocket.subscribeAgentUpdates).toHaveBeenCalled();
            expect(typeof unsubscribe).toBe('function');

            // Simulate incoming socket event
            updateCallback({
                type: 'agent:update',
                agentId: 'agent-1',
                data: { status: 'working', telemetry: { cpu: 50 } }
            });

            const agent = useAgentStore.getState().agents.find(a => a.id === 'agent-1') as any;
            
            // Check that it merged the initial mockAgent1 with the update data
            expect(agent).toMatchObject({
                id: 'agent-1',
                status: 'working',
                telemetry: { cpu: 50 },
                normalized: true // our mock normalizeAgent adds this
            });

            // normalizeAgent should have been called with the correct workspace path resolved from workspaceStore
            expect(AgentService.normalizeAgent).toHaveBeenCalledWith(
                expect.objectContaining({ id: 'agent-1', status: 'working' }),
                '/org/frontend' // resolved path
            );
        });
        
        it('resolves silo path if agent is not in any cluster', () => {
            useAgentStore.setState({ agents: [{ ...mockAgent1, id: 'agent-99' }] });
            useAgentStore.getState().initTelemetry();
            
            updateCallback({
                type: 'agent:update',
                agentId: 'agent-99', // Not in cluster mock
                data: { status: 'working' }
            });
            
            expect(AgentService.normalizeAgent).toHaveBeenCalledWith(
                expect.anything(),
                '/workspaces/agent-silo-agent-99' // fallback path
            );
        });
    });
});
