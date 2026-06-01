/**
 * @docs ARCHITECTURE:Stores
 * 
 * ### AI Assist Note
 * **Core functional element for the Tadpole OS engine.**
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path: Runtime logic error or state corruption.**
 * - **Telemetry Link**: Search `[memory_store.ts]` in tracing logs.
 */

/**
 * @docs ARCHITECTURE:State
 * 
 * ### AI Assist Note
 * **Zustand State**: Long-term memory persistence and retrieval.
 * Orchestrates memory fetch, save, search, and pruning with the Tadpole Engine.
 */

import { create } from 'zustand';
import { tadpole_os_service } from '../services/tadpoleos_service';
import { event_bus } from '../services/event_bus';
import { log_error } from '../services/system_utils';
import type { Agent_Memory_Entry } from '../contracts/agent';

export type MemoryEntry = Agent_Memory_Entry;

interface Memory_State {
    memories: MemoryEntry[];
    is_loading: boolean;
    error: string | null;

    fetch_memories: (agent_id: string) => Promise<void>;
    search_memories: (query: string) => Promise<MemoryEntry[]>;
    delete_memory: (agent_id: string, row_id: string) => Promise<boolean>;
    save_memory: (agent_id: string, text: string) => Promise<void>;
    clear: () => void;
}

/**
 * use_memory_store
 * Global configuration store for agent memory persistence.
 * Refactored for strict snake_case compliance for backend parity.
 * Pattern: PersistentMemory (Tadpole Engine Sync)
 */
export const use_memory_store = create<Memory_State>((set) => ({
    memories: [],
    is_loading: false,
    error: null,

    fetch_memories: async (agent_id: string) => {
        set({ is_loading: true, error: null });
        try {
            const data = await tadpole_os_service.get_agent_memory(agent_id);
            set({ memories: data.entries, is_loading: false });
        } catch (err: unknown) {
            set({ is_loading: false });
            log_error('MemoryStore', 'Memory Retrieval Failed', err);
            const message = err instanceof Error ? err.message : String(err);
            set({ error: message });
        }
    },

    save_memory: async (agent_id: string, text: string): Promise<void> => {
        set({ is_loading: true, error: null });
        try {
            const res = await tadpole_os_service.save_agent_memory(agent_id, text);
            
            // Optimistic update (or refetch)
            const new_entry: MemoryEntry = {
                id: res.id,
                text,
                mission_id: 'manual',
                timestamp: Math.floor(Date.now() / 1000)
            };
            
            set(state => ({
                memories: [new_entry, ...state.memories],
                is_loading: false
            }));

            event_bus.emit_log({
                source: 'System',
                text: '🧠 Memory persisted to long-term storage.',
                severity: 'success'
            });
        } catch (err: unknown) {
            set({ is_loading: false });
            log_error('MemoryStore', 'Memory Save Failed', err);
            const message = err instanceof Error ? err.message : String(err);
            set({ error: message });
        }
    },

    search_memories: async (query: string): Promise<MemoryEntry[]> => {
        const results = await tadpole_os_service.search_memory(query);
        return results.entries ?? [];
    },

    delete_memory: async (agent_id: string, row_id: string) => {
        try {
            await tadpole_os_service.delete_agent_memory(agent_id, row_id);
            // Optimistic update
            set(state => ({
                memories: state.memories.filter(m => m.id !== row_id)
            }));
            event_bus.emit_log({
                source: 'System',
                text: '🧠 Memory pruned successfully.',
                severity: 'success'
            });
            return true;
        } catch (err: unknown) {
            log_error('MemoryStore', 'Memory Deletion Failed', err);
            return false;
        }
    },

    clear: () => set({ memories: [], error: null })
}));

// Metadata: [memory_store]

// Metadata: [memory_store]
