import { create } from 'zustand';
import { TadpoleOSService } from '../services/tadpoleosService';
import { EventBus } from '../services/eventBus';
import type { AgentMemoryEntry } from '../services/AgentApiService';

export type MemoryEntry = AgentMemoryEntry;

interface MemoryState {
    memories: MemoryEntry[];
    isLoading: boolean;
    error: string | null;

    fetchMemories: (agentId: string) => Promise<void>;
    searchMemories: (query: string) => Promise<MemoryEntry[]>;
    deleteMemory: (agentId: string, rowId: string) => Promise<boolean>;
    saveMemory: (agentId: string, text: string) => Promise<void>;
    clear: () => void;
}

export const useMemoryStore = create<MemoryState>((set) => ({
    memories: [],
    isLoading: false,
    error: null,

    fetchMemories: async (agentId: string) => {
        set({ isLoading: true, error: null });
        try {
            const data = await TadpoleOSService.getAgentMemory(agentId);
            set({ memories: data.entries, isLoading: false });
        } catch (err: unknown) {
            console.error('[MemoryStore] Failed to fetch memory', err);
            set({ error: (err as Error).message || 'Failed to fetch memory', isLoading: false });
        }
    },

    saveMemory: async (agentId: string, text: string): Promise<void> => {
        set({ isLoading: true, error: null });
        try {
            const res = await TadpoleOSService.saveAgentMemory(agentId, text);
            
            // Optimistic update (or refetch)
            const newEntry: MemoryEntry = {
                id: res.id,
                text,
                mission_id: 'manual',
                timestamp: Math.floor(Date.now() / 1000)
            };
            
            set(state => ({
                memories: [newEntry, ...state.memories],
                isLoading: false
            }));

            EventBus.emit({
                source: 'System',
                text: '🧠 Memory persisted to long-term storage.',
                severity: 'success'
            });
        } catch (err: unknown) {
            console.error('[MemoryStore] Failed to save memory', err);
            set({ error: (err as Error).message || 'Failed to save memory', isLoading: false });
            EventBus.emit({
                source: 'System',
                text: `⚠️ Memory Save Failed: ${(err as Error).message}`,
                severity: 'error'
            });
        }
    },

    searchMemories: async (query: string): Promise<MemoryEntry[]> => {
        const results = await TadpoleOSService.searchMemory(query);
        return results.entries ?? [];
    },

    deleteMemory: async (agentId: string, rowId: string) => {
        try {
            await TadpoleOSService.deleteAgentMemory(agentId, rowId);
            // Optimistic update
            set(state => ({
                memories: state.memories.filter(m => m.id !== rowId)
            }));
            EventBus.emit({
                source: 'System',
                text: '🧠 Memory pruned successfully.',
                severity: 'success'
            });
            return true;
        } catch (err: unknown) {
            console.error('[MemoryStore] Failed to delete memory', err);
            EventBus.emit({
                source: 'System',
                text: `⚠️ Memory Deletion Failed: ${(err as Error).message}`,
                severity: 'error'
            });
            return false;
        }
    },

    clear: () => set({ memories: [], error: null })
}));

