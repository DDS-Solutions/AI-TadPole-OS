import React, { useState, useRef, useEffect } from 'react';
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
    VolumeX
} from 'lucide-react';
import clsx from 'clsx';
import { useSettingsStore } from '../stores/settingsStore';
import { useSovereignStore, type SovereignScope } from '../stores/sovereignStore';
import { useAgentStore } from '../stores/agentStore';
import { useWorkspaceStore } from '../stores/workspaceStore';
import { processCommand } from '../logic/commandProcessor';
import { voiceClient } from '../services/voiceClient';
import { useDragControls } from 'framer-motion';
import { useChatWindow } from '../hooks/useChatWindow';
import { Tooltip } from './ui';
import { i18n } from '../i18n';

/**
 * SovereignChat
 * A high-performance, detached-capable chat interface for agent orchestration.
 * Supports triple-scope communication: Agent, Cluster, and Swarm.
 * Enhanced with voice input and context isolation.
 */
export const SovereignChat: React.FC = () => {
    const {
        messages,
        activeScope,
        selectedAgentId,
        targetAgent,
        targetCluster,
        addMessage,
        setScope,
        setSelectedAgentId,
        setTargetAgent,
        setTargetCluster,
        clearHistory
    } = useSovereignStore();

    const { agents } = useAgentStore();
    const { clusters } = useWorkspaceStore();
    const [inputValue, setInputValue] = useState('');
    const [isListening, setIsListening] = useState(false);
    const [isSpeechEnabled, setIsSpeechEnabled] = useState(false);
    const [isSpeaking, setIsSpeaking] = useState(false);
    const { settings, updateSetting } = useSettingsStore();
    const isSafeMode = settings.isSafeMode;
    const [openDropdown, setOpenDropdown] = useState<'agent' | 'cluster' | null>(null);
    const scrollRef = useRef<HTMLDivElement>(null);
    const dragControls = useDragControls();

    const {
        isDetached,
        isMinimized,
        constraintsRef,
        xOpen,
        yOpen,
        xMin,
        yMin,
        setDetached,
        toggleDetach,
        performMinimizeTransform,
        performMaximizeTransform
    } = useChatWindow();

    const targetNode = activeScope === 'cluster' ? targetCluster : targetAgent;

    // Sync targetAgent with selectedAgentId
    useEffect(() => {
        if (selectedAgentId) {
            const agent = agents.find(a => a.id === selectedAgentId);
            if (agent) {
                setTargetAgent(agent.name);
                // Only forced-switch to agent scope if we are currently in a generic/unset state
                if (!targetAgent || targetAgent === 'CEO') {
                    setScope('agent');
                }
            }
        }
    }, [selectedAgentId, agents, setScope, targetAgent, setTargetAgent]);

    // Conservative auto-selection: only if absolutely no target is set and agents exist
    useEffect(() => {
        if (agents.length > 0 && !selectedAgentId && (targetAgent === 'CEO' || !targetAgent)) {
            // Check if CEO actually exists before falling back to index 0
            const ceo = agents.find(a => a.name === 'CEO');
            if (!ceo) {
                setTargetAgent(agents[0].name);
            }
        }
    }, [agents, selectedAgentId, targetAgent, setTargetAgent]);

    // Auto-select first cluster if none selected
    useEffect(() => {
        if (clusters.length > 0 && !targetCluster) {
            setTargetCluster(clusters[0].name);
        }
    }, [clusters, targetCluster, setTargetCluster]);

    useEffect(() => {
        if (scrollRef.current) {
            scrollRef.current.scrollTo({
                top: scrollRef.current.scrollHeight,
                behavior: 'smooth'
            });
        }

        // AUTO-SPEAK LOGIC: If speech is enabled and the last message is from the target agent
        const lastMessage = messages[messages.length - 1];
        if (isSpeechEnabled && lastMessage && lastMessage.senderId !== '0' && lastMessage.senderId === selectedAgentId) {
            const agent = agents.find(a => a.id === selectedAgentId);
            if (agent) {
                setTimeout(() => setIsSpeaking(true), 0);
                voiceClient.speak(lastMessage.text, agent.voiceId, agent.voiceEngine || 'browser').finally(() => {
                    // Note: Browser TTS doesn't give a perfect 'onend' easily in this wrapper without refs
                    // but we can estimate or use a timeout for visual effect
                    setTimeout(() => setIsSpeaking(false), Math.min(10000, lastMessage.text.length * 60));
                });
            }
        }
    }, [messages, isSpeechEnabled, selectedAgentId, agents]);

    useEffect(() => {
        if (agents.length === 0) {
            useAgentStore.getState().fetchAgents();
        }
    }, [agents.length]);
    
    // Header interaction: Toggle between maximized and minimized states
    const handleHeaderClick = () => {
        if (isMinimized) performMaximizeTransform();
        else performMinimizeTransform();
    };

    const handleSend = async (textOverride?: string) => {
        const text = textOverride || inputValue;
        if (!text.trim()) return;

        const userMsg = {
            senderId: '0',
            senderName: i18n.t('chat.overlord_name'),
            text: text,
            scope: activeScope,
            targetNode: activeScope !== 'swarm' ? targetNode : undefined
        };

        addMessage(userMsg);
        if (!textOverride) setInputValue('');

        try {
            // Include target context in command processing
            // @ for Agents, # for Clusters
            const prefix = activeScope === 'agent' ? '@' : activeScope === 'cluster' ? '#' : '';
            // Safely quote the target if we are prepending a prefix
            const command = activeScope === 'swarm' ? text : `"${prefix}${targetNode}" ${text}`;

            await processCommand(command, agents, isSafeMode);
        } catch (err: unknown) {
            const message = err instanceof Error ? err.message : 'Unknown command fault';
            addMessage({
                senderId: 'system',
                senderName: i18n.t('chat.system_name'),
                text: i18n.t('chat.fault_detected', { message }),
                scope: activeScope,
            });
        }
    };

    const toggleVoice = () => {
        if (isListening) {
            voiceClient.stopListening();
            setIsListening(false);
        } else {
            setIsListening(true);
            // Pro-logic: If user starts listening, they likely want to hear the response too
            setIsSpeechEnabled(true);
            voiceClient.startListening((transcript) => {
                setInputValue(transcript);
                // Pro-style auto-send if it's a clear directive
                if (transcript.length > 30 && transcript.endsWith('.')) {
                    // handleSend(transcript);
                }
            });
        }
    };

    // Filter messages based on active scope and target
    const filteredMessages = messages.filter(m => {
        if (activeScope === 'swarm') return true;

        // Agent Isolation:
        if (activeScope === 'agent') {
            const target = targetAgent.toLowerCase();
            return m.scope === 'agent' && (
                m.senderId === '0' || // User messages
                m.senderId === selectedAgentId || // Match by ID
                m.agentId === selectedAgentId || // Match by agentId field
                m.senderName.toLowerCase().includes(target) || // Match by name
                // Special mapping for Agent of Nine / CEO
                ((target.includes('nine') || target.includes('ceo')) &&
                    (m.senderName.toLowerCase().includes('nine') || m.senderName.toLowerCase().includes('ceo') || m.senderId === '1'))
            );
        }

        // Cluster Isolation: Show messages tagged with the cluster target
        if (activeScope === 'cluster') {
            return m.senderId === '0' || m.targetNode === targetNode || m.scope === 'swarm';
        }

        return true;
    });

    if (isDetached && window.location.hash !== '#sovereign-chat') {
        return (
            <div className="fixed bottom-6 right-6 z-50">
                <Tooltip content={i18n.t('chat.restore_tooltip')} position="top">
                    <button
                        onClick={() => setDetached(false)}
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
            {!isDetached && (
                <div ref={constraintsRef} className="fixed inset-x-0 inset-y-0 z-[100] pointer-events-none" style={{ padding: '24px' }} />
            )}
            <AnimatePresence>
                {!isMinimized && (
                    <motion.div
                        key="open-chat"
                        style={{ x: xOpen, y: yOpen }}
                        initial={{ opacity: 0, scale: 0.9, filter: 'blur(10px)' }}
                        animate={{ opacity: 1, scale: 1, filter: 'blur(0px)' }}
                        exit={{ opacity: 0, scale: 0.9, filter: 'blur(10px)' }}
                        drag={!isDetached}
                        dragControls={dragControls}
                        dragListener={false}
                        dragMomentum={false}
                        dragElastic={0}
                        dragConstraints={isDetached ? undefined : constraintsRef}
                        className={clsx(
                            "fixed z-50 flex flex-col overflow-hidden transition-[filter,opacity] duration-300 pointer-events-auto",
                            isDetached
                                ? "inset-0 md:static md:w-full md:h-full bg-zinc-950 border-none pointer-events-auto"
                                : "bottom-6 right-6 w-[400px] h-[600px] rounded-2xl border border-zinc-800/50 shadow-[0_30px_60px_-15px_rgba(0,0,0,0.7)] bg-zinc-900/40 backdrop-blur-xl pointer-events-auto"
                        )}
                    >
                        {/* Neural Background Layer */}
                        {!isDetached && <div className="neural-grid opacity-[0.05] absolute inset-0 pointer-events-none" />}

                        {/* Header */}
                        <div
                            onPointerDown={(e) => {
                                if (!isDetached) {
                                    dragControls.start(e);
                                }
                            }}
                            className={clsx(
                                "relative z-10 p-4 border-b border-zinc-800/50 bg-zinc-950/40 backdrop-blur-md flex items-center justify-between shrink-0 overflow-hidden cursor-pointer select-none",
                                !isDetached && "cursor-grab active:cursor-grabbing"
                            )}
                            onDoubleClick={handleHeaderClick}
                            onKeyDown={(e) => (e.key === 'Enter' || e.key === ' ') && handleHeaderClick()}
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
                                        <div className="h-1 w-1 rounded-full bg-emerald-500 animate-pulse" />
                                        <span className="text-[9px] text-zinc-500 font-mono uppercase tracking-tighter">
                                            {i18n.t(`chat.scope_${activeScope}`)} / {activeScope !== 'swarm' ? targetNode : 'Sovereign_Link'}
                                        </span>
                                        {isSpeaking && (
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
                                <Tooltip content={i18n.t('chat.minimize_tooltip')} position="top">
                                    <button
                                        onClick={(e) => {
                                            e.stopPropagation();
                                            performMinimizeTransform();
                                        }}
                                        className="p-2 text-zinc-500 hover:text-zinc-200 hover:bg-zinc-800/50 rounded-lg transition-colors"
                                        aria-label={i18n.t('chat.minimize_aria')}
                                    >
                                        <Minimize2 size={16} />
                                    </button>
                                </Tooltip>
                                <Tooltip content={i18n.t('chat.detach_tooltip')} position="top">
                                    <button 
                                        onClick={toggleDetach} 
                                        className="p-2 text-zinc-500 hover:text-zinc-200 hover:bg-zinc-800/50 rounded-lg transition-colors"
                                        aria-label={i18n.t('chat.detach_aria')}
                                    >
                                        <ExternalLink size={16} />
                                    </button>
                                </Tooltip>
                                <Tooltip content={i18n.t('chat.close_tooltip')} position="top">
                                    <button 
                                        onClick={clearHistory} 
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
                                            <span className="text-[10px] text-blue-400 bg-blue-500/10 px-2 py-0.5 rounded border border-blue-500/20 hover:bg-blue-500/20 transition-all cursor-default shadow-[0_0_10px_rgba(59,130,246,0.1)]">{i18n.t('chat.agent_label', { name: targetAgent })}</span>
                                        </>
                                    )}
                                </div>
                            </div>
                        )}

                        {/* Scope & Target Selector */}
                        <div className="relative z-20 flex flex-col border-b border-zinc-800/30">
                            <div className="flex p-1.5 bg-zinc-950/20 backdrop-blur-sm gap-1">
                                {(['agent', 'cluster', 'swarm'] as SovereignScope[]).map(scope => (
                                    <button
                                        key={scope}
                                        onClick={() => setScope(scope)}
                                        className={clsx(
                                            "flex-1 py-1.5 px-2 text-[10px] font-bold uppercase tracking-[0.15em] rounded-md transition-all relative overflow-hidden",
                                            activeScope === scope ? "text-zinc-100" : "text-zinc-600 hover:text-zinc-400"
                                        )}
                                        aria-pressed={activeScope === scope}
                                    >
                                        {activeScope === scope && (
                                            <motion.div layoutId="scope-bg" className="absolute inset-0 bg-zinc-800 border border-zinc-700/50 shadow-inner rounded-md" />
                                        )}
                                        <span className="relative z-10">{scope}</span>
                                    </button>
                                ))}
                            </div>

                            {activeScope !== 'swarm' && (
                                <div className="px-3 pb-2 flex items-center gap-2">
                                    {/* Agent Selector */}
                                    <div className="relative flex-1">
                                        <button
                                            onClick={() => {
                                                setOpenDropdown(openDropdown === 'agent' ? null : 'agent');
                                                if (activeScope !== 'agent') setScope('agent');
                                            }}
                                            className={clsx(
                                                "w-full flex items-center justify-between gap-2 text-[10px] font-bold transition-colors uppercase tracking-widest bg-zinc-900/50 px-2 py-1.5 rounded border group",
                                                activeScope === 'agent' ? "border-blue-500/50 text-blue-400" : "border-zinc-800 text-zinc-500 hover:text-zinc-300"
                                            )}
                                            aria-haspopup="listbox"
                                            aria-expanded={openDropdown === 'agent'}
                                            aria-label={i18n.t('chat.select_agent_aria')}
                                        >
                                            <div className="flex items-center gap-1.5 truncate">
                                                <TargetIcon size={12} className={activeScope === 'agent' ? "text-blue-500" : "text-zinc-600"} />
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
                                                    {[...agents].sort((a, b) => {
                                                        const getScore = (status: string) => {
                                                            if (['active', 'thinking', 'coding'].includes(status)) return 0;
                                                            if (status === 'idle') return 1;
                                                            return 2;
                                                        };
                                                        const scoreA = getScore(a.status || 'offline');
                                                        const scoreB = getScore(b.status || 'offline');
                                                        if (scoreA !== scoreB) return scoreA - scoreB;
                                                        return a.name.localeCompare(b.name);
                                                    }).map(agent => (
                                                        <button
                                                            key={agent.id}
                                                            onClick={() => {
                                                                setTargetAgent(agent.name);
                                                                setSelectedAgentId(agent.id);
                                                                setScope('agent');
                                                                setOpenDropdown(null);
                                                            }}
                                                            className="w-full text-left px-3 py-2 text-xs hover:bg-zinc-800 text-zinc-400 hover:text-zinc-100 flex items-center gap-2 transition-colors"
                                                        >
                                                            <div className={clsx("w-2 h-2 rounded-full flex-shrink-0", agent.status === 'offline' ? "opacity-30" : "")} style={{ backgroundColor: agent.themeColor || '#52525b' }} />
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
                                    <div className="relative flex-1">
                                        <button
                                            onClick={() => {
                                                setOpenDropdown(openDropdown === 'cluster' ? null : 'cluster');
                                                if (activeScope !== 'cluster') setScope('cluster');
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
                                                    {clusters.map(cluster => (
                                                        <button
                                                            key={cluster.id}
                                                            onClick={() => {
                                                                setTargetCluster(cluster.name);
                                                                setScope('cluster');
                                                                setOpenDropdown(null);
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
                            ref={scrollRef}
                            className="relative z-10 flex-1 overflow-y-auto p-5 space-y-6 custom-scrollbar"
                        >
                            {filteredMessages.length === 0 && (
                                <div className="h-full flex flex-col items-center justify-center text-zinc-700 space-y-4 opacity-40 text-center">
                                    <Bot size={32} className="neural-pulse" />
                                    <div>
                                        <p className="text-xs font-bold uppercase tracking-[0.3em]">
                                            {activeScope === 'agent' ? i18n.t('chat.isolated_stream', { target: targetNode }) : i18n.t('chat.waiting_packets')}
                                        </p>
                                        <p className="text-[9px] font-mono mt-1 opacity-60">{i18n.t('chat.ready_for_input')}</p>
                                    </div>
                                </div>
                            )}
                            {filteredMessages.map((msg) => (
                                <motion.div
                                    key={msg.id}
                                    initial={{ opacity: 0, x: msg.senderId === '0' ? 10 : -10 }}
                                    animate={{ opacity: 1, x: 0 }}
                                    className={clsx(
                                        "flex flex-col gap-2 max-w-[90%]",
                                        msg.senderId === '0' ? "ml-auto items-end" : "items-start"
                                    )}
                                >
                                    <div className="flex items-center gap-2 px-1">
                                        {msg.senderId !== '0' && (
                                            <Bot size={12} className="text-zinc-500" />
                                        )}
                                        <span className="text-[9px] font-bold text-zinc-500 uppercase tracking-widest">
                                            {msg.senderName}
                                        </span>
                                        {msg.senderId === '0' && (
                                            <User size={12} className="text-zinc-500" />
                                        )}
                                    </div>
                                    <div
                                        className={clsx(
                                            "px-4 py-2.5 rounded-2xl text-[13px] leading-relaxed relative group border transition-all duration-300 overflow-hidden break-words whitespace-pre-wrap",
                                            msg.senderId === '0'
                                                ? "bg-zinc-100 text-zinc-900 rounded-tr-sm shadow-[0_5px_20px_-5px_rgba(255,255,255,0.2)]"
                                                : "bg-zinc-800/80 text-zinc-200 rounded-tl-sm border-zinc-700/50 shadow-xl",
                                            msg.text.length > 800 ? "max-h-[300px] overflow-y-auto custom-scrollbar text-xs" : ""
                                        )}
                                    >
                                        {msg.text}
                                        {msg.scope === 'swarm' && msg.senderId !== '0' && (
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
                        </div>

                        {/* Input Area */}
                        <div className="relative z-10 p-4 bg-zinc-950/50 backdrop-blur-xl border-t border-zinc-800/50">
                            <div className="flex items-center gap-2 bg-black/40 border border-zinc-800/60 rounded-xl p-1 focus-within:border-zinc-500/50 transition-all shadow-inner relative">
                                {/* Transcription Indicator Overlay */}
                                {isListening && (
                                    <div className="absolute inset-0 bg-blue-500/10 animate-pulse pointer-events-none rounded-xl" />
                                )}

                                <Tooltip content={isListening ? i18n.t('chat.stop_listening_tooltip') : i18n.t('chat.start_listening_tooltip')} position="top">
                                    <button
                                        onClick={toggleVoice}
                                        className={clsx(
                                            "p-2.5 rounded-lg transition-all active:scale-90",
                                            isListening ? "bg-red-500 text-white animate-pulse" : "text-zinc-500 hover:text-zinc-100 hover:bg-zinc-800"
                                        )}
                                    >
                                        {isListening ? <MicOff size={18} /> : <Mic size={18} />}
                                    </button>
                                </Tooltip>

                                <Tooltip content={isSpeechEnabled ? i18n.t('chat.mute_output_tooltip') : i18n.t('chat.enable_output_tooltip')} position="top">
                                    <button
                                        onClick={() => setIsSpeechEnabled(!isSpeechEnabled)}
                                        className={clsx(
                                            "p-2.5 rounded-lg transition-all active:scale-90",
                                            isSpeechEnabled ? "text-blue-400 bg-blue-500/10" : "text-zinc-500 hover:text-zinc-100 hover:bg-zinc-800"
                                        )}
                                    >
                                        {isSpeechEnabled ? <Volume2 size={18} className={isSpeaking ? "animate-bounce" : ""} /> : <VolumeX size={18} />}
                                    </button>
                                </Tooltip>

                                <input
                                    type="text"
                                    value={inputValue}
                                    onChange={(e) => setInputValue(e.target.value)}
                                    onKeyDown={(e) => e.key === 'Enter' && handleSend()}
                                    placeholder={isListening ? i18n.t('chat.listening_placeholder') : i18n.t('chat.input_placeholder', { scope: i18n.t(`chat.scope_${activeScope}`) })}
                                    className="flex-1 min-w-0 bg-transparent border-none focus:ring-0 text-sm py-2 px-2 text-zinc-100 placeholder:text-zinc-600 font-medium"
                                    aria-label={i18n.t('chat.input_label')}
                                />

                                <Tooltip content={isSafeMode ? i18n.t('chat.safety_on_tooltip') : i18n.t('chat.safety_off_tooltip')} position="top">
                                    <button
                                        onClick={() => updateSetting('isSafeMode', !isSafeMode)}
                                        className={clsx(
                                            "p-2.5 rounded-lg transition-all active:scale-90",
                                            isSafeMode ? "bg-blue-500/20 text-blue-400 border border-blue-500/30" : "text-zinc-500 hover:text-zinc-100 hover:bg-zinc-800"
                                        )}
                                    >
                                        <BrainCircuit size={18} className={isSafeMode ? "animate-pulse" : ""} />
                                    </button>
                                </Tooltip>

                                <Tooltip content={i18n.t('chat.send_tooltip')} position="top">
                                    <button
                                        onClick={() => handleSend()}
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
                {isMinimized && (
                    <motion.button
                        style={{ x: xMin, y: yMin }}
                        initial={{ scale: 0.8, opacity: 0 }}
                        animate={{ scale: 1, opacity: 1 }}
                        exit={{ scale: 0.8, opacity: 0 }}
                        drag
                        dragConstraints={constraintsRef}
                        dragMomentum={false}
                        dragElastic={0}
                        whileDrag={{ scale: 1.05 }}
                        onClick={() => {
                            // Recover coordinates to dynamic proportionally-anchored frame
                            performMaximizeTransform();
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
