/**
 * @docs ARCHITECTURE:Logic
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend React Hooks / use_chat_dispatch
 * - **Primary Entrypoints**: `useChatDispatch`, `ChatDispatchState`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` React hook lifecycle adheres to Rules of Hooks without conditional execution branches.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: `[DISPATCH]`
 * - **Witness Tests**: none declared
 */

import { useState, useCallback } from 'react';
import { use_settings_store } from '../stores/settings_store';
import { use_sovereign_store, type Sovereign_Scope, type Chat_Message } from '../stores/sovereign_store';
import { process_command } from '../logic/command_processor';
import type { Agent } from '../types';
import { i18n } from '../i18n';

const TELEMETRY_SOURCE = '[useChatDispatch]';

export interface ChatDispatchState {
    input_text: string;
    set_input_text: (text: string) => void;
    handle_send: () => Promise<void>;
    toggle_safety: () => void;
    is_safe_mode: boolean;
}

/**
 * useChatDispatch
 * Encapsulates the command dispatch pipeline:
 * 1. Message injection into sovereign_store
 * 2. Freshest-state sync from stores (prevents ghost-targeting)
 * 3. Delegation to process_command with safe_mode context
 * 4. Error handling with system alert injection
 */
export function useChatDispatch(
    active_scope: Sovereign_Scope,
    target_node: string,
    agents: Agent[],
    selected_agent_id: string | null,
    add_message: (msg: Omit<Chat_Message, 'id' | 'timestamp'>) => void
): ChatDispatchState {
    const [input_text, set_input_text] = useState('');
    const { settings, update_setting } = use_settings_store();
    const is_safe_mode = settings.is_safe_mode;

    const handle_send = useCallback(async () => {
        const text = input_text;
        if (!text || !text.trim()) return;

        const user_msg = {
            sender_id: '0',
            sender_name: i18n.t('chat.overlord_name'),
            text: text,
            scope: active_scope,
            target_node: active_scope !== 'swarm' ? target_node : undefined
        };

        add_message(user_msg);
        set_input_text(''); // Clear the box immediately for visual feedback

        try {
            console.debug(`${TELEMETRY_SOURCE} Intent captured: ${text.substring(0, 50)}...`);

            // OVERLORD SYNC: Pull freshest state directly from stores to prevent ghost-targeting stale nodes
            const sovereign_state = use_sovereign_store.getState();
            const current_scope = sovereign_state.active_scope;
            const current_target_agent = sovereign_state.target_agent;
            const current_target_cluster = sovereign_state.target_cluster;

            // Prioritize the role of the selected agent if available
            let fresh_target = current_scope === 'cluster' ? current_target_cluster : current_target_agent;
            if (current_scope === 'agent' && selected_agent_id) {
                const agent = agents.find(a => a.id === selected_agent_id);
                if (agent) fresh_target = agent.name;
            }

            const current_safe_mode = use_settings_store.getState().settings.is_safe_mode;
            console.debug(`${TELEMETRY_SOURCE} [DISPATCH] Target: ${fresh_target}, Safe_Mode: ${current_safe_mode}`);

            await process_command(text, agents, current_safe_mode, current_scope, fresh_target);
        } catch (err: unknown) {
            const message = err instanceof Error ? err.message : 'Unknown command fault';
            add_message({
                sender_id: 'system',
                sender_name: i18n.t('chat.system_name'),
                text: i18n.t('chat.fault_detected', { message }),
                scope: active_scope,
            });
        }
    }, [input_text, active_scope, target_node, agents, add_message, selected_agent_id]);

    const toggle_safety = useCallback(() => {
        update_setting('is_safe_mode', !is_safe_mode);
    }, [is_safe_mode, update_setting]);

    return {
        input_text,
        set_input_text,
        handle_send,
        toggle_safety,
        is_safe_mode,
    };
}
