/**
 * @docs ARCHITECTURE:State
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend State Store / node_store
 * - **Primary Entrypoints**: `use_node_store`, `Node_State`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Store mutations maintain immutable state transitions and notify subscribers deterministically.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { system_api_service } from '../services/system_api_service';
import { event_bus } from '../services/event_bus';
import { log_error } from '../services/system_utils';
import type { Swarm_Node } from '../types';

/**
 * Node_State
 * Managed state for Swarm Bunker nodes.
 * Refactored for strict snake_case compliance for backend parity.
 */
export interface Node_State {
    nodes: Swarm_Node[];
    is_loading: boolean;
    error: string | null;

    fetch_nodes: (options?: RequestInit) => Promise<void>;
    discover_nodes: () => Promise<void>;
}

/**
 * use_node_store
 * Global store for managing physical and virtual infrastructure nodes.
 */
export const use_node_store = create<Node_State>()(
    persist(
        (set, get) => ({
            nodes: [],
            is_loading: false,
            error: null,

            fetch_nodes: async (options: RequestInit = {}) => {
                set({ is_loading: true, error: null });
                try {
                    const nodes = await system_api_service.infra.get_nodes({
                        signal: options.signal || undefined
                    }) as unknown as Swarm_Node[];
                    set({ nodes, is_loading: false, error: null });
                } catch (error: unknown) {
                    const message = error instanceof Error ? error.message : String(error);
                    set({ is_loading: false, error: message });
                    log_error('NodeStore', 'Node Retrieval Failed', error);
                }
            },

            discover_nodes: async () => {
                set({ is_loading: true, error: null });
                try {
                    const data = await system_api_service.infra.discover_nodes();
                    if (data.status === 'success' && data.discovered && data.discovered.length > 0) {
                        event_bus.emit_log({
                            source: 'System',
                            text: `📡 Network Scan: ${data.discovered.length} new node(s) identified.`,
                            severity: 'success'
                        });
                        await get().fetch_nodes();
                    } else {
                        event_bus.emit_log({
                            source: 'System',
                            text: `📡 Network Scan: No new nodes found.`,
                            severity: 'info'
                        });
                    }
                    set({ is_loading: false, error: null });
                } catch (error: unknown) {
                    const message = error instanceof Error ? error.message : String(error);
                    set({ is_loading: false, error: message });
                    log_error('NodeStore', 'Node Discovery Failed', error);
                }
            }
        }),
        {
            name: 'tadpole-nodes-v2',
            partialize: (state) => ({
                nodes: state.nodes
            })
        }
    )
);
