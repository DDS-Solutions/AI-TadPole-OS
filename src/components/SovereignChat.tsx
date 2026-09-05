/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / General / SovereignChat
 * - **Primary Entrypoints**: `SovereignChat`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import React, { useState, useEffect, useMemo, useCallback } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Zap } from 'lucide-react';
import clsx from 'clsx';
import { use_sovereign_store } from '../stores/sovereign_store';
import { use_agent_store } from '../stores/agent_store';
import { use_workspace_store } from '../stores/workspace_store';
import { useShallow } from 'zustand/react/shallow';
import { agent_service } from '../services/agent_service';
import { useDragControls } from 'framer-motion';
import { useChatWindow } from '../hooks/use_chat_window';
import { useChatVoice } from '../hooks/use_chat_voice';
import { useChatDispatch } from '../hooks/use_chat_dispatch';
import { Chat_Content } from './chat/Chat_Content';
import { Chat_Detached_Portal } from './chat/Chat_Detached_Portal';
import { i18n } from '../i18n';

/**
 * SovereignChat
 * A high-performance, detached-capable chat interface for agent orchestration.
 * Supports triple-scope communication: Agent, Cluster, and Swarm.
 * Enhanced with voice input and context isolation.
 */
interface SovereignChatProps {
    isDetachedView?: boolean;
}

export const SovereignChat: React.FC<SovereignChatProps> = ({ isDetachedView }) => {
    const MAX_RENDERED_MESSAGES = 300;

    // ── Store Subscriptions (Optimized via useShallow) ──────────────────────────────
    const {
        messages,
        active_scope,
        selected_agent_id,
        target_agent,
        target_cluster,
        is_detached,
        set_detached,
        set_scope,
        add_message,
        clear_history,
        set_selected_agent_id,
        set_target_agent,
        set_target_cluster,
    } = use_sovereign_store(
        useShallow(s => ({
            messages: s.messages,
            active_scope: s.active_scope,
            selected_agent_id: s.selected_agent_id,
            target_agent: s.target_agent,
            target_cluster: s.target_cluster,
            is_detached: s.is_detached,
            set_detached: s.set_detached,
            set_scope: s.set_scope,
            add_message: s.add_message,
            clear_history: s.clear_history,
            set_selected_agent_id: s.set_selected_agent_id,
            set_target_agent: s.set_target_agent,
            set_target_cluster: s.set_target_cluster,
        }))
    );

    const target_node = active_scope === 'cluster' ? target_cluster : target_agent;

    const agents = use_agent_store(s => s.agents);
    const clusters = use_workspace_store(s => s.clusters);

    // ── Local UI State ───────────────────────────────────
    const [popup_blocked, set_popup_blocked] = useState(false);
    const [open_dropdown, set_open_dropdown] = useState<'agent' | 'cluster' | null>(null);
    const [show_transcript, set_show_transcript] = useState(false);
    const drag_controls = useDragControls();

    // ── Hooks ────────────────────────────────────────────
    const {
        is_minimized,
        constraints_ref,
        x_open,
        y_open,
        x_min,
        y_min,
        toggle_detach,
        perform_minimize_transform,
        perform_maximize_transform
    } = useChatWindow();

    const {
        voice_status,
        is_speech_enabled,
        is_speaking,
        toggle_voice,
        toggle_speech,
    } = useChatVoice(messages, selected_agent_id, agents);

    const {
        input_text,
        set_input_text,
        handle_send,
        toggle_safety,
        is_safe_mode,
    } = useChatDispatch(active_scope, target_node, agents, selected_agent_id, add_message);

    // ── Agent Sorting ────────────────────────────────────
    const sorted_agents = useMemo(() => {
        const get_score = (status: string) => {
            if (['active', 'thinking', 'coding'].includes(status)) return 0;
            if (status === 'idle') return 1;
            return 2;
        };
        return [...agents].sort((a, b) => {
            const score_a = get_score(a.status || 'offline');
            const score_b = get_score(b.status || 'offline');
            if (score_a !== score_b) return score_a - score_b;
            return a.name.localeCompare(b.name);
        });
    }, [agents]);

    // ── Auto-Selection Effects ───────────────────────────

    // Conservative auto-selection: only if absolutely no target is set and agents exist
    useEffect(() => {
        const is_ungetTarget = !target_agent || target_agent.toLowerCase() === 'ceo';
        if (agents.length > 0 && !selected_agent_id && is_ungetTarget) {
            const ceo = agents.find(a => a.role?.toLowerCase().includes('ceo') || a.name.toLowerCase().includes('nine'));
            if (ceo) {
                set_target_agent(ceo.name);
                set_selected_agent_id(ceo.id);
            } else {
                set_target_agent(agents[0].name);
                set_selected_agent_id(agents[0].id);
            }
        }
    }, [agents, selected_agent_id, target_agent, set_target_agent, set_selected_agent_id]);

    // Auto-select first cluster if none selected
    useEffect(() => {
        if (clusters.length > 0 && !target_node) {
            set_target_cluster(clusters[0].name);
        }
    }, [clusters, target_node, set_target_cluster]);

    // Lazy-load agents if store is empty
    useEffect(() => {
        if (agents.length === 0) {
            void agent_service.load_agents_into_store();
        }
    }, [agents.length]);

    // ── Message Filtering ────────────────────────────────
    const filtered_messages = useMemo(() => messages.filter(m => {
        if (active_scope === 'swarm') return true;

        if (active_scope === 'agent') {
            const target = (target_agent ?? '').toLowerCase();
            const sender_lower = (m.sender_name ?? '').toLowerCase();
            return (
                m.sender_id === '0' ||
                m.sender_id === selected_agent_id ||
                m.agent_id === selected_agent_id ||
                (target && (sender_lower.includes(target) || target.includes(sender_lower))) ||
                ((target.includes('nine') || target.includes('ceo') || !selected_agent_id) &&
                    (sender_lower.includes('nine') || sender_lower.includes('ceo') || m.sender_id === '1' || m.agent_id === '1')) ||
                m.scope === 'swarm'
            );
        }

        if (active_scope === 'cluster') {
            return m.sender_id === '0' || m.target_node === target_node || m.scope === 'swarm' || m.scope === 'cluster';
        }

        return true;
    }), [messages, active_scope, selected_agent_id, target_agent, target_node]);

    // ── Header Interaction ───────────────────────────────
    const handle_header_click = useCallback(() => {
        if (is_minimized) perform_maximize_transform();
        else perform_minimize_transform();
    }, [is_minimized, perform_maximize_transform, perform_minimize_transform]);

    // ── Shared Content Props ─────────────────────────────
    const content_props = {
        active_scope,
        target_node,
        target_agent,
        target_cluster,
        selected_agent_id,
        is_speaking,
        voice_status,
        show_transcript,
        set_show_transcript,
        messages: filtered_messages,
        max_rendered_messages: MAX_RENDERED_MESSAGES,
        input_text,
        set_input_text,
        on_send: handle_send,
        on_toggle_voice: toggle_voice,
        on_toggle_speech: toggle_speech,
        is_speech_enabled,
        on_toggle_safety: toggle_safety,
        is_safe_mode,
        on_toggle_detach: toggle_detach,
        on_clear_history: clear_history,
        on_set_scope: set_scope,
        open_dropdown,
        set_open_dropdown,
        sorted_agents,
        set_target_agent,
        set_selected_agent_id,
        set_target_cluster,
        clusters,
        on_minimize: perform_minimize_transform,
    };

    // ── Detached Portal Render Path ──────────────────────
    if (is_detached && !isDetachedView) {
        return (
            <Chat_Detached_Portal
                active_scope={active_scope}
                popup_blocked={popup_blocked}
                on_restore={() => set_detached(false)}
                on_popup_block={() => set_popup_blocked(true)}
                content_props={content_props}
            />
        );
    }

    // ── Inline Render Path ───────────────────────────────
    return (
        <>
            {!is_detached && (
                <div ref={constraints_ref} className="fixed inset-x-0 inset-y-0 z-[100] pointer-events-none" style={{ padding: '24px' }} />
            )}
            <AnimatePresence>
                {!is_minimized && (
                    <motion.div
                        key="open-chat"
                        style={{ x: x_open, y: y_open }}
                        initial={{ opacity: 0, scale: 0.9, filter: 'blur(10px)' }}
                        animate={{ opacity: 1, scale: 1, filter: 'blur(0px)' }}
                        exit={{ opacity: 0, scale: 0.9, filter: 'blur(10px)' }}
                        drag={!is_detached}
                        dragControls={drag_controls}
                        dragListener={false}
                        dragMomentum={false}
                        dragElastic={0}
                        dragConstraints={is_detached ? undefined : constraints_ref}
                        className={clsx(
                            "fixed z-50 flex flex-col overflow-hidden transition-[filter,opacity] duration-300 pointer-events-auto",
                            "bottom-6 right-6 w-[440px] h-[600px] rounded-2xl border border-[color:var(--color-border)]/50 shadow-[0_30px_60px_-15px_rgba(0,0,0,0.7)] bg-[color:var(--color-surface)]/40 backdrop-blur-xl pointer-events-auto"
                        )}
                    >
                        <Chat_Content
                            {...content_props}
                            is_detached={false}
                            drag_controls={drag_controls}
                            on_header_click={handle_header_click}
                        />
                    </motion.div>
                )}
            </AnimatePresence>

            <AnimatePresence>
                {is_minimized && (
                    <motion.button
                        style={{ x: x_min, y: y_min }}
                        initial={{ scale: 0.8, opacity: 0 }}
                        animate={{ scale: 1, opacity: 1 }}
                        exit={{ scale: 0.8, opacity: 0 }}
                        drag
                        dragConstraints={constraints_ref}
                        dragMomentum={false}
                        dragElastic={0}
                        whileDrag={{ scale: 1.05 }}
                        onClick={() => {
                            perform_maximize_transform();
                        }}
                        className="fixed bottom-6 right-6 z-50 bg-zinc-100 text-zinc-950 px-5 py-3 rounded-2xl shadow-[0_10px_40px_-10px_rgba(255,255,255,0.3)] flex items-center gap-3 group border border-white cursor-grab active:cursor-grabbing"
                    >
                        <Zap size={20} className="group-hover:animate-pulse pointer-events-none" />
                        <span className="text-xs font-bold uppercase tracking-widest pointer-events-none">{i18n.t('chat.title')}</span>
                    </motion.button>
                )}
            </AnimatePresence>
        </>
    );
};
