/**
 * @file nodeStore.test.ts
 * @description Suite for distributed infrastructure discovery and node health.
 * @module Stores/NodeStore
 * @testedBehavior
 * - Network Discovery: Scanning for available engine nodes and merging results.
 * - Infrastructure: Monitoring node status and availability across the cluster.
 * @aiContext
 * - Mocks SystemApiService and EventBus.
 * - Validates successful scan events and failure logging.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { useNodeStore } from './nodeStore';
import { SystemApiService } from '../services/SystemApiService';
import { EventBus } from '../services/eventBus';

// Mock dependencies
vi.mock('../services/SystemApiService');
vi.mock('../services/eventBus');

describe('useNodeStore', () => {
    beforeEach(() => {
        // Reset the store state before each test
        useNodeStore.setState({ nodes: [], isLoading: false });
        vi.clearAllMocks();
    });

    it('has the correct initial state', () => {
        const state = useNodeStore.getState();
        expect(state.nodes).toEqual([]);
        expect(state.isLoading).toBe(false);
    });

    describe('fetchNodes', () => {
        it('fetches nodes successfully and updates state', async () => {
            const mockNodes = [{ id: 'node1', url: 'http://localhost:8000', name: 'Test Node', status: 'online' }];
            vi.mocked(SystemApiService.getNodes).mockResolvedValue(mockNodes as any);

            const store = useNodeStore.getState();
            
            // Trigger fetch and verify loading state intermediate
            const fetchPromise = store.fetchNodes();
            expect(useNodeStore.getState().isLoading).toBe(true);
            
            await fetchPromise;

            const updatedState = useNodeStore.getState();
            expect(updatedState.isLoading).toBe(false);
            expect(updatedState.nodes).toEqual(mockNodes);
            expect(SystemApiService.getNodes).toHaveBeenCalled();
        });

        it('handles failure during fetchNodes', async () => {
            vi.mocked(SystemApiService.getNodes).mockRejectedValue(new Error('Network error'));
            const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

            const store = useNodeStore.getState();
            await store.fetchNodes();

            const updatedState = useNodeStore.getState();
            expect(updatedState.isLoading).toBe(false);
            expect(updatedState.nodes).toEqual([]); // Should remain empty
            expect(consoleSpy).toHaveBeenCalledWith('Failed to fetch nodes:', expect.any(Error));
        });
    });

    describe('discoverNodes', () => {
        it('discovers new nodes, emits event, and refetches', async () => {
            const mockDiscoverResult = { status: 'success', discovered: ['http://localhost:8001'] };
            vi.mocked(SystemApiService.discoverNodes).mockResolvedValue(mockDiscoverResult);
            // Mock subsequent fetchNodes call
            vi.mocked(SystemApiService.getNodes).mockResolvedValue([{ id: 'node2', url: 'http://localhost:8001' } as any]);

            const store = useNodeStore.getState();
            await store.discoverNodes();

            const updatedState = useNodeStore.getState();
            expect(updatedState.isLoading).toBe(false);
            expect(SystemApiService.discoverNodes).toHaveBeenCalled();
            expect(EventBus.emit).toHaveBeenCalledWith({
                source: 'System',
                text: '📡 Network Scan: 1 new node(s) identified.',
                severity: 'success'
            });
            // Should have refetched nodes
            expect(SystemApiService.getNodes).toHaveBeenCalled();
        });

        it('handles discovery when no nodes are found', async () => {
            const mockDiscoverResult = { status: 'success', discovered: [] };
            vi.mocked(SystemApiService.discoverNodes).mockResolvedValue(mockDiscoverResult);

            const store = useNodeStore.getState();
            await store.discoverNodes();

            const updatedState = useNodeStore.getState();
            expect(updatedState.isLoading).toBe(false);
            expect(EventBus.emit).toHaveBeenCalledWith({
                source: 'System',
                text: '📡 Network Scan: No new nodes found.',
                severity: 'info'
            });
            // Should NOT have refetched nodes
            expect(SystemApiService.getNodes).not.toHaveBeenCalled();
        });

        it('handles failure during discovery', async () => {
            vi.mocked(SystemApiService.discoverNodes).mockRejectedValue(new Error('Scan failed'));
            const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

            const store = useNodeStore.getState();
            await store.discoverNodes();

            const updatedState = useNodeStore.getState();
            expect(updatedState.isLoading).toBe(false);
            expect(consoleSpy).toHaveBeenCalledWith('Failed to discover nodes:', expect.any(Error));
        });
    });
});
