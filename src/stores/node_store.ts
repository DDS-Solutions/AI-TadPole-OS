import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { tadpole_os_service } from '../services/tadpoleos_service';
import { event_bus } from '../services/event_bus';
import type { Swarm_Node } from '../types';

/**
 * Node_State
 * Managed state for Swarm Bunker nodes.
 * Refactored for strict snake_case compliance for backend parity.
 */
interface Node_State {
    nodes: Swarm_Node[];
    is_loading: boolean;

    fetch_nodes: () => Promise<void>;
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

            fetch_nodes: async () => {
                set({ is_loading: true });
                try {
                    const nodes = await tadpole_os_service.get_nodes() as unknown as Swarm_Node[];
                    set({ nodes, is_loading: false });
                } catch (error) {
                    console.error('Failed to fetch nodes:', error);
                    set({ is_loading: false });
                }
            },

            discover_nodes: async () => {
                set({ is_loading: true });
                try {
                    const data = await tadpole_os_service.discover_nodes();
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
                    set({ is_loading: false });
                } catch (error) {
                    console.error('Failed to discover nodes:', error);
                    set({ is_loading: false });
                }
            }
        }),
        {
            name: 'tadpole-nodes-v2' // Incremented version
        }
    )
);
