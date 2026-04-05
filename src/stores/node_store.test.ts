/**
 * @docs ARCHITECTURE:State
 * 
 * ### AI Assist Note
 * **Test Suite**: Validates the Physical and virtual node cluster mapping. 
 */

/**
 * @file node_store.test.ts
 * @description Suite for distributed infrastructure discovery and node health.
 * @module Stores/NodeStore
 * @testedBehavior
 * - Network Discovery: Scanning for available engine nodes and merging results.
 * - Infrastructure: Monitoring node status and availability across the cluster.
 * @aiContext
 * - Mocks tadpole_os_service and event_bus.
 * - Validates successful scan events and failure logging.
 * - Refactored for 100% snake_case architectural parity.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { use_node_store } from './node_store';
import { tadpole_os_service } from '../services/tadpoleos_service';
import { event_bus } from '../services/event_bus';

// Mock dependencies
vi.mock('../services/tadpoleos_service', () => ({
    tadpole_os_service: {
        get_nodes: vi.fn(),
        discover_nodes: vi.fn(),
    }
}));

vi.mock('../services/event_bus', () => ({
    event_bus: {
        emit_log: vi.fn(),
    }
}));

describe('use_node_store', () => {
    beforeEach(() => {
        // Reset the store state before each test
        use_node_store.setState({ nodes: [], is_loading: false });
        vi.clearAllMocks();
    });

    it('has the correct initial state', () => {
        const state = use_node_store.getState();
        expect(state.nodes).toEqual([]);
        expect(state.is_loading).toBe(false);
    });

    describe('fetch_nodes', () => {
        it('fetches nodes successfully and updates state', async () => {
            const mock_nodes = [{ id: 'node1', url: 'http://localhost:8000', name: 'Test Node', status: 'online' }];
            vi.mocked(tadpole_os_service.get_nodes).mockResolvedValue(mock_nodes as any);

            const store = use_node_store.getState();
            
            // Trigger fetch and verify loading state intermediate
            const fetch_promise = store.fetch_nodes();
            expect(use_node_store.getState().is_loading).toBe(true);
            
            await fetch_promise;

            const updated_state = use_node_store.getState();
            expect(updated_state.is_loading).toBe(false);
            expect(updated_state.nodes).toEqual(mock_nodes);
            expect(tadpole_os_service.get_nodes).toHaveBeenCalled();
        });

        it('handles failure during fetch_nodes', async () => {
            vi.mocked(tadpole_os_service.get_nodes).mockRejectedValue(new Error('Network error'));
            const console_spy = vi.spyOn(console, 'error').mockImplementation(() => {});

            const store = use_node_store.getState();
            await store.fetch_nodes();

            const updated_state = use_node_store.getState();
            expect(updated_state.is_loading).toBe(false);
            expect(updated_state.nodes).toEqual([]); // Should remain empty
            expect(console_spy).toHaveBeenCalledWith('Failed to fetch nodes:', expect.any(Error));
        });
    });

    describe('discover_nodes', () => {
        it('discovers new nodes, emits event, and refetches', async () => {
            const mock_discover_result = { status: 'success', discovered: ['http://localhost:8001'] };
            vi.mocked(tadpole_os_service.discover_nodes).mockResolvedValue(mock_discover_result);
            // Mock subsequent fetch_nodes call
            vi.mocked(tadpole_os_service.get_nodes).mockResolvedValue([{ id: 'node2', url: 'http://localhost:8001' } as any]);

            const store = use_node_store.getState();
            await store.discover_nodes();

            const updated_state = use_node_store.getState();
            expect(updated_state.is_loading).toBe(false);
            expect(tadpole_os_service.discover_nodes).toHaveBeenCalled();
            expect(event_bus.emit_log).toHaveBeenCalledWith({
                source: 'System',
                text: '📡 Network Scan: 1 new node(s) identified.',
                severity: 'success'
            });
            // Should have refetched nodes
            expect(tadpole_os_service.get_nodes).toHaveBeenCalled();
        });

        it('handles discovery when no nodes are found', async () => {
            const mock_discover_result = { status: 'success', discovered: [] };
            vi.mocked(tadpole_os_service.discover_nodes).mockResolvedValue(mock_discover_result);

            const store = use_node_store.getState();
            await store.discover_nodes();

            const updated_state = use_node_store.getState();
            expect(updated_state.is_loading).toBe(false);
            expect(event_bus.emit_log).toHaveBeenCalledWith({
                source: 'System',
                text: '📡 Network Scan: No new nodes found.',
                severity: 'info'
            });
            // Should NOT have refetched nodes via get_nodes call if discover was called
            // But wait, the code calls get().fetch_nodes() if discovered.length > 0
            expect(tadpole_os_service.get_nodes).not.toHaveBeenCalled();
        });

        it('handles failure during discovery', async () => {
            vi.mocked(tadpole_os_service.discover_nodes).mockRejectedValue(new Error('Scan failed'));
            const console_spy = vi.spyOn(console, 'error').mockImplementation(() => {});

            const store = use_node_store.getState();
            await store.discover_nodes();

            const updated_state = use_node_store.getState();
            expect(updated_state.is_loading).toBe(false);
            expect(console_spy).toHaveBeenCalledWith('Failed to discover nodes:', expect.any(Error));
        });
    });
});

