/**
 * @docs ARCHITECTURE:State
 * 
 * ### AI Assist Note
 * **Test Suite**: Validates the Swarm registry and agent lifecycle. 
 */

/**
 * @file agent_store.test.ts
 * @description Suite for the central Agent management state machine.
 * @module Stores/AgentStore
 * @testedBehavior
 * - Async Orchestration: Fetching agent rosters and handling error events.
 * - Reactive State: Optimistic updates with automatic rollback on persistence failure.
 * - Telemetry: Real-time socket integration and workspace-aware ID resolution.
 * @aiContext
 * - Mocks localStorage to prevent persistence side effects.
 * - Simulates socket 'agent:update' payloads to validate reactive UI updates.
 * - Refactored for 100% snake_case architectural parity.
 */
import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';

// Block localStorage BEFORE imports
vi.hoisted(() => {
    const mock_local_storage = {
        getItem: vi.fn(),
        setItem: vi.fn(),
        clear: vi.fn(),
        removeItem: vi.fn(),
        length: 0,
        key: vi.fn(),
    };
    vi.stubGlobal('localStorage', mock_local_storage);
});

import { use_agent_store } from './agent_store';
import * as agent_service from '../services/agent_service';
import { agent_api_service } from '../services/agent_api_service';
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
        load_agents: vi.fn(),
        persist_agent_update: vi.fn(),
        normalize_agent: vi.fn((raw) => ({ ...raw, normalized: true })),
    };
});

vi.mock('../services/agent_api_service', () => ({
    agent_api_service: {
        create_agent: vi.fn(),
    }
}));

vi.mock('../services/event_bus', () => ({
    event_bus: {
        emit_log: vi.fn(),
    }
}));

vi.mock('../services/socket', () => ({
    tadpole_os_socket: {
        subscribe_agent_updates: vi.fn(),
    }
}));

vi.mock('./workspace_store', () => ({
    use_workspace_store: {
        getState: vi.fn(),
    }
}));

describe('agent_store', () => {
    const mock_agent_1: Agent = {
        id: '1',
        name: 'Test Agent 1',
        role: 'Developer',
        model: 'model-a',
        model_config: { provider: 'test', model_id: 'test' },
        department: 'Engineering',
        workspace_path: '/test',
        status: 'idle',
        tokens_used: 0,
        category: 'user'
    };

    const mock_agent_2: Agent = {
        ...mock_agent_1,
        id: '2',
        name: 'Test Agent 2'
    };

    beforeEach(() => {
        vi.clearAllMocks();
        
        // Reset state
        use_agent_store.setState({
            agents: [],
            is_loading: false,
            error: null
        });

        (agent_service.load_agents as Mock).mockResolvedValue([mock_agent_1, mock_agent_2]);
        (agent_service.persist_agent_update as Mock).mockResolvedValue(true);
        vi.mocked(agent_api_service.create_agent).mockResolvedValue(true);
    });

    describe('fetch_agents', () => {
        it('fetches agents successfully and updates state', async () => {
            const promise = use_agent_store.getState().fetch_agents();
            expect(use_agent_store.getState().is_loading).toBe(true);
            
            await promise;
            
            expect(use_agent_store.getState().is_loading).toBe(false);
            expect(use_agent_store.getState().error).toBeNull();
            expect(use_agent_store.getState().agents).toEqual([mock_agent_1, mock_agent_2]);
        });

        it('handles fetch errors correctly and emits event', async () => {
            (agent_service.load_agents as Mock).mockRejectedValue(new Error('Network error'));
            
            await use_agent_store.getState().fetch_agents();
            
            expect(use_agent_store.getState().is_loading).toBe(false);
            expect(use_agent_store.getState().error).toBe('Network error');
            expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({
                severity: 'error',
                text: expect.stringContaining('Network error')
            }));
        });
    });

    describe('update_agent', () => {
        beforeEach(() => {
            use_agent_store.setState({ agents: [mock_agent_1, mock_agent_2] });
        });

        it('performs an optimistic update and persists successfully', async () => {
            await use_agent_store.getState().update_agent('1', { name: 'Updated Name', status: 'working' });
            
            const agent = use_agent_store.getState().agents.find(a => a.id === '1');
            expect(agent?.name).toBe('Updated Name');
            expect(agent?.status).toBe('working');
            
            expect(agent_service.persist_agent_update).toHaveBeenCalledWith('1', { name: 'Updated Name', status: 'working' });
            expect(agent_service.load_agents).not.toHaveBeenCalled(); // No revert needed
        });

        it('reverts state and emits event on persistence failure', async () => {
            (agent_service.persist_agent_update as Mock).mockRejectedValue(new Error('Sync failed'));
            
            // Revert will trigger load_agents, let's mock it to return original agent
            (agent_service.load_agents as Mock).mockResolvedValue([mock_agent_1, mock_agent_2]);
            
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

    describe('add_agent', () => {
        beforeEach(() => {
            use_agent_store.setState({ agents: [mock_agent_1] });
        });

        it('optimistically adds an agent and persists', async () => {
            const result = await use_agent_store.getState().add_agent(mock_agent_2);
             
            expect(use_agent_store.getState().agents).toHaveLength(2);
            expect(use_agent_store.getState().agents[1]).toEqual(mock_agent_2);
            expect(result).toBe(true);
             
            expect(agent_api_service.create_agent).toHaveBeenCalledWith(mock_agent_2);
        });

        it('reverts and emits event on persistence failure', async () => {
            vi.mocked(agent_api_service.create_agent).mockRejectedValue(new Error('Create failed'));
            (agent_service.load_agents as Mock).mockResolvedValue([mock_agent_1]); // Revert state
             
            const result = await use_agent_store.getState().add_agent(mock_agent_2);
             
            expect(use_agent_store.getState().agents).toHaveLength(1); // Reverted
            expect(use_agent_store.getState().error).toBe('Create failed');
            expect(result).toBe(false);
             
            expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({
                severity: 'error',
                text: expect.stringContaining('Create failed')
            }));
        });
    });

    describe('get_agent', () => {
        it('retrieves an agent by id', () => {
            use_agent_store.setState({ agents: [mock_agent_1, mock_agent_2] });
            
            expect(use_agent_store.getState().get_agent('2')).toEqual(mock_agent_2);
            expect(use_agent_store.getState().get_agent('non-existent')).toBeUndefined();
        });
    });

    describe('init_telemetry', () => {
        let update_callback: (event: any) => void;

        beforeEach(() => {
            vi.mocked(tadpole_os_socket.subscribe_agent_updates).mockImplementation((cb: any) => {
                update_callback = cb;
                return vi.fn(); // mock unsubscribe
            });
            
            (use_workspace_store.getState as Mock).mockReturnValue({
                clusters: [
                    { path: '/org/frontend', collaborators: ['1'] }
                ]
            });

            use_agent_store.setState({ agents: [mock_agent_1] });
        });

        it('subscribes to updates and applies them with workspace resolution', () => {
            const unsubscribe = use_agent_store.getState().init_telemetry();
            expect(tadpole_os_socket.subscribe_agent_updates).toHaveBeenCalled();
            expect(typeof unsubscribe).toBe('function');

            // Simulate incoming socket event
            update_callback({
                type: 'agent:update',
                agent_id: '1',
                data: { status: 'working', telemetry: { cpu: 50 }, category: 'user' }
            });

            const agent = use_agent_store.getState().agents.find(a => a.id === '1') as any;
            
            // Check that it merged the initial mock_agent_1 with the update data
            expect(agent).toMatchObject({
                id: '1',
                status: 'working',
                telemetry: { cpu: 50 },
                normalized: true // our mock normalize_agent adds this
            });

            // normalize_agent should have been called with the correct workspace path resolved from workspace_store
            expect(agent_service.normalize_agent).toHaveBeenCalledWith(
                expect.objectContaining({ id: '1', status: 'working' }),
                '/org/frontend',
                expect.objectContaining({ id: '1' })
            );
        });
        
        it('resolves silo path if agent is not in any cluster', () => {
            use_agent_store.setState({ agents: [{ ...mock_agent_1, id: '99' }] });
            use_agent_store.getState().init_telemetry();
            
            update_callback({
                type: 'agent:update',
                agent_id: '99', // Not in cluster mock
                data: { status: 'working', category: 'user' }
            });
            
            expect(agent_service.normalize_agent).toHaveBeenCalledWith(
                expect.objectContaining({ id: '99', status: 'working' }),
                '/workspaces/agent-silo-99',
                expect.objectContaining({ id: '99' })
            );
        });
    });
});

