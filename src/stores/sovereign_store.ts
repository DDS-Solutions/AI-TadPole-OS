/**
 * @docs ARCHITECTURE:State
 * 
 * ### AI Assist Note
 * **Zustand State**: High-level swarm consciousness and sovereign decision-making logs. 
 * Orchestrates the persistent feedback loop between the human operator (Sovereign) and the autonomous swarm brain.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Chat history buffer overflow, or message delivery failure during heavy backend telemetry pulses.
 * - **Telemetry Link**: Search for `[SovereignStore]` or MESSAGE_SYNC in UI logs.
 * 
 * ```mermaid
 * stateDiagram-v2
 *     [*] --> Initializing
 *     Initializing --> Persistence: getItem(tadpole-sovereign-chat)
 *     Persistence --> Migration: migrate(v1 -> v2)
 *     Migration --> Ready: set({messages, index})
 *     Ready --> Message_Flow: add_message(msg)
 *     Message_Flow --> Deduplication: check message_index_by_id
 *     Deduplication --> State_Update: push to array + update index
 *     State_Update --> Sync: BroadcastChannel (ADD_MESSAGE)
 *     Ready --> Part_Append: append_message_part(id, part)
 *     Part_Append --> State_Update
 *     Ready --> Scope_Change: set_scope(scope)
 *     Scope_Change --> Sync: BroadcastChannel (SET_SCOPE)
 *     Ready --> History_Clear: clear_history()
 *     History_Clear --> Sync: BroadcastChannel (CLEAR_HISTORY)
 * ```
 */


import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { type Message_Part } from '../types';
import { event_bus } from '../services/event_bus';

export type Sovereign_Scope = 'agent' | 'cluster' | 'swarm';

export interface Chat_Message {
    id: string;
    sender_id: string;
    sender_name: string;
    text: string;
    parts?: Message_Part[];
    timestamp: string | number | Date;
    scope: Sovereign_Scope;
    agent_id?: string;
    is_sub_agent?: boolean;
    lineage?: string[];
    target_node?: string;
}

interface Sovereign_State {
    messages: Chat_Message[];
    message_index_by_id: Record<string, number>;
    active_scope: Sovereign_Scope;
    selected_agent_id: string | null;
    target_agent: string;
    target_cluster: string;
    is_detached: boolean;

    // Actions
    add_message: (msg: Omit<Chat_Message, 'id' | 'timestamp'> & { id?: string, timestamp?: Chat_Message['timestamp'] }, is_remote_sync?: boolean) => void;
    update_message: (id: string, updates: Partial<Chat_Message>, is_remote_sync?: boolean) => void;
    append_message_part: (id: string, part: Message_Part, is_remote_sync?: boolean) => boolean;
    get_message_by_id: (id: string) => Chat_Message | undefined;
    set_scope: (scope: Sovereign_Scope, is_remote_sync?: boolean) => void;
    set_selected_agent_id: (agent_id: string | null, is_remote_sync?: boolean) => void;
    set_target_agent: (name: string, is_remote_sync?: boolean) => void;
    set_target_cluster: (name: string, is_remote_sync?: boolean) => void;
    set_detached: (is_detached: boolean, is_remote_sync?: boolean) => void;
    clear_history: (is_remote_sync?: boolean) => void;
    initialize_sync_listeners: () => () => void;
}

type Persisted_Sovereign_State = Pick<
    Sovereign_State,
    'active_scope' | 'selected_agent_id' | 'target_agent' | 'target_cluster' | 'is_detached'
>;

// Cross-window synchronization
const chat_channel = typeof window !== 'undefined' ? new BroadcastChannel('tadpole-chat-sync') : null;

export const use_sovereign_store = create<Sovereign_State>()(
    persist(
        (set, get) => ({
            messages: [],
            message_index_by_id: {},
            active_scope: 'agent',
            selected_agent_id: null,
            target_agent: 'Agent of Nine',
            target_cluster: '',
            is_detached: false,

            add_message: (msg, is_remote_sync) => {
                const new_msg = {
                    ...msg,
                    id: msg.id || ((typeof crypto !== 'undefined' && crypto.randomUUID) ? crypto.randomUUID() : Date.now().toString(36)),
                    timestamp: msg.timestamp || new Date().toISOString(),
                    parts: msg.parts || (msg.text ? [{ type: 'text', content: msg.text }] : [])
                } as Chat_Message;
                let was_added = false;
                set((state) => {
                    if (state.message_index_by_id[new_msg.id] !== undefined) {
                        return state; // Deduplicate
                    }
                    was_added = true;

                    const next_messages = [...state.messages, new_msg];
                    // Hard cap for in-memory messages at 5000 (FIFO)
                    const capped_messages = next_messages.slice(-5000);

                    const new_index_map: Record<string, number> = {};
                    capped_messages.forEach((m, idx) => {
                        new_index_map[m.id] = idx;
                    });

                    return {
                        messages: capped_messages,
                        message_index_by_id: new_index_map
                    };
                });
                // Sync to other windows
                if (was_added && !is_remote_sync) {
                    chat_channel?.postMessage({ type: 'ADD_MESSAGE', payload: new_msg });
                }
            },

            update_message: (id, updates, is_remote_sync) => {
                let updated_msg: Chat_Message | undefined;
                set((state) => {
                    const idx = state.message_index_by_id[id];
                    if (idx === undefined) return state;

                    const next_messages = [...state.messages];
                    updated_msg = { ...next_messages[idx], ...updates };
                    next_messages[idx] = updated_msg;
                    return { messages: next_messages };
                });
                if (updated_msg && !is_remote_sync) {
                    chat_channel?.postMessage({ type: 'UPDATE_MESSAGE', payload: updated_msg });
                }
            },

            append_message_part: (id, part, is_remote_sync) => {
                let updated_msg: Chat_Message | undefined;
                set((state) => {
                    const idx = state.message_index_by_id[id];
                    if (idx === undefined) return state;

                    const current_message = state.messages[idx];
                    updated_msg = {
                        ...current_message,
                        parts: [...(current_message.parts || []), part]
                    };

                    const next_messages = [...state.messages];
                    next_messages[idx] = updated_msg;
                    return { messages: next_messages };
                });

                if (updated_msg) {
                    if (!is_remote_sync) {
                        chat_channel?.postMessage({ type: 'UPDATE_MESSAGE', payload: updated_msg });
                    }
                    return true;
                }
                return false;
            },

            get_message_by_id: (id) => {
                const state = get();
                const idx = state.message_index_by_id[id];
                return idx === undefined ? undefined : state.messages[idx];
            },

            set_scope: (active_scope, is_remote_sync) => {
                if (active_scope !== 'agent' && active_scope !== 'cluster' && active_scope !== 'swarm') {
                    console.error('[SovereignStore] Invalid scope payload:', active_scope);
                    return;
                }
                set({ active_scope });
                if (!is_remote_sync) {
                    chat_channel?.postMessage({ type: 'SET_SCOPE', payload: active_scope });
                }
            },

            set_selected_agent_id: (selected_agent_id, is_remote_sync) => {
                set({ selected_agent_id });
                if (!is_remote_sync) {
                    chat_channel?.postMessage({ type: 'SET_AGENT', payload: selected_agent_id });
                }
            },

            set_target_agent: (target_agent, is_remote_sync) => {
                set({ target_agent });
                if (!is_remote_sync) {
                    chat_channel?.postMessage({ type: 'SET_TARGET_AGENT', payload: target_agent });
                }
            },

            set_target_cluster: (target_cluster, is_remote_sync) => {
                set({ target_cluster });
                if (!is_remote_sync) {
                    chat_channel?.postMessage({ type: 'SET_TARGET_CLUSTER', payload: target_cluster });
                }
            },

            set_detached: (is_detached, is_remote_sync) => {
                set({ is_detached });
                if (!is_remote_sync) {
                    chat_channel?.postMessage({ type: 'SET_DETACHED', payload: is_detached });
                }
            },

            clear_history: (is_remote_sync) => {
                set({ messages: [], message_index_by_id: {} });
                if (!is_remote_sync) {
                    chat_channel?.postMessage({ type: 'CLEAR_HISTORY' });
                }
            },

            initialize_sync_listeners: () => {
                const handle_message = (event: MessageEvent) => {
                    const { type, payload } = event.data as { type: string, payload: unknown };
                    const s = get();

                    switch (type) {
                        case 'ADD_MESSAGE':
                            {
                                const msg = payload as Chat_Message;
                                s.add_message(msg, true);
                            }
                            break;
                        case 'UPDATE_MESSAGE':
                            {
                                const msg = payload as Chat_Message;
                                s.update_message(msg.id, msg, true);
                            }
                            break;
                        case 'SET_SCOPE':
                            s.set_scope(payload as Sovereign_Scope, true);
                            break;
                        case 'SET_AGENT':
                            s.set_selected_agent_id(payload as string | null, true);
                            break;
                        case 'SET_TARGET_AGENT':
                            s.set_target_agent(payload as string, true);
                            break;
                        case 'SET_TARGET_CLUSTER':
                            s.set_target_cluster(payload as string, true);
                            break;
                        case 'SET_DETACHED':
                            s.set_detached(payload as boolean, true);
                            break;
                        case 'CLEAR_HISTORY':
                            s.clear_history(true);
                            break;
                    }
                };

                if (chat_channel) {
                    chat_channel.addEventListener('message', handle_message);
                }

                const unsubscribe_logs = event_bus.subscribe_logs((log) => {
                    const type = log.metadata?.type;
                    if (!log.agent_id || !type) return;

                    if (type === 'thought' || (type === 'log' && log.metadata?.level !== 'debug')) {
                        const resolved_name = log.agent_name || log.agent_id;
                        const message_id = log.id;

                        const new_part: Message_Part = type === 'thought'
                            ? { type: 'thought', content: (log.metadata?.message || log.metadata?.text || '') as string, status: 'done' }
                            : { type: 'text', content: (log.metadata?.message || log.metadata?.text || '') as string };

                        const store = get();
                        const did_update = store.append_message_part(message_id, new_part);
                        if (!did_update) {
                            store.add_message({
                                id: message_id,
                                sender_id: log.agent_id,
                                sender_name: resolved_name,
                                agent_id: log.agent_id,
                                text: (log.metadata?.message || log.metadata?.text || '') as string,
                                parts: [new_part],
                                scope: store.active_scope
                            });
                        }
                    } else if (type === 'agent:message') {
                        const resolved_name = log.agent_name || log.agent_id || 'Agent';
                        const message_id = log.id;

                         const new_part: Message_Part = {
                            type: 'text' as const,
                            content: (log.metadata?.text || log.metadata?.message || log.metadata?.content || 'Mission action complete.') as string
                        };

                        const store = get();
                        const did_update = store.append_message_part(message_id, new_part);
                        if (!did_update) {
                            store.add_message({
                                id: message_id,
                                sender_id: log.agent_id || 'system',
                                sender_name: resolved_name,
                                agent_id: log.agent_id,
                                target_node: resolved_name,
                                text: (log.metadata?.text || log.metadata?.message || log.metadata?.content || 'Mission action complete.') as string,
                                parts: [new_part],
                                scope: store.active_scope
                            });
                        }
                    }
                });

                return () => {
                    unsubscribe_logs();
                    if (chat_channel) {
                        chat_channel.removeEventListener('message', handle_message);
                    }
                };
            }
        }),
        {
            name: 'tadpole-sovereign-chat',
            version: 2,
            partialize: (state): Persisted_Sovereign_State => ({
                active_scope: state.active_scope,
                selected_agent_id: state.selected_agent_id,
                target_agent: state.target_agent,
                target_cluster: state.target_cluster,
                is_detached: state.is_detached
            }),
            migrate: (persisted_state: unknown) => {
                const state = (persisted_state || {}) as { logs?: Record<string, unknown>[] };
                const logs = state.logs || [];

                return {
                    ...state,
                    logs: (logs || []).map(l => ({
                        ...l,
                        agent_id: l.agent_id ?? l.agentId
                    }))
                } as unknown as Record<string, unknown>;
            }
        }
    )
);

// Metadata: [sovereign_store]
