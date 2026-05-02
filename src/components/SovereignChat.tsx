/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Detachable Command & Control (C2) interface for Swarm Intelligence. 
 * Orchestrates triple-scope communication (Agent/Cluster/Swarm), real-time voice synthesis (Azure/Groq), and transcript buffering for autonomous agent logs.
 * 
 * ### 🧬 Logic Flow (Mermaid)
 * ```mermaid
 * stateDiagram-v2
 *     [*] --> Initialize: useChatWindow hook
 *     Initialize --> Active: Mounting [Shared Context]
 *     Active --> Detached: onClick(ExternalLink) [Portal_Window]
 *     Detached --> Active: on_close (Recall)
 * 
 *     state "Command Execution Flow" as CMD {
 *         [*] --> Validate: process_command(scope)
 *         Validate --> Dispatch: agent_api.send_command
 *         Dispatch --> Relay: Use sovereign_store [Append Message]
 *         Relay --> Voice: Synthesis (Optional)
 *     }
 * ```
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Voice client initialization stall, portal window context loss (detachment), or message packet starvation during high-frequency telemetry storms.
 * - **Telemetry Link**: Search for `[SovereignChat]` or `sovereign_store` in browser logs.
 */

import React, { useState, useRef, useEffect, useMemo, useCallback } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import {
    X,
    Maximize2,
    Minimize2,
    Zap,
    ExternalLink,
    Target as TargetIcon,
    ChevronDown,
    GripVertical,
    Activity
} from 'lucide-react';
import clsx from 'clsx';
import { use_settings_store } from '../stores/settings_store';
import { use_sovereign_store, type Sovereign_Scope, type Chat_Message } from '../stores/sovereign_store';
import { use_agent_store } from '../stores/agent_store';
import { use_workspace_store, type Mission_Cluster } from '../stores/workspace_store';
import { process_command } from '../logic/command_processor';
import { voice_client } from '../services/voice_client';
import { useDragControls, type DragControls } from 'framer-motion';
import { useChatWindow } from '../hooks/use_chat_window';
import type { Agent } from '../types';
import { Tooltip } from './ui';
import { i18n } from '../i18n';
import { Portal_Window } from './ui/Portal_Window';
import { Buffered_Transcript_View } from './transcript/Buffered_Transcript_View';
import { Chat_Message_List } from './chat/Chat_Message_List';
import { Chat_Input_Bar } from './chat/Chat_Input_Bar';
import { type Voice_Status } from '../services/voice_client';

const TELEMETRY_SOURCE = '[SovereignChat]';

/**
 * SovereignChat
 * A high-performance, detached-capable chat interface for agent orchestration.
 * Supports triple-scope communication: Agent, Cluster, and Swarm.
 * Enhanced with voice input and context isolation.
 * Refactored for strict snake_case compliance and consistent service integration.
 */
interface SovereignChatProps {
    isDetachedView?: boolean;
}

export const SovereignChat: React.FC<SovereignChatProps> = ({ isDetachedView }) => {
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
    const [voice_status, set_voice_status] = useState<Voice_Status>('idle');
    const [popup_blocked, set_popup_blocked] = useState(false);
    const [is_speech_enabled, set_is_speech_enabled] = useState(false);
    const [is_speaking, set_is_speaking] = useState(false);
    const { settings, update_setting } = use_settings_store();
    const is_safe_mode = settings.is_safe_mode;
    const [open_dropdown, set_open_dropdown] = useState<'agent' | 'cluster' | null>(null);
    const [show_transcript, set_show_transcript] = useState(false);
    const [input_text, set_input_text] = useState('');
    const last_spoken_id_ref = useRef<string | null>(null);
    const speak_start_timeout_ref = useRef<ReturnType<typeof setTimeout> | null>(null);
    const speak_end_timeout_ref = useRef<ReturnType<typeof setTimeout> | null>(null);
    const drag_controls = useDragControls();

    // Subscribe to voice status changes for UI feedback
    useEffect(() => {
        voice_client.on_status_change(status => {
            console.debug(`${TELEMETRY_SOURCE} Voice status transition: ${status}`);
            set_voice_status(status);
        });
    }, []);

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

    // Conservative auto-selection: only if absolutely no target is set and agents exist
    useEffect(() => {
        const is_ungetTarget = !target_agent || target_agent.toLowerCase() === 'ceo';
        if (agents.length > 0 && !selected_agent_id && is_ungetTarget) {
            // Find CEO by role check instead of literal name
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

    useEffect(() => {
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
        if (!last_message || last_message.id === last_spoken_id_ref.current) return;

        if (is_speech_enabled && last_message.sender_id !== '0' && last_message.sender_id === selected_agent_id) {
            // Guard: Don't auto-speak technical errors or security alerts
            if (last_message.text.startsWith('❌') || last_message.text.startsWith('🛡️') || last_message.text.includes('Error:')) {
                return;
            }

            const agent = agents.find(a => a.id === selected_agent_id);
            if (agent) {
                last_spoken_id_ref.current = last_message.id; // Mark as handled
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

    const fetch_agents = use_agent_store(s => s.fetch_agents);
    useEffect(() => {
        if (agents.length === 0) {
            fetch_agents();
        }
    }, [agents.length, fetch_agents]);
    
    // Header interaction: Toggle between maximized and minimized states
    const handle_header_click = () => {
        if (is_minimized) perform_maximize_transform();
        else perform_minimize_transform();
    };

    /**
     * Standardized Command Dispatcher
     * 
     * ### 🛰️ Orchestration: Intent Capturing
     * 1. **Message Injection**: Appends the user's message to the local store 
     *    with the requested scope (Agent/Cluster/Swarm).
     * 2. **State Sync**: Pulls the freshest state from `use_sovereign_store` 
     *    to ensure the sub-agent target is not stale.
     * 3. **Logic Dispatch**: routes the text to `process_command` with 
     *    `safe_mode` and `traceparent` context.
     * 4. **Fault Tolerance**: Intercepts command faults and injects them back 
     *    into the chat stream as system alerts (ERR-03).
     */
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
        set_input_text(''); // 🧹 Clear the box immediately for visual feedback

        try {
            console.debug(`${TELEMETRY_SOURCE} Intent captured: ${text.substring(0, 50)}...`);
            
            // 🚨 OVERLORD SYNC: Pull freshest state directly from stores to prevent ghost-targeting stale nodes
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

    const toggle_voice = useCallback(() => {
        if (voice_status !== 'idle') {
            voice_client.stop_listening();
        } else {
            set_is_speech_enabled(true);
            voice_client.start_listening(() => {
                // Potential hook for real-time log ingestion:
                // event_bus.emit_log({ source: 'Voice_Client', text: `[Transcript] ${transcript}`, severity: 'info' });
            });
        }
    }, [voice_status]);

    const toggle_speech = useCallback(() => {
        set_is_speech_enabled(prev => !prev);
    }, []);

    const toggle_safety = useCallback(() => {
        update_setting('is_safe_mode', !is_safe_mode);
    }, [is_safe_mode, update_setting]);

    /**
     * REACTIVE FILTERING LOGIC
     */
    const filtered_messages = useMemo(() => messages.filter(m => {
        // 1. Global Swarm Scope: Always visible
        if (active_scope === 'swarm') return true;

        // 2. Agent Isolation
        if (active_scope === 'agent') {
            const target = (target_agent ?? '').toLowerCase();
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


    /**
     * Portal Detachment Logic
     * 
     * ### 🪟 Interface: External Window Portal
     * When `is_detached` is active, the main chat UI is unmounted from the 
     * layout and re-initialized within a `Portal_Window`. 
     * 
     * ### 🛰️ Context Persistence
     * Because the state is held in global Zustand stores (`use_sovereign_store`, 
     * `use_agent_store`), the detached window retains full parity with the 
     * primary workspace without needing prop-drilling or event-bus remapping.
     */
    if (is_detached && !isDetachedView) {
        return (
            <>
                <div className="fixed bottom-6 right-6 z-50 flex flex-col items-end gap-3">
                    <AnimatePresence>
                        {popup_blocked && (
                            <motion.div
                                initial={{ opacity: 0, x: 20 }}
                                animate={{ opacity: 1, x: 0 }}
                                className="bg-red-500/90 text-white text-[10px] font-bold px-3 py-1.5 rounded-lg shadow-xl backdrop-blur-md"
                            >
                                ⚠️ {i18n.t('chat.popup_blocked_warning')}
                            </motion.div>
                        )}
                    </AnimatePresence>
                    <Tooltip content={i18n.t('chat.restore_tooltip')} position="top">
                        <button
                            onClick={() => set_detached(false)}
                            className="bg-zinc-900/80 backdrop-blur-md border border-zinc-700/50 p-4 rounded-full text-zinc-400 hover:text-zinc-100 shadow-[0_0_20px_rgba(0,0,0,0.5)] transition-all hover:scale-110 active:scale-95 group"
                        >
                            <Maximize2 size={24} className="group-hover:rotate-12 transition-transform" />
                        </button>
                    </Tooltip>
                </div>

                <Portal_Window
                    id="sovereign-chat"
                    title={`${i18n.t('chat.title')} - ${i18n.t(`chat.scope_${active_scope}`)}`}
                    on_close={() => set_detached(false)}
                    on_popup_block={() => set_popup_blocked(true)}
                    width={440}
                    height={720}
                    url="/detached/chat"
                >
                    <div className="w-full h-full bg-zinc-950 text-white overflow-hidden flex flex-col">
                        <SovereignChatContent 
                            isDetached={true} 
                            activeScope={active_scope}
                            targetNode={target_node}
                            targetAgent={target_agent}
                            targetCluster={target_cluster}
                            selectedAgentId={selected_agent_id}
                            isSpeaking={is_speaking}
                            voiceStatus={voice_status}
                            showTranscript={show_transcript}
                            setShowTranscript={set_show_transcript}
                            messages={filtered_messages}
                            maxRenderedMessages={MAX_RENDERED_MESSAGES}
                            onSend={handle_send}
                            inputText={input_text}
                            setInputText={set_input_text}
                            onToggleVoice={toggle_voice}
                            onToggleSpeech={toggle_speech}
                            onToggleSafety={toggle_safety}
                            isSafeMode={is_safe_mode}
                            isSpeechEnabled={is_speech_enabled}
                            onToggleDetach={toggle_detach}
                            onClearHistory={clear_history}
                            onSetScope={set_scope}
                            openDropdown={open_dropdown}
                            setOpenDropdown={set_open_dropdown}
                            sortedAgents={sorted_agents}
                            setTargetAgent={set_target_agent}
                            setSelectedAgentId={set_selected_agent_id}
                            setTargetCluster={set_target_cluster}
                            clusters={clusters}
                            onMinimize={perform_minimize_transform}
                            // Portal-specific props
                            containerProps={{}}
                        />
                    </div>
                </Portal_Window>
            </>
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
                            "bottom-6 right-6 w-[440px] h-[600px] rounded-2xl border border-zinc-800/50 shadow-[0_30px_60px_-15px_rgba(0,0,0,0.7)] bg-zinc-900/40 backdrop-blur-xl pointer-events-auto"
                        )}
                    >
                        <SovereignChatContent 
                            isDetached={false} 
                            activeScope={active_scope}
                            targetNode={target_node}
                            targetAgent={target_agent}
                            targetCluster={target_cluster}
                            selectedAgentId={selected_agent_id}
                            isSpeaking={is_speaking}
                            voiceStatus={voice_status}
                            showTranscript={show_transcript}
                            setShowTranscript={set_show_transcript}
                            messages={filtered_messages}
                            maxRenderedMessages={MAX_RENDERED_MESSAGES}
                            onSend={handle_send}
                            inputText={input_text}
                            setInputText={set_input_text}
                            onToggleVoice={toggle_voice}
                            onToggleSpeech={toggle_speech}
                            onToggleSafety={toggle_safety}
                            isSafeMode={is_safe_mode}
                            isSpeechEnabled={is_speech_enabled}
                            onToggleDetach={toggle_detach}
                            onClearHistory={clear_history}
                            onSetScope={set_scope}
                            openDropdown={open_dropdown}
                            setOpenDropdown={set_open_dropdown}
                            sortedAgents={sorted_agents}
                            setTargetAgent={set_target_agent}
                            setSelectedAgentId={set_selected_agent_id}
                            setTargetCluster={set_target_cluster}
                            clusters={clusters}
                            onMinimize={perform_minimize_transform}
                            dragControls={drag_controls}
                            onHeaderClick={handle_header_click}
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

/**
 * SovereignChatContent
 * Extracted UI content of the chat window for multi-portal rendering parity.
 */
interface SovereignChatContentProps {
    isDetached: boolean;
    activeScope: Sovereign_Scope;
    targetNode: string;
    targetAgent: string;
    targetCluster: string;
    selectedAgentId: string | null;
    isSpeaking: boolean;
    voiceStatus: Voice_Status;
    showTranscript: boolean;
    setShowTranscript: (show: boolean) => void;
    messages: Chat_Message[];
    maxRenderedMessages: number;
    onSend: () => Promise<void>;
    inputText: string;
    setInputText: (text: string) => void;
    onToggleVoice: () => void;
    onToggleSpeech: () => void;
    onToggleSafety: () => void;
    isSafeMode: boolean;
    isSpeechEnabled: boolean;
    onToggleDetach: () => void;
    onClearHistory: () => void;
    onSetScope: (scope: Sovereign_Scope) => void;
    openDropdown: 'agent' | 'cluster' | null;
    setOpenDropdown: (val: 'agent' | 'cluster' | null) => void;
    sortedAgents: Agent[];
    setTargetAgent: (name: string) => void;
    setSelectedAgentId: (id: string) => void;
    setTargetCluster: (name: string) => void;
    clusters: Mission_Cluster[];
    onMinimize: () => void;
    dragControls?: DragControls;
    onHeaderClick?: () => void;
    containerProps?: React.HTMLAttributes<HTMLDivElement>;
}

const SovereignChatContent: React.FC<SovereignChatContentProps> = ({
    isDetached,
    activeScope,
    targetNode,
    targetAgent,
    targetCluster,
    selectedAgentId,
    isSpeaking,
    voiceStatus,
    showTranscript,
    setShowTranscript,
    messages,
    maxRenderedMessages,
    onSend,
    inputText,
    setInputText,
    onToggleVoice,
    onToggleSpeech,
    onToggleSafety,
    isSafeMode,
    isSpeechEnabled,
    onToggleDetach,
    onClearHistory,
    onSetScope,
    openDropdown,
    setOpenDropdown,
    sortedAgents,
    setTargetAgent,
    setSelectedAgentId,
    setTargetCluster,
    clusters,
    onMinimize,
    dragControls,
    onHeaderClick,
    containerProps
}) => {
    const scroll_ref = useRef<HTMLDivElement>(null);

    useEffect(() => {
        if (scroll_ref.current) {
            scroll_ref.current.scrollTo({
                top: scroll_ref.current.scrollHeight,
                behavior: 'smooth'
            });
        }
    }, [messages]);

    return (
        <div className="w-full h-full flex flex-col relative" {...containerProps}>
            {!isDetached && <div className="neural-grid opacity-[0.05] absolute inset-0 pointer-events-none" />}

            {/* Header */}
            <div
                onPointerDown={(e) => {
                    if (!isDetached && dragControls) {
                        dragControls.start(e);
                    }
                }}
                className={clsx(
                    "relative z-10 p-4 border-b border-zinc-800/50 bg-zinc-950/40 backdrop-blur-md flex items-center justify-between shrink-0 overflow-hidden cursor-pointer select-none",
                    !isDetached && "cursor-grab active:cursor-grabbing"
                )}
                onDoubleClick={onHeaderClick}
                onKeyDown={(e) => (e.key === 'Enter' || e.key === ' ') && onHeaderClick?.()}
                role="button"
                tabIndex={0}
                title={i18n.t('chat.header_drag_hint')}
                aria-label={i18n.t('chat.header_drag_aria')}
            >
                <div className="flex items-center gap-3">
                    {!isDetached && <GripVertical size={14} className="text-zinc-700" />}
                    <div className="relative bg-zinc-100 p-1.5 rounded-md text-black shadow-lg">
                        <Zap size={14} className="fill-current" />
                    </div>
                    <div>
                        <span className="font-bold text-[11px] tracking-[0.2em] text-zinc-100 uppercase">{i18n.t('chat.title')}</span>
                        <div className="flex items-center gap-1.5">
                            <div className={clsx(
                                "h-1 w-1 rounded-full animate-pulse",
                                voiceStatus === 'active' ? "bg-emerald-500" : 
                                voiceStatus === 'initializing' ? "bg-green-500" :
                                voiceStatus === 'stalled' ? "bg-red-500 shadow-[0_0_8px_rgba(239,68,68,0.8)]" : "bg-emerald-500"
                            )} />
                            <span className="text-[9px] text-zinc-500 font-mono uppercase tracking-tighter">
                                {i18n.t(`chat.scope_${activeScope}`)} / {activeScope !== 'swarm' ? targetNode : i18n.t('chat.sovereign_link')}
                            </span>
                            {isSpeaking && (
                                <div className="flex items-center gap-0.5 ml-2 mr-1">
                                    {[1, 2, 3, 4].map(i => (
                                        <motion.div
                                            key={i}
                                            animate={{ height: [4, 12, 4] }}
                                            transition={{ repeat: Infinity, duration: 0.6, delay: i * 0.1 }}
                                            className="w-0.5 bg-green-500 rounded-full"
                                        />
                                    ))}
                                </div>
                            )}
                        </div>
                    </div>
                </div>
                <div className="flex items-center gap-1">
                    <Tooltip content={showTranscript ? i18n.t('chat.show_chat_tooltip') : i18n.t('chat.show_transcript_tooltip')} position="top">
                        <button
                            onClick={(e) => {
                                e.stopPropagation();
                                setShowTranscript(!showTranscript);
                            }}
                            className={clsx(
                                "p-2 rounded-lg transition-all active:scale-95",
                                showTranscript ? "text-green-400 bg-green-500/10 border border-green-500/30 shadow-[0_0_15px_rgba(59,130,246,0.15)]" : "text-zinc-500 hover:text-zinc-200 hover:bg-zinc-800/50"
                            )}
                            aria-label={i18n.t('chat.toggle_transcript_aria')}
                        >
                            <Activity size={16} />
                        </button>
                    </Tooltip>
                    {!isDetached && (
                        <Tooltip content={i18n.t('chat.minimize_tooltip')} position="top">
                            <button
                                onClick={(e) => {
                                    e.stopPropagation();
                                    onMinimize();
                                }}
                                className="p-2 text-zinc-500 hover:text-zinc-200 hover:bg-zinc-800/50 rounded-lg transition-colors"
                                aria-label={i18n.t('chat.minimize_aria')}
                            >
                                <Minimize2 size={16} />
                            </button>
                        </Tooltip>
                    )}
                    <Tooltip content={isDetached ? i18n.t('chat.restore_tooltip') : i18n.t('chat.detach_tooltip')} position="top">
                        <button 
                            onClick={(e) => { e.stopPropagation(); onToggleDetach(); }} 
                            className={clsx(
                                "p-2 rounded-lg transition-colors",
                                isDetached ? "text-green-400 bg-green-500/10" : "text-zinc-500 hover:text-zinc-200 hover:bg-zinc-800/50"
                            )}
                            aria-label={isDetached ? i18n.t('chat.restore_aria') : i18n.t('chat.detach_aria')}
                        >
                            <ExternalLink size={16} />
                        </button>
                    </Tooltip>
                    <Tooltip content={i18n.t('chat.close_tooltip')} position="top">
                        <button 
                            onClick={(e) => { e.stopPropagation(); onClearHistory(); }} 
                            className="p-2 text-red-500/50 hover:text-red-400 hover:bg-red-500/10 rounded-lg transition-colors"
                            aria-label={i18n.t('chat.close_aria')}
                        >
                            <X size={16} />
                        </button>
                    </Tooltip>
                </div>
            </div>

            {/* Neural Lineage Breadcrumbs */}
            {activeScope === 'agent' && (
                <div className="bg-zinc-950/40 border-b border-zinc-800/30 px-4 py-2 flex items-center gap-2 overflow-x-auto no-scrollbar relative z-10 select-none">
                    <span className="text-[10px] text-zinc-600 font-bold uppercase tracking-wider whitespace-nowrap">{i18n.t('chat.lineage_label')}</span>
                    <div className="flex items-center gap-1.5 scroll-smooth">
                        <span className="text-[10px] text-zinc-100 bg-zinc-800 px-2 py-0.5 rounded border border-zinc-700/50 hover:bg-zinc-700 transition-colors cursor-default shadow-sm">{i18n.t('chat.overlord_name')}</span>
                        {targetAgent !== 'CEO' && (
                            <>
                                <span className="text-zinc-700 text-[10px] animate-pulse">/</span>
                                <span className="text-[10px] text-green-400 bg-green-500/10 px-2 py-0.5 rounded border border-green-500/20 hover:bg-green-500/20 transition-all cursor-default shadow-[0_0_10px_rgba(59,130,246,0.15)]">{i18n.t('chat.agent_label', { name: targetAgent })}</span>
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
                            onClick={() => onSetScope(scope)}
                            className={clsx(
                                "flex-1 py-1.5 px-2 text-[10px] font-bold uppercase tracking-[0.15em] rounded-md transition-all relative overflow-hidden",
                                activeScope === scope ? "text-zinc-100" : "text-zinc-600 hover:text-zinc-400"
                            )}
                            aria-pressed={activeScope === scope}
                        >
                            {activeScope === scope && (
                                <motion.div layoutId="scope-bg" className="absolute inset-0 bg-zinc-800 border border-zinc-700/50 shadow-inner rounded-md" />
                            )}
                            <span className="relative z-10">{i18n.t(`chat.scope_${scope}`)}</span>
                        </button>
                    ))}
                </div>

                {activeScope !== 'swarm' && (
                    <div className="px-3 pb-2 flex items-center gap-2">
                        {/* Agent Selector */}
                        <div className="relative flex-1 min-w-0">
                            <button
                                onClick={() => {
                                    setOpenDropdown(openDropdown === 'agent' ? null : 'agent');
                                    if (activeScope !== 'agent') onSetScope('agent');
                                }}
                                className={clsx(
                                    "w-full flex items-center justify-between gap-2 text-[10px] font-bold transition-colors uppercase tracking-widest bg-zinc-900/50 px-2 py-1.5 rounded border group",
                                    activeScope === 'agent' ? "border-green-500/50 text-green-400" : "border-zinc-800 text-zinc-500 hover:text-zinc-300"
                                )}
                                aria-haspopup="listbox"
                                aria-expanded={openDropdown === 'agent'}
                                aria-label={i18n.t('chat.select_agent_aria')}
                            >
                                <div className="flex items-center gap-1.5 truncate">
                                    <TargetIcon size={12} className={activeScope === 'agent' ? "text-green-500" : "text-zinc-600"} />
                                    <span className="truncate">{i18n.t('chat.agent_prefix')}<span className={activeScope === 'agent' ? "text-zinc-100" : "text-zinc-400"}>{targetAgent || i18n.t('chat.select_placeholder')}</span></span>
                                </div>
                                <ChevronDown size={12} className={clsx("transition-transform flex-shrink-0", openDropdown === 'agent' && "rotate-180")} />
                            </button>

                            <AnimatePresence>
                                {openDropdown === 'agent' && (
                                    <motion.div
                                        initial={{ opacity: 0, y: -10 }}
                                        animate={{ opacity: 1, y: 0 }}
                                        exit={{ opacity: 0, y: -10 }}
                                        className="absolute left-0 top-full mt-1 w-full min-w-[160px] bg-zinc-900 border border-zinc-800 rounded-lg shadow-2xl z-20 py-1 overflow-y-auto max-h-64 custom-scrollbar backdrop-blur-xl"
                                    >
                                        {(sortedAgents || []).map((agent: Agent) => (
                                            <button
                                                key={agent.id}
                                                onClick={() => {
                                                    setTargetAgent(agent.name);
                                                    setSelectedAgentId(agent.id);
                                                    onSetScope('agent');
                                                    setOpenDropdown(null);
                                                    setInputText(`@${agent.name}: `);
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
                                    setOpenDropdown(openDropdown === 'cluster' ? null : 'cluster');
                                    if (activeScope !== 'cluster') onSetScope('cluster');
                                }}
                                className={clsx(
                                    "w-full flex items-center justify-between gap-2 text-[10px] font-bold transition-colors uppercase tracking-widest bg-zinc-900/50 px-2 py-1.5 rounded border group",
                                    activeScope === 'cluster' ? "border-emerald-500/50 text-emerald-400" : "border-zinc-800 text-zinc-500 hover:text-zinc-300"
                                )}
                                aria-haspopup="listbox"
                                aria-expanded={openDropdown === 'cluster'}
                                aria-label={i18n.t('chat.select_cluster_aria')}
                            >
                                <div className="flex items-center gap-1.5 truncate">
                                    <TargetIcon size={12} className={activeScope === 'cluster' ? "text-emerald-500" : "text-zinc-600"} />
                                    <span className="truncate">{i18n.t('chat.cluster_prefix')}<span className={activeScope === 'cluster' ? "text-zinc-100" : "text-zinc-400"}>{targetCluster || i18n.t('chat.select_placeholder')}</span></span>
                                </div>
                                <ChevronDown size={12} className={clsx("transition-transform flex-shrink-0", openDropdown === 'cluster' && "rotate-180")} />
                            </button>

                            <AnimatePresence>
                                {openDropdown === 'cluster' && (
                                    <motion.div
                                        initial={{ opacity: 0, y: -10 }}
                                        animate={{ opacity: 1, y: 0 }}
                                        exit={{ opacity: 0, y: -10 }}
                                        className="absolute right-0 top-full mt-1 w-full min-w-[160px] bg-zinc-900 border border-zinc-800 rounded-lg shadow-2xl z-20 py-1 overflow-y-auto max-h-64 custom-scrollbar backdrop-blur-xl"
                                    >
                                        {(clusters || []).map(cluster => (
                                            <button
                                                key={cluster.id}
                                                onClick={() => {
                                                    setTargetCluster(cluster.name);
                                                    onSetScope('cluster');
                                                    setOpenDropdown(null);
                                                }}
                                                className="w-full text-left px-3 py-2 text-xs hover:bg-zinc-800 text-zinc-400 hover:text-zinc-100 flex items-center gap-2 transition-colors"
                                            >
                                                <div className={clsx(
                                                    "w-2 h-2 rounded-full flex-shrink-0",
                                                    cluster.theme === 'cyan' ? 'bg-cyan-500' :
                                                        cluster.theme === 'zinc' ? 'bg-zinc-500' :
                                                            cluster.theme === 'amber' ? 'bg-amber-500' : 'bg-green-500'
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
                {showTranscript ? (
                    <Buffered_Transcript_View agent_id={selectedAgentId || undefined} />
                ) : (
                    <Chat_Message_List
                        messages={messages}
                        max_rendered={maxRenderedMessages}
                        active_scope={activeScope}
                        target_node={targetNode}
                    />
                )}
            </div>

            {/* Input Area */}
            <Chat_Input_Bar
                active_scope={activeScope}
                is_safe_mode={isSafeMode}
                is_speech_enabled={isSpeechEnabled}
                is_speaking={isSpeaking}
                input_value={inputText}
                on_change={setInputText}
                on_send={onSend}
                on_toggle_voice={onToggleVoice}
                on_toggle_speech={onToggleSpeech}
                on_toggle_safety={onToggleSafety}
                is_listening={voiceStatus !== 'idle'}
            />
        </div>
    );
};
