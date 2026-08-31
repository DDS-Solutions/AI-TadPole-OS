/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend State Store / node_store.test
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
import { use_node_store } from './node_store';
import { system_api_service } from '../services/system_api_service';
import { event_bus } from '../services/event_bus';
import { log_error } from '../services/system_utils';

// Mock dependencies
vi.mock('../services/system_api_service', () => ({
    system_api_service: {
        infra: {
            get_nodes: vi.fn(),
            discover_nodes: vi.fn(),
        }
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
            vi.mocked(system_api_service.infra.get_nodes).mockResolvedValue(mock_nodes as any);

            const store = use_node_store.getState();
            
            // Trigger fetch and verify loading state intermediate
            const fetch_promise = store.fetch_nodes();
            expect(use_node_store.getState().is_loading).toBe(true);
            
            await fetch_promise;

            const updated_state = use_node_store.getState();
            expect(updated_state.is_loading).toBe(false);
            expect(updated_state.nodes).toEqual(mock_nodes);
            expect(system_api_service.infra.get_nodes).toHaveBeenCalled();
        });

        it('handles failure during fetch_nodes', async () => {
            vi.mocked(system_api_service.infra.get_nodes).mockRejectedValue(new Error('Network error'));

            const store = use_node_store.getState();
            await store.fetch_nodes();

            const updated_state = use_node_store.getState();
            expect(updated_state.is_loading).toBe(false);
            expect(log_error).toHaveBeenCalledWith('NodeStore', 'Node Retrieval Failed', expect.any(Error));
        });
    });

    describe('discover_nodes', () => {
        it('discovers new nodes, emits event, and refetches', async () => {
            const mock_discover_result = { status: 'success', discovered: ['http://localhost:8001'] };
            vi.mocked(system_api_service.infra.discover_nodes).mockResolvedValue(mock_discover_result);
            // Mock subsequent fetch_nodes call
            vi.mocked(system_api_service.infra.get_nodes).mockResolvedValue([{ id: 'node2', url: 'http://localhost:8001' } as any]);

            const store = use_node_store.getState();
            await store.discover_nodes();

            const updated_state = use_node_store.getState();
            expect(updated_state.is_loading).toBe(false);
            expect(system_api_service.infra.discover_nodes).toHaveBeenCalled();
            expect(event_bus.emit_log).toHaveBeenCalledWith({
                source: 'System',
                text: '📡 Network Scan: 1 new node(s) identified.',
                severity: 'success'
            });
            // Should have refetched nodes
            expect(system_api_service.infra.get_nodes).toHaveBeenCalled();
        });

        it('handles discovery when no nodes are found', async () => {
            const mock_discover_result = { status: 'success', discovered: [] };
            vi.mocked(system_api_service.infra.discover_nodes).mockResolvedValue(mock_discover_result);

            const store = use_node_store.getState();
            await store.discover_nodes();

            const updated_state = use_node_store.getState();
            expect(updated_state.is_loading).toBe(false);
            expect(event_bus.emit_log).toHaveBeenCalledWith({
                source: 'System',
                text: '📡 Network Scan: No new nodes found.',
                severity: 'info'
            });
            expect(system_api_service.infra.get_nodes).not.toHaveBeenCalled();
        });

        it('handles failure during discovery', async () => {
            vi.mocked(system_api_service.infra.discover_nodes).mockRejectedValue(new Error('Scan failed'));

            const store = use_node_store.getState();
            await store.discover_nodes();

            const updated_state = use_node_store.getState();
            expect(updated_state.is_loading).toBe(false);
            expect(log_error).toHaveBeenCalledWith('NodeStore', 'Node Discovery Failed', expect.any(Error));
        });
    });
});
