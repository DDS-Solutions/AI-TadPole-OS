/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Detachable Command & Control (C2) interface for Swarm Intelligence. 
 * Orchestrates triple-scope communication (Agent/Cluster/Swarm), real-time voice synthesis (Azure/Groq), and transcript buffering for autonomous agent logs.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Voice client initialization stall, portal window context loss (detachment), or message packet starvation during high-frequency telemetry storms.
 * - **Telemetry Link**: Search for `[Sovereign_Chat]` or `sovereign_store` in browser logs.
 */

import React, { useState, useRef, useEffect, useMemo } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import {
    Send,
    X,
    Maximize2,
    Minimize2,
    Bot,
    User,
    Zap,
    ExternalLink,
    Mic,
    MicOff,
    Target as TargetIcon,
    ChevronDown,
    BrainCircuit,
    GripVertical,
    Volume2,
    VolumeX,
    Activity
} from 'lucide-react';
import clsx from 'clsx';
import { use_settings_store } from '../stores/settings_store';
import { use_sovereign_store, type Message_Part, type Sovereign_Scope } from '../stores/sovereign_store';
import { use_agent_store } from '../stores/agent_store';
import { use_workspace_store } from '../stores/workspace_store';
import { process_command } from '../logic/command_processor';
import { voice_client } from '../services/voice_client';
import { useDragControls } from 'framer-motion';
import { useChatWindow } from '../hooks/use_chat_window';
import { Tooltip } from './ui';
import { i18n } from '../i18n';
import { Buffered_Transcript_View } from './transcript/Buffered_Transcript_View';

/**
 * Sovereign_Chat
 * A high-performance, detached-capable chat interface for agent orchestration.
 * Supports triple-scope communication: Agent, Cluster, and Swarm.
 * Enhanced with voice input and context isolation.
 * Refactored for strict snake_case compliance and consistent service integration.
 */
export const Sovereign_Chat: React.FC = () => {
    const MAX_RENDERED_MESSAGES = 300;
    const messages = use_sovereign_store(s => s.messages);
    const active_scope = use_sovereign_store(s => s.active_scope);
    const selected_agent_id = use_sovereign_store(s => s.selected_agent_id);
    const target_agent = use_sovereign_store(s => s.target_agent);
    const target_cluster = use_sovereign_store(s => s.target_cluster);
    const is_detached = use_sovereign_store(s => s.is_detached);
    const set_detached = use_sovereign_store(s => s.set_detached);
    const set_scope = use_sovereign_store(s => s.set_scope);
    const add_message = use_sovereign_store(s => s.add_message);
    const clear_history = use_sovereign_store(s => s.clear_history);
    const set_selected_agent_id = use_sovereign_store(s => s.set_selected_agent_id);
    const set_target_agent = use_sovereign_store(s => s.set_target_agent);
    const set_target_cluster = use_sovereign_store(s => s.set_target_cluster);

    const target_node = active_scope === 'cluster' ? target_cluster : target_agent;

    const { agents } = use_agent_store();
    const { clusters } = use_workspace_store();
    const [input_value, set_input_value] = useState('');
    const [is_listening, set_is_listening] = useState(false);
    const [is_speech_enabled, set_is_speech_enabled] = useState(false);
    const [is_speaking, set_is_speaking] = useState(false);
    const { settings, update_setting } = use_settings_store();
    const is_safe_mode = settings.is_safe_mode;
    const [open_dropdown, set_open_dropdown] = useState<'agent' | 'cluster' | null>(null);
    const [show_transcript, set_show_transcript] = useState(false);
    const scroll_ref = useRef<HTMLDivElement>(null);
    const speak_start_timeout_ref = useRef<ReturnType<typeof setTimeout> | null>(null);
    const speak_end_timeout_ref = useRef<ReturnType<typeof setTimeout> | null>(null);
    const drag_controls = useDragControls();

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
    
    /**
     * get_score
     * Prioritizes active/thinking agents for UI sorting.
     */
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

    // Sync target_agent with selected_agent_id
    useEffect(() => {
        if (selected_agent_id) {
            const agent = agents.find(a => a.id === selected_agent_id);
            if (agent) {
                set_target_agent(agent.name);
                // Only forced-switch to agent scope if we are currently in a generic/unset state
                if (!target_agent || target_agent === 'CEO') {
                    set_scope('agent');
                }
            }
        }
    }, [selected_agent_id, agents, set_scope, target_agent, set_target_agent]);

    // Conservative auto-selection: only if absolutely no target is set and agents exist
    useEffect(() => {
        if (agents.length > 0 && !selected_agent_id && (target_agent === 'CEO' || !target_agent)) {
            // Check if CEO actually exists before falling back to index 0
            const ceo = agents.find(a => a.name === 'CEO');
            if (!ceo) {
                set_target_agent(agents[0].name);
            }
        }
    }, [agents, selected_agent_id, target_agent, set_target_agent]);

    // Auto-select first cluster if none selected
    useEffect(() => {
        if (clusters.length > 0 && !target_node) {
            set_target_cluster(clusters[0].name);
        }
    }, [clusters, target_node, set_target_cluster]);

    useEffect(() => {
        if (scroll_ref.current) {
            scroll_ref.current.scrollTo({
                top: scroll_ref.current.scrollHeight,
                behavior: 'smooth'
            });
        }

        if (speak_start_timeout_ref.current) {
            clearTimeout(speak_start_timeout_ref.current);
            speak_start_timeout_ref.current = null;
        }
        if (speak_end_timeout_ref.current) {
            clearTimeout(speak_end_timeout_ref.current);
            speak_end_timeout_ref.current = null;
        }

        // AUTO-SPEAK LOGIC
        const last_message = messages[messages.length - 1];
        if (is_speech_enabled && last_message && last_message.sender_id !== '0' && last_message.sender_id === selected_agent_id) {
            // Guard: Don't auto-speak technical errors or security alerts
            if (last_message.text.startsWith('❌') || last_message.text.startsWith('🛡️') || last_message.text.includes('Error:')) {
                return;
            }

            const agent = agents.find(a => a.id === selected_agent_id);
            if (agent) {
                speak_start_timeout_ref.current = setTimeout(() => set_is_speaking(true), 0);
                voice_client.speak(last_message.text, agent.voice_id, agent.voice_engine || 'browser').finally(() => {
                    speak_end_timeout_ref.current = setTimeout(() => set_is_speaking(false), Math.min(10000, last_message.text.length * 60));
                });
            }
        }

        return () => {
            if (speak_start_timeout_ref.current) {
                clearTimeout(speak_start_timeout_ref.current);
                speak_start_timeout_ref.current = null;
            }
            if (speak_end_timeout_ref.current) {
                clearTimeout(speak_end_timeout_ref.current);
                speak_end_timeout_ref.current = null;
            }
        };
    }, [messages, is_speech_enabled, selected_agent_id, agents]);

    useEffect(() => {
        if (agents.length === 0) {
            use_agent_store.getState().fetch_agents();
        }
    }, [agents.length]);
    
    // Header interaction: Toggle between maximized and minimized states
    const handle_header_click = () => {
        if (is_minimized) perform_maximize_transform();
        else perform_minimize_transform();
    };

    const handle_send = async (text_override?: string) => {
        const text = text_override || input_value;
        if (!text.trim()) return;

        const user_msg = {
            sender_id: '0',
            sender_name: i18n.t('chat.overlord_name'),
            text: text,
            scope: active_scope,
            target_node: active_scope !== 'swarm' ? target_node : undefined
        };

        add_message(user_msg);
        if (!text_override) set_input_value('');

        try {
            // Include target context in command processing
            const prefix = active_scope === 'agent' ? '@' : active_scope === 'cluster' ? '#' : '';
            const command = active_scope === 'swarm' ? text : `"${prefix}${target_node}" ${text}`;

            await process_command(command, agents, is_safe_mode);
        } catch (err: unknown) {
            const message = err instanceof Error ? err.message : 'Unknown command fault';
            add_message({
                sender_id: 'system',
                sender_name: i18n.t('chat.system_name'),
                text: i18n.t('chat.fault_detected', { message }),
                scope: active_scope,
            });
        }
    };

    const toggle_voice = () => {
        if (is_listening) {
            voice_client.stop_listening();
            set_is_listening(false);
        } else {
            set_is_listening(true);
            set_is_speech_enabled(true);
            voice_client.start_listening((transcript) => {
                set_input_value(transcript);
            });
        }
    };

    /**
     * REACTIVE FILTERING LOGIC
     */
    const filtered_messages = useMemo(() => messages.filter(m => {
        // 1. Global Swarm Scope: Always visible
        if (active_scope === 'swarm') return true;

        // 2. Agent Isolation
        if (active_scope === 'agent') {
            const target = target_agent.toLowerCase();
            return m.scope === 'agent' && (
                m.sender_id === '0' || 
                m.sender_id === selected_agent_id || 
                m.agent_id === selected_agent_id || 
                m.sender_name.toLowerCase().includes(target) || 
                ((target.includes('nine') || target.includes('ceo')) &&
                    (m.sender_name.toLowerCase().includes('nine') || m.sender_name.toLowerCase().includes('ceo') || m.sender_id === '1'))
            );
        }

        // 3. Cluster Isolation
        if (active_scope === 'cluster') {
            return m.sender_id === '0' || m.target_node === target_node || m.scope === 'swarm';
        }

        return true;
    }), [messages, active_scope, selected_agent_id, target_agent, target_node]);

    const rendered_messages = useMemo(() => {
        if (filtered_messages.length <= MAX_RENDERED_MESSAGES) {
            return filtered_messages;
        }
        return filtered_messages.slice(filtered_messages.length - MAX_RENDERED_MESSAGES);
    }, [filtered_messages]);
    
    const hidden_message_count = filtered_messages.length - rendered_messages.length;

    if (is_detached && window.location.hash !== '#sovereign-chat') {
        return (
            <div className="fixed bottom-6 right-6 z-50">
                <Tooltip content={i18n.t('chat.restore_tooltip')} position="top">
                    <button
                        onClick={() => set_detached(false)}
                        className="bg-zinc-900/80 backdrop-blur-md border border-zinc-700/50 p-4 rounded-full text-zinc-400 hover:text-zinc-100 shadow-[0_0_20px_rgba(0,0,0,0.5)] transition-all hover:scale-110 active:scale-95 group"
                    >
                        <Maximize2 size={24} className="group-hover:rotate-12 transition-transform" />
                    </button>
                </Tooltip>
            </div>
        );
    }

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
                            is_detached
                                ? "inset-0 md:static md:w-full md:h-full bg-zinc-950 border-none pointer-events-auto"
                                : "bottom-6 right-6 w-[440px] h-[600px] rounded-2xl border border-zinc-800/50 shadow-[0_30px_60px_-15px_rgba(0,0,0,0.7)] bg-zinc-900/40 backdrop-blur-xl pointer-events-auto"
                        )}
                    >
                        {!is_detached && <div className="neural-grid opacity-[0.05] absolute inset-0 pointer-events-none" />}

                        {/* Header */}
                        <div
                            onPointerDown={(e) => {
                                if (!is_detached) {
                                    drag_controls.start(e);
                                }
                            }}
                            className={clsx(
                                "relative z-10 p-4 border-b border-zinc-800/50 bg-zinc-950/40 backdrop-blur-md flex items-center justify-between shrink-0 overflow-hidden cursor-pointer select-none",
                                !is_detached && "cursor-grab active:cursor-grabbing"
                            )}
                            onDoubleClick={handle_header_click}
                            onKeyDown={(e) => (e.key === 'Enter' || e.key === ' ') && handle_header_click()}
                            role="button"
                            tabIndex={0}
                            title={i18n.t('chat.header_drag_hint')}
                            aria-label={i18n.t('chat.header_drag_aria')}
                        >
                            <div className="flex items-center gap-3">
                                {!is_detached && <GripVertical size={14} className="text-zinc-700" />}
                                <div className="relative bg-zinc-100 p-1.5 rounded-md text-black shadow-lg">
                                    <Zap size={14} className="fill-current" />
                                </div>
                                <div>
                                    <span className="font-bold text-[11px] tracking-[0.2em] text-zinc-100 uppercase">{i18n.t('chat.title')}</span>
                                    <div className="flex items-center gap-1.5">
                                        <div className="h-1 w-1 rounded-full bg-emerald-500 animate-pulse" />
                                        <span className="text-[9px] text-zinc-500 font-mono uppercase tracking-tighter">
                                            {i18n.t(`chat.scope_${active_scope}`)} / {active_scope !== 'swarm' ? target_node : 'Sovereign_Link'}
                                        </span>
                                        {is_speaking && (
                                            <div className="flex items-center gap-0.5 ml-2 mr-1">
                                                {[1, 2, 3, 4].map(i => (
                                                    <motion.div
                                                        key={i}
                                                        animate={{ height: [4, 12, 4] }}
                                                        transition={{ repeat: Infinity, duration: 0.6, delay: i * 0.1 }}
                                                        className="w-0.5 bg-blue-500 rounded-full"
                                                    />
                                                ))}
                                            </div>
                                        )}
                                    </div>
                                </div>
                            </div>
                            <div className="flex items-center gap-1">
                                <Tooltip content={show_transcript ? i18n.t('chat.show_chat_tooltip') : i18n.t('chat.show_transcript_tooltip')} position="top">
                                    <button
                                        onClick={(e) => {
                                            e.stopPropagation();
                                            set_show_transcript(!show_transcript);
                                        }}
                                        className={clsx(
                                            "p-2 rounded-lg transition-all active:scale-95",
                                            show_transcript ? "text-blue-400 bg-blue-500/10 border border-blue-500/30 shadow-[0_0_15px_rgba(59,130,246,0.15)]" : "text-zinc-500 hover:text-zinc-200 hover:bg-zinc-800/50"
                                        )}
                                        aria-label={i18n.t('chat.toggle_transcript_aria')}
                                    >
                                        <Activity size={16} />
                                    </button>
                                </Tooltip>
                                <Tooltip content={i18n.t('chat.minimize_tooltip')} position="top">
                                    <button
                                        onClick={(e) => {
                                            e.stopPropagation();
                                            perform_minimize_transform();
                                        }}
                                        className="p-2 text-zinc-500 hover:text-zinc-200 hover:bg-zinc-800/50 rounded-lg transition-colors"
                                        aria-label={i18n.t('chat.minimize_aria')}
                                    >
                                        <Minimize2 size={16} />
                                    </button>
                                </Tooltip>
                                <Tooltip content={i18n.t('chat.detach_tooltip')} position="top">
                                    <button 
                                        onClick={toggle_detach} 
                                        className="p-2 text-zinc-500 hover:text-zinc-200 hover:bg-zinc-800/50 rounded-lg transition-colors"
                                        aria-label={i18n.t('chat.detach_aria')}
                                    >
                                        <ExternalLink size={16} />
                                    </button>
                                </Tooltip>
                                <Tooltip content={i18n.t('chat.close_tooltip')} position="top">
                                    <button 
                                        onClick={clear_history} 
                                        className="p-2 text-red-500/50 hover:text-red-400 hover:bg-red-500/10 rounded-lg transition-colors"
                                        aria-label={i18n.t('chat.close_aria')}
                                    >
                                        <X size={16} />
                                    </button>
                                </Tooltip>
                            </div>
                        </div>

                        {/* Neural Lineage Breadcrumbs */}
                        {active_scope === 'agent' && (
                            <div className="bg-zinc-950/40 border-b border-zinc-800/30 px-4 py-2 flex items-center gap-2 overflow-x-auto no-scrollbar relative z-10 select-none">
                                <span className="text-[10px] text-zinc-600 font-bold uppercase tracking-wider whitespace-nowrap">{i18n.t('chat.lineage_label')}</span>
                                <div className="flex items-center gap-1.5 scroll-smooth">
                                    <span className="text-[10px] text-zinc-100 bg-zinc-800 px-2 py-0.5 rounded border border-zinc-700/50 hover:bg-zinc-700 transition-colors cursor-default shadow-sm">{i18n.t('chat.overlord_name')}</span>
                                    {target_agent !== 'CEO' && (
                                        <>
                                            <span className="text-zinc-700 text-[10px] animate-pulse">/</span>
                                            <span className="text-[10px] text-blue-400 bg-blue-500/10 px-2 py-0.5 rounded border border-blue-500/20 hover:bg-blue-500/20 transition-all cursor-default shadow-[0_0_10px_rgba(59,130,246,0.15)]">{i18n.t('chat.agent_label', { name: target_agent })}</span>
                                        </>
                                    )}
                                </div>
                            </div>
                        )}

                        {/* Scope & Target Selector */}
                        <div className="relative z-20 flex flex-col border-b border-zinc-800/30">
                            <div className="flex p-1.5 bg-zinc-950/20 backdrop-blur-sm gap-1">
                                {(['agent', 'cluster', 'swarm'] as Sovereign_Scope[]).map(scope => (
                                    <button
                                        key={scope}
                                        onClick={() => set_scope(scope)}
                                        className={clsx(
                                            "flex-1 py-1.5 px-2 text-[10px] font-bold uppercase tracking-[0.15em] rounded-md transition-all relative overflow-hidden",
                                            active_scope === scope ? "text-zinc-100" : "text-zinc-600 hover:text-zinc-400"
                                        )}
                                        aria-pressed={active_scope === scope}
                                    >
                                        {active_scope === scope && (
                                            <motion.div layoutId="scope-bg" className="absolute inset-0 bg-zinc-800 border border-zinc-700/50 shadow-inner rounded-md" />
                                        )}
                                        <span className="relative z-10">{scope}</span>
                                    </button>
                                ))}
                            </div>

                            {active_scope !== 'swarm' && (
                                <div className="px-3 pb-2 flex items-center gap-2">
                                    {/* Agent Selector */}
                                    <div className="relative flex-1 min-w-0">
                                        <button
                                            onClick={() => {
                                                set_open_dropdown(open_dropdown === 'agent' ? null : 'agent');
                                                if (active_scope !== 'agent') set_scope('agent');
                                            }}
                                            className={clsx(
                                                "w-full flex items-center justify-between gap-2 text-[10px] font-bold transition-colors uppercase tracking-widest bg-zinc-900/50 px-2 py-1.5 rounded border group",
                                                active_scope === 'agent' ? "border-blue-500/50 text-blue-400" : "border-zinc-800 text-zinc-500 hover:text-zinc-300"
                                            )}
                                            aria-haspopup="listbox"
                                            aria-expanded={open_dropdown === 'agent'}
                                            aria-label={i18n.t('chat.select_agent_aria')}
                                        >
                                            <div className="flex items-center gap-1.5 truncate">
                                                <TargetIcon size={12} className={active_scope === 'agent' ? "text-blue-500" : "text-zinc-600"} />
                                                <span className="truncate">{i18n.t('chat.agent_prefix')}<span className={active_scope === 'agent' ? "text-zinc-100" : "text-zinc-400"}>{target_agent || i18n.t('chat.select_placeholder')}</span></span>
                                            </div>
                                            <ChevronDown size={12} className={clsx("transition-transform flex-shrink-0", open_dropdown === 'agent' && "rotate-180")} />
                                        </button>

                                        <AnimatePresence>
                                            {open_dropdown === 'agent' && (
                                                <motion.div
                                                    initial={{ opacity: 0, y: -10 }}
                                                    animate={{ opacity: 1, y: 0 }}
                                                    exit={{ opacity: 0, y: -10 }}
                                                    className="absolute left-0 top-full mt-1 w-full min-w-[160px] bg-zinc-900 border border-zinc-800 rounded-lg shadow-2xl z-20 py-1 overflow-y-auto max-h-64 custom-scrollbar backdrop-blur-xl"
                                                >
                                                    {sorted_agents.map(agent => (
                                                        <button
                                                            key={agent.id}
                                                            onClick={() => {
                                                                set_target_agent(agent.name);
                                                                set_selected_agent_id(agent.id);
                                                                set_scope('agent');
                                                                set_open_dropdown(null);
                                                            }}
                                                            className="w-full text-left px-3 py-2 text-xs hover:bg-zinc-800 text-zinc-400 hover:text-zinc-100 flex items-center gap-2 transition-colors"
                                                        >
                                                            <div className={clsx("w-2 h-2 rounded-full flex-shrink-0", agent.status === 'offline' ? "opacity-30" : "")} style={{ backgroundColor: agent.theme_color || '#52525b' }} />
                                                            <span className={clsx("truncate flex-1 max-w-[100px]", agent.status === 'offline' && "text-zinc-600")}>{agent.name}</span>
                                                            {agent.status !== 'offline' && agent.status !== 'idle' && (
                                                                <div className="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse ml-auto" />
                                                            )}
                                                        </button>
                                                    ))}
                                                </motion.div>
                                            )}
                                        </AnimatePresence>
                                    </div>

                                    {/* Cluster Selector */}
                                    <div className="relative flex-1 min-w-0">
                                        <button
                                            onClick={() => {
                                                set_open_dropdown(open_dropdown === 'cluster' ? null : 'cluster');
                                                if (active_scope !== 'cluster') set_scope('cluster');
                                            }}
                                            className={clsx(
                                                "w-full flex items-center justify-between gap-2 text-[10px] font-bold transition-colors uppercase tracking-widest bg-zinc-900/50 px-2 py-1.5 rounded border group",
                                                active_scope === 'cluster' ? "border-emerald-500/50 text-emerald-400" : "border-zinc-800 text-zinc-500 hover:text-zinc-300"
                                            )}
                                            aria-haspopup="listbox"
                                            aria-expanded={open_dropdown === 'cluster'}
                                            aria-label={i18n.t('chat.select_cluster_aria')}
                                        >
                                            <div className="flex items-center gap-1.5 truncate">
                                                <TargetIcon size={12} className={active_scope === 'cluster' ? "text-emerald-500" : "text-zinc-600"} />
                                                <span className="truncate">{i18n.t('chat.cluster_prefix')}<span className={active_scope === 'cluster' ? "text-zinc-100" : "text-zinc-400"}>{target_cluster || i18n.t('chat.select_placeholder')}</span></span>
                                            </div>
                                            <ChevronDown size={12} className={clsx("transition-transform flex-shrink-0", open_dropdown === 'cluster' && "rotate-180")} />
                                        </button>

                                        <AnimatePresence>
                                            {open_dropdown === 'cluster' && (
                                                <motion.div
                                                    initial={{ opacity: 0, y: -10 }}
                                                    animate={{ opacity: 1, y: 0 }}
                                                    exit={{ opacity: 0, y: -10 }}
                                                    className="absolute right-0 top-full mt-1 w-full min-w-[160px] bg-zinc-900 border border-zinc-800 rounded-lg shadow-2xl z-20 py-1 overflow-y-auto max-h-64 custom-scrollbar backdrop-blur-xl"
                                                >
                                                    {clusters.map(cluster => (
                                                        <button
                                                            key={cluster.id}
                                                            onClick={() => {
                                                                set_target_cluster(cluster.name);
                                                                set_scope('cluster');
                                                                set_open_dropdown(null);
                                                            }}
                                                            className="w-full text-left px-3 py-2 text-xs hover:bg-zinc-800 text-zinc-400 hover:text-zinc-100 flex items-center gap-2 transition-colors"
                                                        >
                                                            <div className={clsx(
                                                                "w-2 h-2 rounded-full flex-shrink-0",
                                                                cluster.theme === 'cyan' ? 'bg-cyan-500' :
                                                                    cluster.theme === 'zinc' ? 'bg-zinc-500' :
                                                                        cluster.theme === 'amber' ? 'bg-amber-500' : 'bg-blue-500'
                                                            )} />
                                                            <span className="truncate">{cluster.name}</span>
                                                        </button>
                                                    ))}
                                                </motion.div>
                                            )}
                                        </AnimatePresence>
                                    </div>
                                </div>
                            )}
                        </div>

                        {/* Messages Window */}
                        <div
                            ref={scroll_ref}
                            className="relative z-10 flex-1 overflow-y-auto p-5 space-y-6 custom-scrollbar"
                        >
                            {show_transcript ? (
                                <Buffered_Transcript_View agent_id={selected_agent_id || undefined} />
                            ) : (
                                <>
                                    {filtered_messages.length === 0 && (
                                        <div className="h-full flex flex-col items-center justify-center text-zinc-700 space-y-4 opacity-40 text-center">
                                            <Bot size={32} className="neural-pulse" />
                                            <div>
                                                <p className="text-xs font-bold uppercase tracking-[0.3em]">
                                                    {active_scope === 'agent' ? i18n.t('chat.isolated_stream', { target: target_node }) : i18n.t('chat.waiting_packets')}
                                                </p>
                                                <p className="text-[9px] font-mono mt-1 opacity-60">{i18n.t('chat.ready_for_input')}</p>
                                            </div>
                                        </div>
                                    )}
                                    {hidden_message_count > 0 && (
                                        <div className="text-[9px] font-mono uppercase tracking-[0.14em] text-zinc-500 bg-zinc-900/50 border border-zinc-800 rounded-lg px-3 py-2 text-center">
                                            Rendering latest {MAX_RENDERED_MESSAGES} packets ({hidden_message_count} older packets hidden)
                                        </div>
                                    )}
                                    {rendered_messages.map((msg) => (
                                        <motion.div
                                            key={msg.id}
                                            initial={{ opacity: 0, x: msg.sender_id === '0' ? 10 : -10 }}
                                            animate={{ opacity: 1, x: 0 }}
                                            style={{ contentVisibility: 'auto', containIntrinsicSize: '180px' }}
                                            className={clsx(
                                                "flex flex-col gap-2 max-w-[90%]",
                                                msg.sender_id === '0' ? "ml-auto items-end" : "items-start"
                                            )}
                                        >
                                            <div className="flex items-center gap-2 px-1">
                                                {msg.sender_id !== '0' && (
                                                    <Bot size={12} className="text-zinc-500" />
                                                )}
                                                <span className="text-[9px] font-bold text-zinc-500 uppercase tracking-widest">
                                                    {msg.sender_name}
                                                </span>
                                                {msg.sender_id === '0' && (
                                                    <User size={12} className="text-zinc-500" />
                                                )}
                                            </div>
                                            <div
                                                className={clsx(
                                                    "px-4 py-3 rounded-2xl text-[13px] leading-relaxed relative group border transition-all duration-300 overflow-hidden break-words whitespace-pre-wrap flex flex-col gap-3",
                                                    msg.sender_id === '0'
                                                        ? "bg-zinc-100 text-zinc-900 rounded-tr-sm shadow-[0_5px_20px_-5px_rgba(255,255,255,0.2)]"
                                                        : "bg-zinc-800/80 text-zinc-200 rounded-tl-sm border-zinc-700/50 shadow-xl",
                                                    (msg.text.length > 800 || (msg.parts?.length || 0) > 5) ? "max-h-[400px] overflow-y-auto custom-scrollbar text-xs" : ""
                                                )}
                                            >
                                                {msg.parts && msg.parts.length > 0 ? (
                                                    (msg.parts as Message_Part[]).map((part: Message_Part, idx) => (
                                                        <div key={`${msg.id}-part-${idx}`} className="animate-in fade-in slide-in-from-bottom-1 duration-300">
                                                            {part.type === 'text' && (
                                                                <span className="opacity-90">{part.content}</span>
                                                            )}
                                                            {part.type === 'thought' && (
                                                                <div className="flex items-start gap-2 py-1 px-2 bg-black/20 rounded-lg border border-white/5 italic text-zinc-400 text-[11px] leading-tight group/thought">
                                                                    <div className="mt-1">
                                                                        {part.status === 'thinking' ? (
                                                                            <div className="w-2 h-2 rounded-full bg-blue-500/50 animate-ping" />
                                                                        ) : (
                                                                            <Bot size={10} className="text-blue-400/60" />
                                                                        )}
                                                                    </div>
                                                                    <span>{part.content}</span>
                                                                </div>
                                                            )}
                                                            {part.type === 'tool' && (
                                                                <div className="flex flex-col gap-1 py-2 px-3 bg-zinc-900/40 rounded-lg border border-zinc-700/30 group/tool">
                                                                    <div className="flex items-center gap-2">
                                                                        <Zap size={10} className="text-amber-400" />
                                                                        <span className="font-mono text-[10px] uppercase tracking-wider text-zinc-300">
                                                                            Execution: {part.name || 'unknown'}
                                                                        </span>
                                                                    </div>
                                                                    {Boolean(part.output) && (
                                                                        <div className="mt-1 text-[10px] font-mono text-zinc-500 overflow-hidden text-ellipsis">
                                                                            Result: {typeof part.output === 'string' ? part.output : JSON.stringify(part.output).slice(0, 100) + '...'}
                                                                        </div>
                                                                    )}
                                                                </div>
                                                            )}
                                                        </div>
                                                    ))
                                                ) : (
                                                    msg.text
                                                )}
                                                {msg.scope === 'swarm' && msg.sender_id !== '0' && (
                                                    <div className="absolute top-0 right-0 p-1">
                                                        <Zap size={8} className="text-blue-400" />
                                                    </div>
                                                )}
                                            </div>
                                            <div className="flex flex-row items-center gap-2 px-1 text-[8px] font-mono text-zinc-600 uppercase tracking-tighter whitespace-nowrap">
                                                <span>{new Date(msg.timestamp).toLocaleTimeString([], { hour12: false, minute: '2-digit' })}</span>
                                                <span>•</span>
                                                <span className="text-zinc-400">{i18n.t(`chat.scope_${msg.scope}`)}</span>
                                            </div>
                                        </motion.div>
                                    ))}
                                </>
                            )}
                        </div>

                        {/* Input Area */}
                        <div className="relative z-10 p-4 bg-zinc-950/50 backdrop-blur-xl border-t border-zinc-800/50">
                            <div className="flex items-center gap-2 bg-black/40 border border-zinc-800/60 rounded-xl p-1 focus-within:border-zinc-500/50 transition-all shadow-inner relative">
                                {is_listening && (
                                    <div className="absolute inset-0 bg-blue-500/10 animate-pulse pointer-events-none rounded-xl" />
                                )}

                                <Tooltip content={is_listening ? i18n.t('chat.stop_listening_tooltip') : i18n.t('chat.start_listening_tooltip')} position="top">
                                    <button
                                        onClick={toggle_voice}
                                        className={clsx(
                                            "p-2.5 rounded-lg transition-all active:scale-90",
                                            is_listening ? "bg-red-500 text-white animate-pulse" : "text-zinc-500 hover:text-zinc-100 hover:bg-zinc-800"
                                        )}
                                    >
                                        {is_listening ? <MicOff size={18} /> : <Mic size={18} />}
                                    </button>
                                </Tooltip>

                                <Tooltip content={is_speech_enabled ? i18n.t('chat.mute_output_tooltip') : i18n.t('chat.enable_output_tooltip')} position="top">
                                    <button
                                        onClick={() => set_is_speech_enabled(!is_speech_enabled)}
                                        className={clsx(
                                            "p-2.5 rounded-lg transition-all active:scale-90",
                                            is_speech_enabled ? "text-blue-400 bg-blue-500/10" : "text-zinc-500 hover:text-zinc-100 hover:bg-zinc-800"
                                        )}
                                    >
                                        {is_speech_enabled ? <Volume2 size={18} className={is_speaking ? "animate-bounce" : ""} /> : <VolumeX size={18} />}
                                    </button>
                                </Tooltip>

                                <input
                                    type="text"
                                    value={input_value}
                                    onChange={(e) => set_input_value(e.target.value)}
                                    onKeyDown={(e) => e.key === 'Enter' && handle_send()}
                                    placeholder={is_listening ? i18n.t('chat.listening_placeholder') : i18n.t('chat.input_placeholder', { scope: i18n.t(`chat.scope_${active_scope}`) })}
                                    className="flex-1 min-w-0 bg-transparent border-none focus:ring-0 text-sm py-2 px-2 text-zinc-100 placeholder:text-zinc-600 font-medium"
                                    aria-label={i18n.t('chat.input_label')}
                                />

                                <Tooltip content={is_safe_mode ? i18n.t('chat.safety_on_tooltip') : i18n.t('chat.safety_off_tooltip')} position="top">
                                    <button
                                        onClick={() => update_setting('is_safe_mode', !is_safe_mode)}
                                        className={clsx(
                                            "p-2.5 rounded-lg transition-all active:scale-90",
                                            is_safe_mode ? "bg-blue-500/20 text-blue-400 border border-blue-500/30" : "text-zinc-500 hover:text-zinc-100 hover:bg-zinc-800"
                                        )}
                                    >
                                        <BrainCircuit size={18} className={is_safe_mode ? "animate-pulse" : ""} />
                                    </button>
                                </Tooltip>

                                <Tooltip content={i18n.t('chat.send_tooltip')} position="top">
                                    <button
                                        onClick={() => handle_send()}
                                        className="p-2.5 bg-zinc-100 text-black rounded-lg hover:bg-white hover:scale-105 active:scale-95 transition-all shadow-lg"
                                    >
                                        <Send size={16} />
                                    </button>
                                </Tooltip>
                            </div>
                        </div>
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
                        className="fixed bottom-6 right-6 z-50 bg-zinc-100 text-black px-5 py-3 rounded-2xl shadow-[0_10px_40px_-10px_rgba(255,255,255,0.3)] flex items-center gap-3 group border border-white cursor-grab active:cursor-grabbing"
                    >
                        <Zap size={20} className="group-hover:animate-pulse pointer-events-none" />
                        <span className="text-xs font-bold uppercase tracking-widest pointer-events-none">{i18n.t('chat.title')}</span>
                    </motion.button>
                )}
            </AnimatePresence>
        </>
    );
};
