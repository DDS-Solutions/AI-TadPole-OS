/**
 * @file memoryStore.test.ts
 * @description Suite for agent cognitive memory, search, and retrieval.
 * @module Stores/MemoryStore
 * @testedBehavior
 * - Persistence: Optimistic CUD of agent memory segments.
 * - Retrieval: Proxying of vector search requests to the backend.
 * - Safety: Error propagation via EventBus on retrieval failures.
 * @aiContext
 * - Mocks TadpoleOSService and EventBus.
 * - Validates optimistic state transitions before and after server confirmation.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { useMemoryStore } from './memoryStore';
import { TadpoleOSService } from '../services/tadpoleosService';
import { EventBus } from '../services/eventBus';

// Mock dependencies
vi.mock('../services/tadpoleosService', () => ({
    TadpoleOSService: {
        getAgentMemory: vi.fn(),
        searchMemory: vi.fn(),
        saveAgentMemory: vi.fn(),
        deleteAgentMemory: vi.fn(),
    }
}));
vi.mock('../services/eventBus');

describe('useMemoryStore', () => {
    beforeEach(() => {
        useMemoryStore.setState({ memories: [], isLoading: false, error: null });
        vi.clearAllMocks();
    });

    describe('fetchMemories', () => {
        it('fetches memories and updates state successfully', async () => {
            const mockEntries = [
                { id: '1', text: 'memory 1', mission_id: 'm1', timestamp: 1000 },
                { id: '2', text: 'memory 2', mission_id: 'm2', timestamp: 2000 }
            ];
            vi.mocked(TadpoleOSService.getAgentMemory).mockResolvedValue({ entries: mockEntries } as any);

            const store = useMemoryStore.getState();
            await store.fetchMemories('agent-1');

            const state = useMemoryStore.getState();
            expect(state.isLoading).toBe(false);
            expect(state.error).toBeNull();
            expect(state.memories).toEqual(mockEntries);
            expect(TadpoleOSService.getAgentMemory).toHaveBeenCalledWith('agent-1');
        });

        it('handles failures during fetch', async () => {
            vi.mocked(TadpoleOSService.getAgentMemory).mockRejectedValue(new Error('Fetch failed'));
            const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

            const store = useMemoryStore.getState();
            await store.fetchMemories('agent-1');

            const state = useMemoryStore.getState();
            expect(state.isLoading).toBe(false);
            expect(state.error).toBe('Fetch failed');
            expect(state.memories).toHaveLength(0);

            consoleSpy.mockRestore();
        });
    });

    describe('saveMemory', () => {
        it('saves memory, updates state optimistically, and emits success event', async () => {
            vi.mocked(TadpoleOSService.saveAgentMemory).mockResolvedValue({ id: 'new-id' } as any);
            
            // Start with some existing memory
            useMemoryStore.setState({ memories: [{ id: 'old-id', text: 'old', mission_id: 'x', timestamp: 0 }] });
            
            const store = useMemoryStore.getState();
            await store.saveMemory('agent-1', 'new memory text');

            const state = useMemoryStore.getState();
            expect(state.isLoading).toBe(false);
            expect(state.memories).toHaveLength(2);
            expect(state.memories[0].text).toBe('new memory text');
            expect(state.memories[0].id).toBe('new-id');
            expect(EventBus.emit).toHaveBeenCalledWith(expect.objectContaining({
                severity: 'success'
            }));
            expect(TadpoleOSService.saveAgentMemory).toHaveBeenCalledWith('agent-1', 'new memory text');
        });

        it('handles save failures and emits error event', async () => {
            vi.mocked(TadpoleOSService.saveAgentMemory).mockRejectedValue(new Error('Save failed'));
            const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

            const store = useMemoryStore.getState();
            await store.saveMemory('agent-1', 'bad memory');

            const state = useMemoryStore.getState();
            expect(state.isLoading).toBe(false);
            expect(state.error).toBe('Save failed');
            expect(state.memories).toHaveLength(0); // Assuming it started at 0
            expect(EventBus.emit).toHaveBeenCalledWith(expect.objectContaining({
                severity: 'error'
            }));

            consoleSpy.mockRestore();
        });
    });

    describe('deleteMemory', () => {
        it('deletes memory, removes from state optimistically, and emits success event', async () => {
            vi.mocked(TadpoleOSService.deleteAgentMemory).mockResolvedValue({} as any);
            
            useMemoryStore.setState({ 
                memories: [
                    { id: 'keep-me', text: 'keep', mission_id: 'x', timestamp: 0 },
                    { id: 'delete-me', text: 'delete', mission_id: 'y', timestamp: 0 }
                ] 
            });

            const store = useMemoryStore.getState();
            const result = await store.deleteMemory('agent-1', 'delete-me');

            const state = useMemoryStore.getState();
            expect(result).toBe(true);
            expect(state.memories).toHaveLength(1);
            expect(state.memories[0].id).toBe('keep-me');
            expect(TadpoleOSService.deleteAgentMemory).toHaveBeenCalledWith('agent-1', 'delete-me');
            expect(EventBus.emit).toHaveBeenCalledWith(expect.objectContaining({ severity: 'success' }));
        });

        it('handles deletion failure and emits error event', async () => {
            vi.mocked(TadpoleOSService.deleteAgentMemory).mockRejectedValue(new Error('Delete DB Error'));
            const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

            useMemoryStore.setState({ memories: [{ id: 'keep-me', text: 'keep', mission_id: 'x', timestamp: 0 }] });

            const store = useMemoryStore.getState();
            const result = await store.deleteMemory('agent-1', 'keep-me');

            const state = useMemoryStore.getState();
            expect(result).toBe(false);
            // On failure, optimistic update does not run to revert, but we caught it before state modified
            expect(state.memories).toHaveLength(1);
            expect(EventBus.emit).toHaveBeenCalledWith(expect.objectContaining({ severity: 'error' }));

            consoleSpy.mockRestore();
        });
    });

    describe('searchMemories', () => {
        it('proxies search via TadpoleOSService.searchMemory', async () => {
            const mockSearchReturn = [{ id: 'found', text: 'search hit', mission_id: '1', timestamp: 1 }];
            vi.mocked(TadpoleOSService.searchMemory).mockResolvedValue({ status: 'success', entries: mockSearchReturn } as any);

            const store = useMemoryStore.getState();
            const results = await store.searchMemories('hit');

            expect(results).toEqual(mockSearchReturn);
            expect(TadpoleOSService.searchMemory).toHaveBeenCalledWith('hit');
        });
    });

    describe('clear', () => {
        it('resets local state correctly', () => {
            useMemoryStore.setState({ memories: [{ id: '1', text: 't', mission_id: '1', timestamp: 1 }], error: 'err' });
            const store = useMemoryStore.getState();
            
            store.clear();

            const state = useMemoryStore.getState();
            expect(state.memories).toHaveLength(0);
            expect(state.error).toBeNull();
        });
    });
});
