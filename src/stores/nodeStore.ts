import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { TadpoleOSService } from '../services/tadpoleosService';
import { EventBus } from '../services/eventBus';
import type { SwarmNode } from '../types';

interface NodeState {
    nodes: SwarmNode[];
    isLoading: boolean;

    fetchNodes: () => Promise<void>;
    discoverNodes: () => Promise<void>;
}

export const useNodeStore = create<NodeState>()(
    persist(
        (set, get) => ({
            nodes: [],
            isLoading: false,

            fetchNodes: async () => {
                set({ isLoading: true });
                try {
                    const nodes = await TadpoleOSService.getNodes();
                    set({ nodes, isLoading: false });
                } catch (error) {
                    console.error('Failed to fetch nodes:', error);
                    set({ isLoading: false });
                }
            },

            discoverNodes: async () => {
                set({ isLoading: true });
                try {
                    const data = await TadpoleOSService.discoverNodes();
                    if (data.status === 'success' && data.discovered && data.discovered.length > 0) {
                        EventBus.emit({
                            source: 'System',
                            text: `📡 Network Scan: ${data.discovered.length} new node(s) identified.`,
                            severity: 'success'
                        });
                        await get().fetchNodes();
                    } else {
                        EventBus.emit({
                            source: 'System',
                            text: `📡 Network Scan: No new nodes found.`,
                            severity: 'info'
                        });
                    }
                    set({ isLoading: false });
                } catch (error) {
                    console.error('Failed to discover nodes:', error);
                    set({ isLoading: false });
                }
            }
        }),
        {
            name: 'tadpole-nodes'
        }
    )
);

