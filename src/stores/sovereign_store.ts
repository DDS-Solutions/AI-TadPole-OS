/**
 * @docs ARCHITECTURE:State
 *
 * ### AI Assist Note
 * - Keep side effects at service boundaries and preserve the public contracts of shared domain transitions.
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend State Store / sovereign_store
 * - **Primary Entrypoints**: `use_sovereign_store`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Store mutations maintain immutable state transitions and notify subscribers deterministically.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: `[SovereignStore]`
 */

import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { event_bus } from '../services/event_bus';
import type { Chat_Message, Sovereign_State } from '../domain/chat/types';
import { empty_transcript, message_index, reduce_transcript, type Transcript_Action } from '../domain/chat/transcript';
import { create_question_decisions } from '../services/chat/question_decisions';
import { create_chat_replication } from '../services/chat/replication';
export type { Chat_Message, Sovereign_Scope, Mission_Execution_Metrics } from '../domain/chat/types';

type Persisted_Sovereign_State = Pick<Sovereign_State,
    'active_scope' | 'selected_agent_id' | 'target_agent' | 'target_cluster' | 'is_detached'>;

export const use_sovereign_store = create<Sovereign_State>()(persist((set, get) => {
    const apply = (action: Transcript_Action) => {
        let changed = false;
        set(state => {
            const next = reduce_transcript(state, action);
            changed = next !== state;
            return next;
        });
        return changed;
    };
    const remove_message = (id: string) => { apply({ type: 'remove', id }); };
    const replication = create_chat_replication({
        get, remove_message, subscribe_logs: listener => event_bus.subscribe_logs(listener)
    });
    const resolve_user_question = create_question_decisions({
        get, remove_message: id => {
            remove_message(id);
            replication.publish({ type: 'REMOVE_MESSAGE', payload: id });
        },
        submit: async (id, decision, answer) => {
            const { system_api_service } = await import('../services/system_api_service');
            return system_api_service.oversight.decide_oversight(id, decision, answer);
        },
        report_failure: agent_id => event_bus.emit_log({
            source: 'System', severity: 'error', agent_id,
            text: '[Oversight] Failed to submit decision. Please try again.',
            metadata: { type: 'log', level: 'error', message: 'Oversight submission failed. Your answer was not recorded.' }
        })
    });
    return {
        ...empty_transcript(), active_scope: 'agent', selected_agent_id: null,
        target_agent: 'Agent of Nine', target_cluster: '', is_detached: false, latest_metrics: null,
        add_message: (msg, remote) => {
            const message: Chat_Message = {
                ...msg, id: msg.id || crypto.randomUUID(), timestamp: msg.timestamp ?? new Date().toISOString(),
                parts: msg.parts || (msg.text ? [{ type: 'text', content: msg.text }] : [])
            };
            if (apply({ type: 'add', message }) && !remote) replication.publish({ type: 'ADD_MESSAGE', payload: message });
        },
        update_message: (id, updates, remote) => {
            if (apply({ type: 'update', id, updates }) && !remote) {
                replication.publish({ type: 'UPDATE_MESSAGE', payload: get().get_message_by_id(id) });
            }
        },
        append_message_part: (id, part, remote) => {
            if (message_index(get(), id) === undefined) return false;
            if (apply({ type: 'append', id, part }) && !remote) {
                replication.publish({ type: 'UPDATE_MESSAGE', payload: get().get_message_by_id(id) });
            }
            return true;
        },
        get_message_by_id: id => {
            const index = message_index(get(), id);
            return index === undefined ? undefined : get().messages[index];
        },
        set_scope: (active_scope, is_remote_sync) => {
            if (active_scope !== 'agent' && active_scope !== 'cluster' && active_scope !== 'swarm') {
                console.error('[SovereignStore] Invalid scope payload:', active_scope);
                return;
            }
            set({ active_scope });
            if (!is_remote_sync) {
                replication.publish({ type: 'SET_SCOPE', payload: active_scope });
            }
        },

        set_selected_agent_id: (selected_agent_id, is_remote_sync) => {
            set({ selected_agent_id });
            if (!is_remote_sync) {
                replication.publish({ type: 'SET_AGENT', payload: selected_agent_id });
            }
        },

        set_target_agent: (target_agent, is_remote_sync) => {
            set({ target_agent });
            if (!is_remote_sync) {
                replication.publish({ type: 'SET_TARGET_AGENT', payload: target_agent });
            }
        },

        set_target_cluster: (target_cluster, is_remote_sync) => {
            set({ target_cluster });
            if (!is_remote_sync) {
                replication.publish({ type: 'SET_TARGET_CLUSTER', payload: target_cluster });
            }
        },

        set_detached: (is_detached, is_remote_sync) => {
            set({ is_detached });
            if (!is_remote_sync) {
                replication.publish({ type: 'SET_DETACHED', payload: is_detached });
            }
        },

        clear_history: (is_remote_sync) => {
            apply({ type: 'clear' });
            if (!is_remote_sync) {
                replication.publish({ type: 'CLEAR_HISTORY' });
            }
        },

        set_execution_metrics: (metrics, is_remote_sync) => {
            set({ latest_metrics: metrics });
            if (!is_remote_sync) {
                replication.publish({ type: 'SET_EXECUTION_METRICS', payload: metrics });
            }
        },

        resolve_user_question,
        initialize_sync_listeners: () => replication.acquire()
    };
}, {
    name: 'tadpole-sovereign-chat', version: 2,
    partialize: (state): Persisted_Sovereign_State => ({
        active_scope: state.active_scope, selected_agent_id: state.selected_agent_id,
        target_agent: state.target_agent, target_cluster: state.target_cluster, is_detached: state.is_detached
    }),
    migrate: persisted => persisted as Persisted_Sovereign_State
}));


import { register_store_for_test_reset } from '../utils/store_registry';

register_store_for_test_reset(() => {
    use_sovereign_store.setState(use_sovereign_store.getInitialState(), true);
});
