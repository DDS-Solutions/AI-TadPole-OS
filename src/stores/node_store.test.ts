/**
 * @file nodeStore.test.ts
 * @description Suite for distributed infrastructure discovery and node health.
 * @module Stores/NodeStore
 * @testedBehavior
 * - Network Discovery: Scanning for available engine nodes and merging results.
 * - Infrastructure: Monitoring node status and availability across the cluster.
 * @aiContext
 * - Mocks system_api_service and event_bus.
 * - Validates successful scan events and failure logging.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { use_node_store } from './node_store';
import { system_api_service } from '../services/system_api_service';
import { event_bus } from '../services/event_bus';

// Mock dependencies
vi.mock('../services/system_api_service');
vi.mock('../services/event_bus');

describe('use_node_store', () => {
    beforeEach(() => {
        // Reset the store state before each test
        use_node_store.setState({ nodes: [], isLoading: false });
        vi.clearAllMocks();
    });

    it('has the correct initial state', () => {
        const state = use_node_store.getState();
        expect(state.nodes).toEqual([]);
        expect(state.isLoading).toBe(false);
    });

    describe('fetchNodes', () => {
        it('fetches nodes successfully and updates state', async () => {
            const mockNodes = [{ id: 'node1', url: 'http://localhost:8000', name: 'Test Node', status: 'online' }];
            vi.mocked(system_api_service.get_nodes).mockResolvedValue(mockNodes as any);

            const store = use_node_store.getState();
            
            // Trigger fetch and verify loading state intermediate
            const fetchPromise = store.fetchNodes();
            expect(use_node_store.getState().isLoading).toBe(true);
            
            await fetchPromise;

            const updatedState = use_node_store.getState();
            expect(updatedState.isLoading).toBe(false);
            expect(updatedState.nodes).toEqual(mockNodes);
            expect(system_api_service.get_nodes).toHaveBeenCalled();
        });

        it('handles failure during fetchNodes', async () => {
            vi.mocked(system_api_service.get_nodes).mockRejectedValue(new Error('Network error'));
            const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

            const store = use_node_store.getState();
            await store.fetchNodes();

            const updatedState = use_node_store.getState();
            expect(updatedState.isLoading).toBe(false);
            expect(updatedState.nodes).toEqual([]); // Should remain empty
            expect(consoleSpy).toHaveBeenCalledWith('Failed to fetch nodes:', expect.any(Error));
        });
    });

    describe('discover_nodes', () => {
        it('discovers new nodes, emits event, and refetches', async () => {
            const mockDiscoverResult = { status: 'success', discovered: ['http://localhost:8001'] };
            vi.mocked(system_api_service.discover_nodes).mockResolvedValue(mockDiscoverResult);
            // Mock subsequent fetchNodes call
            vi.mocked(system_api_service.get_nodes).mockResolvedValue([{ id: 'node2', url: 'http://localhost:8001' } as any]);

            const store = use_node_store.getState();
            await store.discover_nodes();

            const updatedState = use_node_store.getState();
            expect(updatedState.isLoading).toBe(false);
            expect(system_api_service.discover_nodes).toHaveBeenCalled();
            expect(event_bus.emit_log).toHaveBeenCalledWith({
                source: 'System',
                text: '📡 Network Scan: 1 new node(s) identified.',
                severity: 'success'
            });
            // Should have refetched nodes
            expect(system_api_service.get_nodes).toHaveBeenCalled();
        });

        it('handles discovery when no nodes are found', async () => {
            const mockDiscoverResult = { status: 'success', discovered: [] };
            vi.mocked(system_api_service.discover_nodes).mockResolvedValue(mockDiscoverResult);

            const store = use_node_store.getState();
            await store.discover_nodes();

            const updatedState = use_node_store.getState();
            expect(updatedState.isLoading).toBe(false);
            expect(event_bus.emit_log).toHaveBeenCalledWith({
                source: 'System',
                text: '📡 Network Scan: No new nodes found.',
                severity: 'info'
            });
            // Should NOT have refetched nodes
            expect(system_api_service.get_nodes).not.toHaveBeenCalled();
        });

        it('handles failure during discovery', async () => {
            vi.mocked(system_api_service.discover_nodes).mockRejectedValue(new Error('Scan failed'));
            const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

            const store = use_node_store.getState();
            await store.discover_nodes();

            const updatedState = use_node_store.getState();
            expect(updatedState.isLoading).toBe(false);
            expect(consoleSpy).toHaveBeenCalledWith('Failed to discover nodes:', expect.any(Error));
        });
    });
});

