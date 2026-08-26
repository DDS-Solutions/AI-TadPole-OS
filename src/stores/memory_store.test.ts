/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend State Store / memory_store.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Store mutations maintain immutable state transitions and notify subscribers deterministically.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { use_memory_store } from './memory_store';
import { agent_api_service } from '../services/agent';
import { event_bus } from '../services/event_bus';
import { log_error } from '../services/system_utils';

// Mock dependencies
vi.mock('../services/agent', () => ({
    agent_api_service: {
        get_agent_memory: vi.fn(),
        search_memory: vi.fn(),
        save_agent_memory: vi.fn(),
        delete_agent_memory: vi.fn(),
    }
}));
vi.mock('../services/event_bus', () => ({
    event_bus: {
        emit_log: vi.fn(),
        subscribe_traces: vi.fn(() => () => {}),
    }
}));
vi.mock('../services/system_utils', () => ({
    log_error: vi.fn(),
}));

describe('use_memory_store', () => {
    beforeEach(() => {
        use_memory_store.setState({ memories: [], is_loading: false, error: null });
        vi.clearAllMocks();
    });

    describe('fetch_memories', () => {
        it('fetches memories and updates state successfully', async () => {
            const mock_entries = [
                { id: '1', text: 'memory 1', mission_id: 'm1', timestamp: 1000 },
                { id: '2', text: 'memory 2', mission_id: 'm2', timestamp: 2000 }
            ];
            vi.mocked(agent_api_service.get_agent_memory).mockResolvedValue({ entries: mock_entries } as any);

            const store = use_memory_store.getState();
            await store.fetch_memories('agent-1');

            const state = use_memory_store.getState();
            expect(state.is_loading).toBe(false);
            expect(state.error).toBeNull();
            expect(state.memories).toEqual(mock_entries);
            expect(agent_api_service.get_agent_memory).toHaveBeenCalledWith('agent-1');
        });

        it('handles failures during fetch', async () => {
            vi.mocked(agent_api_service.get_agent_memory).mockRejectedValue(new Error('Fetch failed'));

            const store = use_memory_store.getState();
            await store.fetch_memories('agent-1');

            const state = use_memory_store.getState();
            expect(state.is_loading).toBe(false);
            expect(state.error).toBe('Fetch failed');
            expect(log_error).toHaveBeenCalledWith('MemoryStore', 'Memory Retrieval Failed', expect.any(Error));
        });
    });

    describe('save_memory', () => {
        it('saves memory, updates state optimistically, and emits success event', async () => {
            vi.mocked(agent_api_service.save_agent_memory).mockResolvedValue({ id: 'new-id' } as any);
            
            // Start with some existing memory
            use_memory_store.setState({ memories: [{ id: 'old-id', text: 'old', mission_id: 'x', timestamp: 0 }] });
            
            const store = use_memory_store.getState();
            await store.save_memory('agent-1', 'new memory text');

            const state = use_memory_store.getState();
            expect(state.is_loading).toBe(false);
            expect(state.memories).toHaveLength(2);
            expect(state.memories[0].text).toBe('new memory text');
            expect(state.memories[0].id).toBe('new-id');
            expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({
                severity: 'success'
            }));
            expect(agent_api_service.save_agent_memory).toHaveBeenCalledWith('agent-1', 'new memory text');
        });

        it('handles save failures and emits error event', async () => {
            vi.mocked(agent_api_service.save_agent_memory).mockRejectedValue(new Error('Save failed'));

            const store = use_memory_store.getState();
            await store.save_memory('agent-1', 'bad memory');

            const state = use_memory_store.getState();
            expect(state.is_loading).toBe(false);
            expect(state.error).toBe('Save failed');
            expect(log_error).toHaveBeenCalledWith('MemoryStore', 'Memory Save Failed', expect.any(Error));
        });
    });

    describe('delete_memory', () => {
        it('deletes memory, removes from state optimistically, and emits success event', async () => {
            vi.mocked(agent_api_service.delete_agent_memory).mockResolvedValue({} as any);
            
            use_memory_store.setState({ 
                memories: [
                    { id: 'keep-me', text: 'keep', mission_id: 'x', timestamp: 0 },
                    { id: 'delete-me', text: 'delete', mission_id: 'y', timestamp: 0 }
                ] 
            });

            const store = use_memory_store.getState();
            const result = await store.delete_memory('agent-1', 'delete-me');

            const state = use_memory_store.getState();
            expect(result).toBe(true);
            expect(state.memories).toHaveLength(1);
            expect(state.memories[0].id).toBe('keep-me');
            expect(agent_api_service.delete_agent_memory).toHaveBeenCalledWith('agent-1', 'delete-me');
            expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({ severity: 'success' }));
        });

        it('handles deletion failure and emits error event', async () => {
            vi.mocked(agent_api_service.delete_agent_memory).mockRejectedValue(new Error('Delete DB Error'));

            use_memory_store.setState({ memories: [{ id: 'keep-me', text: 'keep', mission_id: 'x', timestamp: 0 }] });

            const store = use_memory_store.getState();
            const result = await store.delete_memory('agent-1', 'keep-me');

            const state = use_memory_store.getState();
            expect(result).toBe(false);
            expect(state.memories).toHaveLength(1);
            expect(log_error).toHaveBeenCalledWith('MemoryStore', 'Memory Deletion Failed', expect.any(Error));
        });
    });

    describe('search_memories', () => {
        it('proxies search via agent_api_service.search_memory', async () => {
            const mock_search_return = [{ id: 'found', text: 'search hit', mission_id: '1', timestamp: 1 }];
            vi.mocked(agent_api_service.search_memory).mockResolvedValue({ status: 'success', entries: mock_search_return } as any);

            const store = use_memory_store.getState();
            const results = await store.search_memories('hit');

            expect(results).toEqual(mock_search_return);
            expect(agent_api_service.search_memory).toHaveBeenCalledWith('hit');
        });
    });

    describe('clear', () => {
        it('resets local state correctly', () => {
            use_memory_store.setState({ memories: [{ id: '1', text: 't', mission_id: '1', timestamp: 1 }], error: 'err' });
            const store = use_memory_store.getState();
            
            store.clear();

            const state = use_memory_store.getState();
            expect(state.memories).toHaveLength(0);
            expect(state.error).toBeNull();
        });
    });
});
