/**
 * @file memoryStore.test.ts
 * @description Suite for agent cognitive memory, search, and retrieval.
 * @module Stores/MemoryStore
 * @testedBehavior
 * - Persistence: Optimistic CUD of agent memory segments.
 * - Retrieval: Proxying of vector search requests to the backend.
 * - Safety: Error propagation via event_bus on retrieval failures.
 * @aiContext
 * - Mocks tadpole_os_service and event_bus.
 * - Validates optimistic state transitions before and after server confirmation.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { use_memory_store } from './memory_store';
import { tadpole_os_service } from '../services/tadpoleos_service';
import { event_bus } from '../services/event_bus';

// Mock dependencies
vi.mock('../services/tadpoleos_service', () => ({
    tadpole_os_service: {
        get_agent_memory: vi.fn(),
        search_memory: vi.fn(),
        save_agent_memory: vi.fn(),
        delete_agent_memory: vi.fn(),
    }
}));
vi.mock('../services/event_bus');

describe('use_memory_store', () => {
    beforeEach(() => {
        use_memory_store.setState({ memories: [], isLoading: false, error: null });
        vi.clearAllMocks();
    });

    describe('fetchMemories', () => {
        it('fetches memories and updates state successfully', async () => {
            const mockEntries = [
                { id: '1', text: 'memory 1', mission_id: 'm1', timestamp: 1000 },
                { id: '2', text: 'memory 2', mission_id: 'm2', timestamp: 2000 }
            ];
            vi.mocked(tadpole_os_service.get_agent_memory).mockResolvedValue({ entries: mockEntries } as any);

            const store = use_memory_store.getState();
            await store.fetchMemories('agent-1');

            const state = use_memory_store.getState();
            expect(state.isLoading).toBe(false);
            expect(state.error).toBeNull();
            expect(state.memories).toEqual(mockEntries);
            expect(tadpole_os_service.get_agent_memory).toHaveBeenCalledWith('agent-1');
        });

        it('handles failures during fetch', async () => {
            vi.mocked(tadpole_os_service.get_agent_memory).mockRejectedValue(new Error('Fetch failed'));
            const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

            const store = use_memory_store.getState();
            await store.fetchMemories('agent-1');

            const state = use_memory_store.getState();
            expect(state.isLoading).toBe(false);
            expect(state.error).toBe('Fetch failed');
            expect(state.memories).toHaveLength(0);

            consoleSpy.mockRestore();
        });
    });

    describe('saveMemory', () => {
        it('saves memory, updates state optimistically, and emits success event', async () => {
            vi.mocked(tadpole_os_service.save_agent_memory).mockResolvedValue({ id: 'new-id' } as any);
            
            // Start with some existing memory
            use_memory_store.setState({ memories: [{ id: 'old-id', text: 'old', mission_id: 'x', timestamp: 0 }] });
            
            const store = use_memory_store.getState();
            await store.saveMemory('agent-1', 'new memory text');

            const state = use_memory_store.getState();
            expect(state.isLoading).toBe(false);
            expect(state.memories).toHaveLength(2);
            expect(state.memories[0].text).toBe('new memory text');
            expect(state.memories[0].id).toBe('new-id');
            expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({
                severity: 'success'
            }));
            expect(tadpole_os_service.save_agent_memory).toHaveBeenCalledWith('agent-1', 'new memory text');
        });

        it('handles save failures and emits error event', async () => {
            vi.mocked(tadpole_os_service.save_agent_memory).mockRejectedValue(new Error('Save failed'));
            const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

            const store = use_memory_store.getState();
            await store.saveMemory('agent-1', 'bad memory');

            const state = use_memory_store.getState();
            expect(state.isLoading).toBe(false);
            expect(state.error).toBe('Save failed');
            expect(state.memories).toHaveLength(0); // Assuming it started at 0
            expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({
                severity: 'error'
            }));

            consoleSpy.mockRestore();
        });
    });

    describe('deleteMemory', () => {
        it('deletes memory, removes from state optimistically, and emits success event', async () => {
            vi.mocked(tadpole_os_service.delete_agent_memory).mockResolvedValue({} as any);
            
            use_memory_store.setState({ 
                memories: [
                    { id: 'keep-me', text: 'keep', mission_id: 'x', timestamp: 0 },
                    { id: 'delete-me', text: 'delete', mission_id: 'y', timestamp: 0 }
                ] 
            });

            const store = use_memory_store.getState();
            const result = await store.deleteMemory('agent-1', 'delete-me');

            const state = use_memory_store.getState();
            expect(result).toBe(true);
            expect(state.memories).toHaveLength(1);
            expect(state.memories[0].id).toBe('keep-me');
            expect(tadpole_os_service.delete_agent_memory).toHaveBeenCalledWith('agent-1', 'delete-me');
            expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({ severity: 'success' }));
        });

        it('handles deletion failure and emits error event', async () => {
            vi.mocked(tadpole_os_service.delete_agent_memory).mockRejectedValue(new Error('Delete DB Error'));
            const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

            use_memory_store.setState({ memories: [{ id: 'keep-me', text: 'keep', mission_id: 'x', timestamp: 0 }] });

            const store = use_memory_store.getState();
            const result = await store.deleteMemory('agent-1', 'keep-me');

            const state = use_memory_store.getState();
            expect(result).toBe(false);
            // On failure, optimistic update does not run to revert, but we caught it before state modified
            expect(state.memories).toHaveLength(1);
            expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({ severity: 'error' }));

            consoleSpy.mockRestore();
        });
    });

    describe('searchMemories', () => {
        it('proxies search via tadpole_os_service.search_memory', async () => {
            const mockSearchReturn = [{ id: 'found', text: 'search hit', mission_id: '1', timestamp: 1 }];
            vi.mocked(tadpole_os_service.search_memory).mockResolvedValue({ status: 'success', entries: mockSearchReturn } as any);

            const store = use_memory_store.getState();
            const results = await store.searchMemories('hit');

            expect(results).toEqual(mockSearchReturn);
            expect(tadpole_os_service.search_memory).toHaveBeenCalledWith('hit');
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

