import { create } from 'zustand';
import { tadpole_os_service } from '../services/tadpoleos_service';
import { event_bus } from '../services/event_bus';
import type { Agent_Memory_Entry } from '../services/agent_api_service';

export type MemoryEntry = Agent_Memory_Entry;

interface MemoryState {
    memories: MemoryEntry[];
    isLoading: boolean;
    error: string | null;

    fetchMemories: (agent_id: string) => Promise<void>;
    searchMemories: (query: string) => Promise<MemoryEntry[]>;
    deleteMemory: (agent_id: string, rowId: string) => Promise<boolean>;
    saveMemory: (agent_id: string, text: string) => Promise<void>;
    clear: () => void;
}

export const use_memory_store = create<MemoryState>((set) => ({
    memories: [],
    isLoading: false,
    error: null,

    fetchMemories: async (agent_id: string) => {
        set({ isLoading: true, error: null });
        try {
            const data = await tadpole_os_service.get_agent_memory(agent_id);
            set({ memories: data.entries, isLoading: false });
        } catch (err: unknown) {
            console.error('[MemoryStore] Failed to fetch memory', err);
            set({ error: (err as Error).message || 'Failed to fetch memory', isLoading: false });
        }
    },

    saveMemory: async (agent_id: string, text: string): Promise<void> => {
        set({ isLoading: true, error: null });
        try {
            const res = await tadpole_os_service.save_agent_memory(agent_id, text);
            
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

            event_bus.emit_log({
                source: 'System',
                text: '🧠 Memory persisted to long-term storage.',
                severity: 'success'
            });
        } catch (err: unknown) {
            console.error('[MemoryStore] Failed to save memory', err);
            set({ error: (err as Error).message || 'Failed to save memory', isLoading: false });
            event_bus.emit_log({
                source: 'System',
                text: `⚠️ Memory Save Failed: ${(err as Error).message}`,
                severity: 'error'
            });
        }
    },

    searchMemories: async (query: string): Promise<MemoryEntry[]> => {
        const results = await tadpole_os_service.search_memory(query);
        return results.entries ?? [];
    },

    deleteMemory: async (agent_id: string, rowId: string) => {
        try {
            await tadpole_os_service.delete_agent_memory(agent_id, rowId);
            // Optimistic update
            set(state => ({
                memories: state.memories.filter(m => m.id !== rowId)
            }));
            event_bus.emit_log({
                source: 'System',
                text: '🧠 Memory pruned successfully.',
                severity: 'success'
            });
            return true;
        } catch (err: unknown) {
            console.error('[MemoryStore] Failed to delete memory', err);
            event_bus.emit_log({
                source: 'System',
                text: `⚠️ Memory Deletion Failed: ${(err as Error).message}`,
                severity: 'error'
            });
            return false;
        }
    },

    clear: () => set({ memories: [], error: null })
}));

