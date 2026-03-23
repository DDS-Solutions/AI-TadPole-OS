import { useReducer, useMemo, useEffect, useState } from 'react';
import { X, Save, Pause, Play, Send, Sliders, ChevronDown, Info, Brain, Trash2, RefreshCw, Cpu, CheckSquare } from 'lucide-react';
import { Tooltip } from './ui';
import { TadpoleOSService, type SkillManifest } from '../services/tadpoleosService';
import { resolveAgentModelConfig, resolveProvider } from '../utils/modelUtils';
import { EventBus } from '../services/eventBus';
import { useProviderStore } from '../stores/providerStore';
import { useModelStore } from '../stores/modelStore';
import { useRoleStore } from '../stores/roleStore';
import { useSkillStore } from '../stores/skillStore';
import { useMemoryStore } from '../stores/memoryStore';
import type { Agent } from '../types';
import { i18n } from '../i18n';

// Master lists calculation moved into component using useMemo

/**
 * Props for the AgentConfigPanel component.
 */
interface AgentConfigPanelProps {
    /** The agent object to configure (optional for new agents) */
    agent?: Agent;
    /** Callback to close the panel */
    onClose: () => void;
    /** Callback to update agent data in the parent state */
    onUpdate: (agentId: string, updates: Partial<Agent>) => void;
    /** Optional: Whether this is a new agent being created */
    isNew?: boolean;
}

import { configReducer } from '../hooks/useAgentForm';

/**
 * AgentConfigPanel
 * 
 * A high-fidelity, slide-out configuration interface for managing the neural
 * state of a specific AI agent node. 
 *
 * Features:
 * - Multi-slot provider/model selection (Primary, Secondary, Tertiary).
 * - Real-time system prompt and temperature adjustments.
 * - Dynamic Skill/Workflow management with blueprint "Promotion" features.
 * - Persistent direct messaging for out-of-band communication with the node.
 *
 * @param props - Component properties including the agent instance and update callbacks.
 */
export default function AgentConfigPanel({ agent, onClose, onUpdate, isNew = false }: AgentConfigPanelProps): JSX.Element {

    const [state, dispatch] = useReducer(configReducer, {
        mainTab: 'cognition',
        activeTab: 'primary',
        identity: {
            name: agent?.name || 'New Neural Node',
            role: agent?.role || 'assistant'
        },
        slots: {
            primary: {
                provider: agent?.modelConfig?.provider || (agent?.model ? resolveProvider(agent.model) : 'google'),
                model: agent?.model || 'gemini-1.5-flash',
                temperature: agent?.modelConfig?.temperature ?? 0.7,
                systemPrompt: agent?.modelConfig?.systemPrompt ?? '',
                skills: agent?.modelConfig?.skills ?? agent?.skills ?? [],
                workflows: agent?.modelConfig?.workflows ?? agent?.workflows ?? []
            },
            secondary: {
                provider: agent?.modelConfig2?.provider || resolveProvider(agent?.model2 ?? 'Claude Opus 4.5'),
                model: agent?.model2 ?? 'Claude Opus 4.5',
                temperature: agent?.modelConfig2?.temperature ?? 0.5,
                systemPrompt: agent?.modelConfig2?.systemPrompt ?? '',
                skills: agent?.modelConfig2?.skills ?? [],
                workflows: agent?.modelConfig2?.workflows ?? []
            },
            tertiary: {
                provider: agent?.modelConfig3?.provider || resolveProvider(agent?.model3 ?? 'LLaMA 4 Maverick'),
                model: agent?.model3 ?? 'LLaMA 4 Maverick',
                temperature: agent?.modelConfig3?.temperature ?? 0.9,
                systemPrompt: agent?.modelConfig3?.systemPrompt ?? '',
                skills: agent?.modelConfig3?.skills ?? [],
                workflows: agent?.modelConfig3?.workflows ?? []
            }
        },
        voice: {
            voiceId: agent?.voiceId || 'alloy',
            voiceEngine: agent?.voiceEngine || 'browser',
            sttEngine: agent?.sttEngine || 'groq'
        },
        ui: {
            directMessage: '',
            saving: false,
            themeColor: agent?.themeColor || '#10b981',
            newRoleName: '',
            showPromote: false
        },
        governance: {
            budgetUsd: agent?.budgetUsd || 0
        },
        mcpTools: agent?.mcpTools || []
    });

    const roles = useRoleStore(s => s.roles);
    const availableRoles = useMemo(() => Object.keys(roles).sort(), [roles]);

    // Connect to global skill store
    const { manifests, scripts, workflows, mcpTools, fetchSkills, fetchMcpTools } = useSkillStore();

    useEffect(() => {
        fetchSkills();
        fetchMcpTools();
    }, [fetchSkills, fetchMcpTools]);

    const allSkills = useMemo(() => {
        const agentCategory = agent?.category || 'user';
        const standardNames = scripts.filter(s => s.category === agentCategory).map(s => s.name);
        const manifestNames = manifests.filter(m => m.category === agentCategory).map(m => m.name);
        const agentAssigned = agent?.skills || [];

        return [...new Set([
            ...standardNames,
            ...manifestNames,
            ...agentAssigned
        ])].sort((a, b) => a.localeCompare(b));
    }, [scripts, manifests, agent?.skills, agent?.category]);

    const allMcpTools = useMemo(() => {
        const agentCategory = agent?.category || 'user';
        const mcpNames = mcpTools.filter(t => t.category === agentCategory).map(t => t.name);
        const agentAssigned = agent?.mcpTools || [];

        return [...new Set([
            ...mcpNames,
            ...agentAssigned
        ])].sort((a, b) => a.localeCompare(b));
    }, [mcpTools, agent?.mcpTools, agent?.category]);

    const allWorkflows = useMemo(() => {
        const agentCategory = agent?.category || 'user';
        const standardNames = workflows.filter(w => w.category === agentCategory).map(w => w.name);
        const agentAssigned = agent?.workflows || [];

        return [...new Set([
            ...standardNames,
            ...agentAssigned
        ])].sort((a, b) => a.localeCompare(b));
    }, [workflows, agent?.workflows, agent?.category]);

    const addRole = useRoleStore(s => s.addRole);
    const { memories, isLoading: isMemoryLoading, fetchMemories, deleteMemory, saveMemory } = useMemoryStore();
    const [memoryInput, setMemoryInput] = useState('');

    const { mainTab, activeTab, identity, slots, ui, governance } = state;
    const currentSlot = slots[activeTab];

    const providers = useProviderStore(s => s.providers);
    const sortedProviders = useMemo(() =>
        [...providers].sort((a, b) => a.name.localeCompare(b.name)),
        [providers]);

    const allModels = useModelStore(s => s.models);
    const filteredModels = useMemo(() =>
        allModels.filter(m => m.provider === currentSlot.provider)
            .sort((a, b) => a.name.localeCompare(b.name)),
        [allModels, currentSlot.provider]);

    useEffect(() => {
        if (mainTab === 'memory' && agent?.id) {
            fetchMemories(agent.id);
        }
    }, [mainTab, agent?.id, fetchMemories]);

    const handleRoleChange = (newRole: string): void => {
        dispatch({ type: 'RESET_ROLE', role: newRole });
    };

    const handleProviderChange = (val: string): void => {
        dispatch({ type: 'UPDATE_SLOT', slot: activeTab, field: 'provider', value: val });
        // Auto-select first model from this provider
        const providerModels = useModelStore.getState().models.filter(m => m.provider === val);
        if (providerModels.length > 0) {
            dispatch({ type: 'UPDATE_SLOT', slot: activeTab, field: 'model', value: providerModels[0].name });
        } else {
            // Clear model selection if no models exist for this provider to prevent stale state
            dispatch({ type: 'UPDATE_SLOT', slot: activeTab, field: 'model', value: '' });
        }
    };

    const handleModelChange = (val: string): void => {
        dispatch({ type: 'UPDATE_SLOT', slot: activeTab, field: 'model', value: val });
    };

    const handleTempChange = (val: number): void => {
        dispatch({ type: 'UPDATE_SLOT', slot: activeTab, field: 'temperature', value: val });
    };

    const handlePromptChange = (val: string): void => {
        dispatch({ type: 'UPDATE_SLOT', slot: activeTab, field: 'systemPrompt', value: val });
    };

    /**
     * Creates a new agent in the global Rust registry.
     * This endpoint handles persistence to agents.json and initiates WS broadcasts.
     * 
     * @param agent - The agent instance to serialize and persist.
     * @returns Promise<boolean> success status.
     */
    const handleSave = async (): Promise<void> => {
        dispatch({ type: 'SET_UI', field: 'saving', value: true });

        try {
            const updates: Partial<Agent> & { provider?: string; provider2?: string; provider3?: string } = {
                role: identity.role,
                name: identity.name,
                budgetUsd: governance.budgetUsd,
                voiceId: state.voice.voiceId,
                voiceEngine: state.voice.voiceEngine
            };

            // Helper to construct model config updates
            const buildConfig = (slotKey: keyof typeof slots) => {
                const slot = slots[slotKey];
                return {
                    modelId: slot.model,
                    provider: slot.provider,
                    temperature: slot.temperature,
                    systemPrompt: slot.systemPrompt,
                    skills: slot.skills,
                    workflows: slot.workflows
                };
            };

            updates.model = slots.primary.model;
            updates.provider = slots.primary.provider;
            updates.modelConfig = buildConfig('primary');

            updates.model2 = slots.secondary.model;
            // provider2 etc aren't in Agent yet but passed in body
            updates.modelConfig2 = buildConfig('secondary');

            updates.model3 = slots.tertiary.model;
            updates.modelConfig3 = buildConfig('tertiary');

            updates.skills = [...new Set([...slots.primary.skills, ...slots.secondary.skills, ...slots.tertiary.skills])];
            updates.workflows = [...new Set([...slots.primary.workflows, ...slots.secondary.workflows, ...slots.tertiary.workflows])];
            updates.themeColor = ui.themeColor;
            updates.budgetUsd = governance.budgetUsd;
            updates.name = identity.name;
            updates.role = identity.role;
            updates.mcpTools = state.mcpTools;
            updates.category = agent?.category || 'user';

            if (agent?.id) {
                onUpdate(agent.id, updates);
            } else {
                // For new agents, we might need a dedicated creation callback
                // but for now let's just use onUpdate with a placeholder or handle in parent
                onUpdate('new', updates);
            }
            onClose();

            EventBus.emit({
                source: 'System',
                text: i18n.t('agent_config.agent_updated', { name: identity.name }),
                severity: 'success'
            });

            setTimeout(onClose, 800);
        } catch (error) {
            console.error('[ConfigPanel] Save Failed:', error);
            EventBus.emit({
                source: 'System',
                text: i18n.t('agent_config.save_failed'),
                severity: 'error'
            });
        } finally {
            dispatch({ type: 'SET_UI', field: 'saving', value: false });
        }
    };

    const handlePause = async (): Promise<void> => {
        if (!agent?.id) return;
        const success = await TadpoleOSService.pauseAgent(agent.id);
        onUpdate(agent.id, { status: 'idle' });
        EventBus.emit({
            source: 'System',
            text: success ? i18n.t('agent_config.agent_paused', { name: identity.name }) : i18n.t('agent_config.agent_paused_locally', { name: identity.name }),
            severity: 'info'
        });
    };

    const handleResume = async (): Promise<void> => {
        if (!agent?.id) return;
        const success = await TadpoleOSService.resumeAgent(agent.id);
        onUpdate(agent.id, { status: 'active' });
        EventBus.emit({
            source: 'System',
            text: success ? i18n.t('agent_config.agent_resumed', { name: identity.name }) : i18n.t('agent_config.agent_resumed_locally', { name: identity.name }),
            severity: 'success'
        });
    };

    const handleSendMessage = async (): Promise<void> => {
        if (!ui.directMessage.trim()) return;
        if (!agent?.id) return;
        const { modelId, provider } = resolveAgentModelConfig(agent);
        await TadpoleOSService.sendCommand(agent.id, ui.directMessage, modelId, provider);
        EventBus.emit({ source: 'User', text: `→ ${identity.name}: ${ui.directMessage}`, severity: 'info' });
        dispatch({ type: 'SET_UI', field: 'directMessage', value: '' });
    };

    const handlePromote = (): void => {
        if (!ui.newRoleName.trim()) {
            EventBus.emit({ text: i18n.t('agent_config.enter_role_name'), severity: 'warning', source: 'System' });
            return;
        }

        addRole(ui.newRoleName, {
            skills: currentSlot.skills,
            workflows: currentSlot.workflows
        });

        EventBus.emit({
            text: i18n.t('agent_config.role_saved', { name: ui.newRoleName }),
            severity: 'success',
            source: 'System'
        });

        dispatch({ type: 'RESET_ROLE', role: ui.newRoleName });
        dispatch({ type: 'SET_UI', field: 'showPromote', value: false });
        dispatch({ type: 'SET_UI', field: 'newRoleName', value: '' });
    };

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 md:p-6">
            <div className="absolute inset-0 bg-black/60 backdrop-blur-sm animate-in fade-in duration-300" onClick={onClose} />

            <div className="relative w-full max-w-2xl bg-zinc-950 border border-zinc-800 rounded-3xl shadow-2xl flex flex-col overflow-hidden animate-in zoom-in-95 duration-300 max-h-[90vh]">
                {/* Header */}
                <div className="p-6 border-b border-zinc-800 bg-zinc-900/80 backdrop-blur-md flex items-start justify-between shrink-0 relative overflow-hidden">
                    <div className="absolute top-0 left-0 w-full h-px bg-gradient-to-r from-transparent via-emerald-500/20 to-transparent" />

                    <div className="flex items-start gap-4 z-10">
                        <div className="relative group/picker">
                            <Tooltip content="Select a custom color profile for this agent node's signature." position="top">
                                <div
                                    className="p-3 rounded-xl border transition-all duration-300 relative overflow-hidden"
                                    style={{
                                        backgroundColor: `${ui.themeColor}15`,
                                        borderColor: `${ui.themeColor}40`,
                                        boxShadow: `0 0 20px ${ui.themeColor}10`
                                    }}
                                >
                                    <Sliders size={20} style={{ color: ui.themeColor }} />
                                    <input
                                        type="color"
                                        value={ui.themeColor}
                                        onChange={(e) => dispatch({ type: 'SET_UI', field: 'themeColor', value: e.target.value })}
                                        className="absolute inset-0 opacity-0 cursor-pointer w-full h-full"
                                        aria-label={i18n.t('agent_config.aria_theme_color')}
                                    />
                                </div>
                            </Tooltip>
                            {/* Visual Indicator of HEX */}
                            <div className="absolute -bottom-1 -right-1 w-3 h-3 rounded-full border border-black/50 shadow-sm" style={{ backgroundColor: ui.themeColor }} />
                        </div>
                        <div className="space-y-1">
                            <h2 className="text-[11px] font-bold text-emerald-500 tracking-[0.2em] uppercase opacity-80">
                                {isNew ? i18n.t('agent_config.init_new') : i18n.t('agent_config.init_config')}
                            </h2>
                                <input
                                    value={identity.name}
                                    onChange={(e) => dispatch({ type: 'UPDATE_IDENTITY', field: 'name', value: e.target.value })}
                                    className="bg-transparent border-none p-0 font-bold text-zinc-100 text-xl leading-tight focus:ring-0 w-full hover:bg-white/5 rounded px-1 -ml-1 transition-colors"
                                    spellCheck={false}
                                    aria-label={i18n.t('agent_config.aria_agent_name')}
                                />
                            <div className="flex items-center gap-3 pt-1">
                                <div className="relative group/role">
                                    <select
                                        value={identity.role}
                                        onChange={(e) => handleRoleChange(e.target.value)}
                                        aria-label={i18n.t('agent_config.aria_role_selector')}
                                        className="appearance-none bg-zinc-900/80 border border-zinc-700/50 rounded px-2 py-0.5 text-xs font-bold text-zinc-300 uppercase tracking-widest cursor-pointer hover:border-emerald-500/50 hover:text-emerald-400 transition-all focus:outline-none pr-6"
                                    >
                                        {availableRoles.map(r => (
                                            <option key={r} value={r} className="bg-zinc-900">{r.toUpperCase()}</option>
                                        ))}
                                    </select>
                                    <ChevronDown size={10} className="absolute right-1.5 top-1/2 -translate-y-1/2 text-zinc-600 group-hover/role:text-emerald-400 pointer-events-none" />
                                </div>
                                <span className="text-[11px] text-zinc-500 font-mono tracking-tighter opacity-50">
                                    {agent?.id ? i18n.t('agent_config.neural_node_id', { id: agent.id.substring(0, 8).toUpperCase() }) : i18n.t('agent_config.id_pending')}
                                </span>
                            </div>
                        </div>
                    </div>

                    <button onClick={onClose} className="p-3 w-11 h-11 flex items-center justify-center rounded-lg hover:bg-zinc-800 transition-colors text-zinc-500 hover:text-white z-10 shrink-0" aria-label="Close configuration panel">
                        <X size={20} />
                    </button>
                </div>

                {/* Top-Level Section Toggles */}
                <div className="flex border-b border-zinc-800 bg-[#0a0a0c]">
                    <Tooltip content={i18n.t('agent_config.tooltip_cognition')} position="bottom" className="flex-1">
                        <button
                            onClick={() => dispatch({ type: 'SET_MAIN_TAB', payload: 'cognition' })}
                            className={`w-full py-3 text-[11px] uppercase font-bold tracking-widest transition-colors relative flex items-center justify-center gap-2 ${mainTab === 'cognition' ? 'text-zinc-100 bg-zinc-900' : 'text-zinc-500 hover:text-zinc-300 hover:bg-zinc-800/50'}`}
                        >
                            <Cpu size={14} className={mainTab === 'cognition' ? 'text-emerald-500' : ''} />
                            {i18n.t('agent_config.tab_cognition')}
                            {mainTab === 'cognition' && <div className="absolute bottom-0 left-0 right-0 h-0.5 bg-emerald-500" />}
                        </button>
                    </Tooltip>
                    <Tooltip content={!agent?.id ? i18n.t('agent_config.memory_unavailable') : i18n.t('agent_config.tooltip_memory')} position="bottom" className="flex-1">
                        <button
                            onClick={() => agent?.id && dispatch({ type: 'SET_MAIN_TAB', payload: 'memory' })}
                            disabled={!agent?.id}
                            className={`w-full py-3 text-[11px] uppercase font-bold tracking-widest transition-colors relative flex items-center justify-center gap-2 ${!agent?.id ? 'opacity-30 cursor-not-allowed' : (mainTab === 'memory' ? 'text-zinc-100 bg-zinc-900' : 'text-zinc-500 hover:text-zinc-300 hover:bg-zinc-800/50')}`}
                        >
                            <Brain size={14} className={mainTab === 'memory' && agent?.id ? 'text-blue-500' : ''} />
                            {i18n.t('agent_config.tab_memory')}
                            {mainTab === 'memory' && agent?.id && <div className="absolute bottom-0 left-0 right-0 h-0.5 bg-blue-500 shadow-[0_0_10px_blue]" />}
                        </button>
                    </Tooltip>
                </div>

                {mainTab === 'cognition' ? (
                    <>
                        {/* Tab Bar */}
                        <div className="flex border-b border-zinc-800 bg-[#121215]">
                            {(['primary', 'secondary', 'tertiary'] as const).map(tab => (
                                <Tooltip
                                    key={tab}
                                    content={
                                        tab === 'primary' ? i18n.t('agent_config.tooltip_primary') :
                                            tab === 'secondary' ? i18n.t('agent_config.tooltip_secondary') :
                                                i18n.t('agent_config.tooltip_tertiary')
                                    }
                                    position="bottom"
                                    className="flex-1"
                                >
                                    <button
                                        onClick={() => dispatch({ type: 'SET_TAB', payload: tab })}
                                        className={`w-full py-3 text-xs uppercase font-bold tracking-wider transition-colors relative ${activeTab === tab ? 'text-zinc-100 bg-zinc-900' : 'text-zinc-500 hover:text-zinc-300 hover:bg-zinc-800/50'}`}
                                    >
                                        {tab}
                                        {activeTab === tab && <div className="absolute bottom-0 left-0 right-0 h-0.5 bg-blue-500" />}
                                    </button>
                                </Tooltip>
                            ))}
                        </div>

                        {/* Content */}
                        <div className="flex-1 overflow-y-auto p-4 space-y-6 custom-scrollbar">
                            <div className="px-4 py-2 border-l-2 border-emerald-500/30 bg-emerald-500/5 rounded-r-lg">
                                <p className="text-[11px] text-emerald-500/70 leading-relaxed italic">
                                    {i18n.t('agent_config.personality_derived')}
                                </p>
                            </div>

                            <div className="flex items-center justify-between">
                                <div className="flex items-center gap-2">
                                    <div className={`w-2 h-2 rounded-full ${agent?.status === 'active' ? 'bg-green-500 animate-pulse' : 'bg-zinc-600'}`} />
                                    <span className="text-xs text-zinc-400 font-mono uppercase">{agent?.status || i18n.t('agent_config.status_initializing')}</span>
                                </div>
                                <div className="flex gap-2">
                                    {!agent ? (
                                        <div className="text-[10px] text-zinc-500 italic">{i18n.t('agent_config.save_to_init')}</div>
                                    ) : (agent.status === 'active' || agent.status === 'thinking' || agent.status === 'working' || agent.status === 'speaking' || agent.status === 'coding') ? (
                                        <Tooltip content={i18n.t('agent_config.tooltip_pause')} position="left">
                                            <button onClick={handlePause} className="flex items-center justify-center min-h-[44px] gap-1.5 px-4 py-2 rounded-lg bg-yellow-500/10 text-yellow-400 text-xs font-bold hover:bg-yellow-500/20">
                                                <Pause size={12} /> {i18n.t('agent_config.btn_pause')}
                                            </button>
                                        </Tooltip>
                                    ) : (
                                        <Tooltip content={i18n.t('agent_config.tooltip_resume')} position="left">
                                            <button onClick={handleResume} className="flex items-center justify-center min-h-[44px] gap-1.5 px-4 py-2 rounded-lg bg-emerald-500/10 text-emerald-400 text-xs font-bold hover:bg-emerald-500/20">
                                                <Play size={12} /> {i18n.t('agent_config.btn_resume')}
                                            </button>
                                        </Tooltip>
                                    )}
                                </div>
                            </div>

                            <div className="pt-4 border-t border-zinc-800/50">
                                <label className="block text-[10px] font-bold text-zinc-500 uppercase tracking-widest mb-2 flex items-center gap-2">
                                    {i18n.t('agent_config.voice_integration')}
                                    <Tooltip content={i18n.t('agent_config.tooltip_voice_engine')} position="top">
                                        <Info size={10} className="text-zinc-700 hover:text-emerald-500 cursor-help transition-colors" />
                                    </Tooltip>
                                </label>
                                <div className="grid grid-cols-2 gap-3 bg-zinc-900/50 border border-zinc-800 rounded-lg p-3">
                                    <div className="space-y-1.5">
                                        <label className="block text-[9px] font-bold text-zinc-600 uppercase opacity-60">{i18n.t('agent_config.label_engine')}</label>
                                        <select
                                            value={state.voice.voiceEngine}
                                            onChange={(e) => dispatch({ type: 'UPDATE_VOICE', field: 'voiceEngine', value: e.target.value as 'browser' | 'openai' | 'groq' })}
                                            aria-label={i18n.t('agent_config.aria_voice_engine')}
                                            className="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-2 text-xs text-zinc-200 focus:outline-none focus:border-emerald-500/50 appearance-none font-bold cursor-pointer"
                                        >
                                            <option value="browser">{i18n.t('agent_config.label_browser_local')}</option>
                                            <option value="piper">PIPER (LOCAL)</option>
                                            <option value="openai">OPENAI (HI-FI)</option>
                                            <option value="groq">{i18n.t('agent_config.label_groq_fast')}</option>
                                        </select>
                                    </div>
                                    <div className="space-y-1.5">
                                        <label className="block text-[9px] font-bold text-zinc-600 uppercase opacity-60 flex items-center gap-1.5">
                                            {i18n.t('agent_config.label_vocal_identity')}
                                            <Tooltip content={i18n.t('agent_config.tooltip_vocal_identity')} position="top">
                                                <Info size={9} className="text-zinc-700 hover:text-emerald-500 cursor-help transition-colors" />
                                            </Tooltip>
                                        </label>
                                        {state.voice.voiceEngine === 'openai' ? (
                                            <select
                                                value={state.voice.voiceId}
                                                onChange={(e) => dispatch({ type: 'UPDATE_VOICE', field: 'voiceId', value: e.target.value })}
                                                aria-label={i18n.t('agent_config.aria_voice_id')}
                                                className="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-2 text-xs text-zinc-200 focus:outline-none focus:border-emerald-500/50 appearance-none font-bold cursor-pointer"
                                            >
                                                <option value="alloy">ALLOY (NEUTRAL)</option>
                                                <option value="echo">{i18n.t('agent_config.label_echo_deep')}</option>
                                                <option value="fable">FABLE (BRITISH)</option>
                                                <option value="onyx">ONYX (AUTHORITATIVE)</option>
                                                <option value="nova">NOVA (ENERGIZED)</option>
                                                <option value="shimmer">SHIMMER (SOFT)</option>
                                            </select>
                                        ) : (
                                            <input
                                                type="text"
                                                placeholder={i18n.t('agent_config.placeholder_voice_id')}
                                                value={state.voice.voiceId}
                                                onChange={(e) => dispatch({ type: 'UPDATE_VOICE', field: 'voiceId', value: e.target.value })}
                                                className="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-2 text-xs text-zinc-200 focus:outline-none focus:border-emerald-500/50 font-mono"
                                                aria-label={i18n.t('agent_config.aria_voice_id')}
                                            />
                                        )}
                                    </div>
                                </div>
                            </div>

                            <div className="pt-4 border-t border-zinc-800/50">
                                <label className="block text-[10px] font-bold text-zinc-500 uppercase tracking-widest mb-2 flex items-center gap-2">
                                    {i18n.t('agent_config.stt_integration')}
                                    <Tooltip content={i18n.t('agent_config.tooltip_stt')} position="top">
                                        <Info size={10} className="text-zinc-700 hover:text-blue-500 cursor-help transition-colors" />
                                    </Tooltip>
                                </label>
                                <div className="bg-zinc-900/50 border border-zinc-800 rounded-lg p-3">
                                    <div className="space-y-1.5">
                                        <label className="block text-[9px] font-bold text-zinc-600 uppercase opacity-60">{i18n.t('agent_config.label_engine')}</label>
                                        <select
                                            value={state.voice.sttEngine || 'groq'}
                                            onChange={(e) => dispatch({ type: 'UPDATE_VOICE', field: 'sttEngine', value: e.target.value })}
                                            aria-label={i18n.t('agent_config.aria_stt_engine')}
                                            className="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-2 text-xs text-zinc-200 focus:outline-none focus:border-blue-500/50 appearance-none font-bold cursor-pointer"
                                        >
                                            <option value="groq">{i18n.t('agent_config.label_groq_ultra')}</option>
                                            <option value="whisper">WHISPER (LOCAL - PRIVATE)</option>
                                        </select>
                                    </div>
                                </div>
                            </div>

                            <div className="space-y-5 animate-in fade-in duration-300" key={activeTab}>
                                <div className="grid grid-cols-2 gap-3">
                                    <div className="space-y-1.5">
                                        <label className="block text-[10px] font-bold text-zinc-500 uppercase tracking-widest flex items-center gap-1.5">
                                            {i18n.t('agent_config.label_provider', { slot: activeTab })}
                                            <Tooltip content={i18n.t('agent_config.tooltip_provider')} position="top">
                                                <Info size={9} className="text-zinc-700 hover:text-blue-500 cursor-help transition-colors" />
                                            </Tooltip>
                                        </label>
                                        <select
                                            value={currentSlot.provider}
                                            onChange={(e) => handleProviderChange(e.target.value)}
                                            aria-label={i18n.t('agent_config.aria_provider_selector')}
                                            className="w-full bg-zinc-900 border border-zinc-800 rounded-lg px-3 py-2 text-xs text-zinc-200 focus:outline-none focus:border-blue-500 cursor-pointer font-bold appearance-none"
                                        >
                                            {sortedProviders.map(p => (
                                                <option key={p.id} value={p.id} className="bg-zinc-950">
                                                    {p.name.toUpperCase()}
                                                </option>
                                            ))}
                                        </select>
                                    </div>

                                    <div className="space-y-1.5">
                                        <label className="block text-[10px] font-bold text-zinc-500 uppercase tracking-widest flex items-center gap-1.5">
                                            {i18n.t('agent_config.label_model')}
                                            <Tooltip content={i18n.t('agent_config.tooltip_model')} position="top">
                                                <Info size={9} className="text-zinc-700 hover:text-blue-500 cursor-help transition-colors" />
                                            </Tooltip>
                                        </label>
                                        <select
                                            value={currentSlot.model}
                                            onChange={(e) => handleModelChange(e.target.value)}
                                            aria-label={i18n.t('agent_config.aria_model_selector')}
                                            className="w-full bg-zinc-900 border border-zinc-800 rounded-lg px-3 py-2 text-xs text-zinc-200 focus:outline-none focus:border-blue-500 font-mono cursor-pointer appearance-none"
                                        >
                                            {filteredModels.map(m => (
                                                <option key={m.id} value={m.name} className="bg-zinc-950">
                                                    [{m.modality?.toUpperCase() || 'LLM'}] {m.name}
                                                </option>
                                            ))}
                                        </select>
                                    </div>
                                </div>

                                <div>
                                    <label className="block text-xs font-bold text-zinc-500 uppercase tracking-wider mb-1.5 flex items-center gap-1.5">
                                        {i18n.t('agent_config.label_temperature')} <span className="text-blue-400 font-mono">{currentSlot.temperature.toFixed(2)}</span>
                                        <Tooltip content={i18n.t('agent_config.tooltip_temperature')} position="top">
                                            <Info size={10} className="text-zinc-700 hover:text-blue-400 cursor-help transition-colors" />
                                        </Tooltip>
                                    </label>
                                    <input type="range" min="0" max="2" step="0.05" value={currentSlot.temperature} onChange={(e) => handleTempChange(parseFloat(e.target.value))} className="w-full accent-blue-500" aria-label={i18n.t('agent_config.aria_temperature')} />
                                </div>

                                <div>
                                    <label className="block text-xs font-bold text-zinc-500 uppercase tracking-wider mb-1.5 flex items-center gap-1.5">
                                        {i18n.t('agent_config.label_system_prompt', { slot: activeTab })}
                                        <Tooltip content={i18n.t('agent_config.tooltip_system_prompt')} position="top">
                                            <Info size={10} className="text-zinc-700 hover:text-blue-400 cursor-help transition-colors" />
                                        </Tooltip>
                                    </label>
                                    <textarea
                                        value={currentSlot.systemPrompt}
                                        onChange={(e) => handlePromptChange(e.target.value)}
                                        aria-label={i18n.t('agent_config.aria_system_prompt')}
                                        rows={6}
                                        className="w-full bg-zinc-900 border border-zinc-800 rounded-lg px-3 py-2 text-xs text-zinc-200 focus:outline-none focus:border-blue-500 font-mono resize-none custom-scrollbar"
                                    />
                                </div>

                                <div className="pt-4 border-t border-zinc-800/50">
                                    <div className="flex items-center justify-between mb-2">
                                        <label className="block text-[10px] font-bold text-zinc-500 uppercase tracking-widest flex items-center gap-1.5">
                                            {i18n.t('agent_config.label_budget')}
                                            <Tooltip content={i18n.t('agent_config.tooltip_budget')} position="top">
                                                <Info size={9} className="text-zinc-700 hover:text-emerald-500 cursor-help transition-colors" />
                                            </Tooltip>
                                        </label>
                                        <div className="text-[10px] font-mono text-zinc-400">
                                            {i18n.t('agent_config.current_spend')} <span className="text-emerald-400">${agent?.costUsd?.toFixed(4) || '0.0000'}</span>
                                        </div>
                                    </div>
                                    <div className="flex items-center gap-3 bg-zinc-900/50 border border-zinc-800 rounded-lg p-3">
                                        <div className="flex-1 space-y-1">
                                            <span className="text-[11px] text-zinc-500 uppercase font-bold tracking-tighter">{i18n.t('agent_config.budget_limit')}</span>
                                            <div className="flex items-center gap-2">
                                                <span className="text-zinc-500 font-mono text-xs">{i18n.t('agent_config.label_currency_symbol')}</span>
                                                <input
                                                    type="number"
                                                    step="0.01"
                                                    value={governance.budgetUsd}
                                                    onChange={(e) => dispatch({ type: 'UPDATE_GOVERNANCE', field: 'budgetUsd', value: parseFloat(e.target.value) || 0 })}
                                                    className="bg-transparent border-none p-0 text-sm font-bold text-zinc-100 focus:ring-0 w-full"
                                                    aria-label={i18n.t('agent_config.aria_budget_limit')}
                                                />
                                            </div>
                                        </div>
                                        <div className="h-8 w-px bg-zinc-800" />
                                        <div className="flex-1 space-y-1">
                                            <span className="text-[11px] text-zinc-500 uppercase font-bold tracking-tighter">{i18n.t('agent_config.label_status')}</span>
                                            <div className="flex items-center gap-1.5">
                                                {(agent?.costUsd || 0) >= (governance.budgetUsd || 0) && (governance.budgetUsd || 0) > 0 ? (
                                                    <>
                                                        <div className="w-1.5 h-1.5 rounded-full bg-red-500 shadow-[0_0_8px_red]" />
                                                        <span className="text-[10px] font-bold text-red-500 uppercase tracking-tighter">{i18n.t('agent_config.status_breached')}</span>
                                                    </>
                                                ) : (
                                                    <>
                                                        <div className="w-1.5 h-1.5 rounded-full bg-emerald-500 shadow-[0_0_8px_emerald]" />
                                                        <span className="text-[10px] font-bold text-emerald-500 uppercase tracking-tighter">{i18n.t('agent_config.status_nominal')}</span>
                                                    </>
                                                )}
                                            </div>
                                        </div>
                                    </div>
                                </div>

                                <div className="grid grid-cols-2 gap-4 pt-4 border-t border-zinc-800/50">
                                    <div className="flex flex-col h-48">
                                        <label className="block text-xs font-bold text-zinc-500 uppercase tracking-wider mb-2 flex items-center gap-1.5">
                                            {i18n.t('agent_config.label_skills', { count: currentSlot.skills.length })}
                                            <Tooltip content={i18n.t('agent_config.tooltip_skills')} position="top">
                                                <Info size={10} className="text-zinc-700 hover:text-emerald-500 cursor-help transition-colors" />
                                            </Tooltip>
                                        </label>
                                        <div className="flex-1 overflow-y-auto custom-scrollbar border border-zinc-800 rounded-lg bg-zinc-900/30 p-2 space-y-2">
                                            {allSkills.map(skillName => {
                                                const manifest = manifests.find((m: SkillManifest) => m.name === skillName);
                                                const mcpTool = mcpTools.find(t => t.name === skillName);
                                                const isSelected = currentSlot.skills.includes(skillName as string);

                                                // Colors based on danger level or source
                                                const getDangerColor = (level?: string) => {
                                                    if (level === 'critical') return 'text-red-500 bg-red-500/10 border-red-500/20';
                                                    if (level === 'high') return 'text-orange-500 bg-orange-500/10 border-orange-500/20';
                                                    if (level === 'medium') return 'text-yellow-500 bg-yellow-500/10 border-yellow-500/20';
                                                    return 'text-emerald-500 bg-emerald-500/10 border-emerald-500/20'; // Low or MCP
                                                };
                                                const dangerClasses = getDangerColor(manifest?.danger_level);

                                                return (
                                                    <div
                                                        key={skillName}
                                                        role="button"
                                                        tabIndex={0}
                                                        onClick={() => dispatch({ type: 'TOGGLE_SKILL', slot: activeTab, kind: 'skills', value: skillName as string })}
                                                        onKeyDown={(e) => (e.key === 'Enter' || e.key === ' ') && dispatch({ type: 'TOGGLE_SKILL', slot: activeTab, kind: 'skills', value: skillName as string })}
                                                        className={`w-full flex flex-col p-2.5 rounded-lg border text-left cursor-pointer transition-all ${isSelected ? 'bg-zinc-800 border-zinc-600' : 'bg-transparent border-transparent hover:bg-zinc-800/50'}`}
                                                    >
                                                        <div className="flex items-start justify-between gap-2 overflow-hidden">
                                                            <div className="flex items-center gap-2 min-w-0">
                                                                <div className={`w-3 h-3 rounded-full border shrink-0 ${isSelected ? 'bg-emerald-500 border-emerald-500' : 'border-zinc-600 bg-zinc-950'}`} />
                                                                <span className={`text-xs font-bold truncate ${isSelected ? 'text-zinc-100' : 'text-zinc-400'}`}>
                                                                    {manifest?.display_name || skillName}
                                                                </span>
                                                            </div>
                                                            <div className="flex items-center gap-1.5 shrink-0">
                                                                {mcpTool && (
                                                                    <span className="text-[8px] font-bold px-1 py-0.5 rounded bg-blue-500/10 text-blue-400 border border-blue-500/20 uppercase tracking-tighter">{i18n.t('agent_config.label_mcp')}</span>
                                                                )}
                                                                {manifest ? (
                                                                    <span className={`text-[9px] font-mono px-1.5 py-0.5 rounded border uppercase tracking-tighter ${dangerClasses}`}>
                                                                        {manifest.danger_level}
                                                                    </span>
                                                                ) : !mcpTool && (
                                                                    <span className="text-[8px] font-bold px-1 py-0.5 rounded bg-zinc-800 text-zinc-500 border border-zinc-700 uppercase tracking-tighter">{i18n.t('agent_config.label_core')}</span>
                                                                )}
                                                            </div>
                                                        </div>
                                                        {(manifest?.description || mcpTool?.description) && (
                                                            <div className="mt-1.5 pl-5">
                                                                <p className="text-[10px] text-zinc-500/60 leading-tight mb-1">
                                                                    {manifest?.description || mcpTool?.description}
                                                                </p>
                                                                {manifest?.requires_oversight && (
                                                                    <p className="text-[9px] text-amber-500 font-mono tracking-tighter mt-1 bg-amber-500/10 px-1 py-0.5 rounded inline-block">
                                                                        {i18n.t('agent_config.requires_oversight')}
                                                                    </p>
                                                                )}
                                                            </div>
                                                        )}
                                                    </div>
                                                );
                                            })}
                                        </div>
                                    </div>
                                    <div className="flex flex-col h-48">
                                        <label className="block text-xs font-bold text-zinc-500 uppercase tracking-wider mb-2 flex items-center gap-1.5">
                                            {i18n.t('agent_config.label_workflows', { count: currentSlot.workflows.length })}
                                            <Tooltip content={i18n.t('agent_config.tooltip_workflows')} position="top">
                                                <Info size={10} className="text-zinc-700 hover:text-amber-500 cursor-help transition-colors" />
                                            </Tooltip>
                                        </label>
                                        <div className="flex-1 overflow-y-auto custom-scrollbar border border-zinc-800 rounded-lg bg-zinc-900/30 p-2 space-y-1">
                                            {allWorkflows.map(wf => (
                                                <button
                                                    key={wf}
                                                    onClick={() => dispatch({ type: 'TOGGLE_SKILL', slot: activeTab, kind: 'workflows', value: wf as string })}
                                                    className={`w-full flex items-center min-h-[44px] gap-2 p-2 rounded text-xs transition-colors ${currentSlot.workflows.includes(wf as string) ? 'bg-amber-500/10 text-amber-400' : 'text-zinc-400 hover:bg-zinc-800'}`}
                                                >
                                                    <div className={`w-2 h-2 rounded-full border ${currentSlot.workflows.includes(wf as string) ? 'bg-amber-500 border-amber-500' : 'border-zinc-600'}`} />
                                                    {wf}
                                                </button>
                                            ))}
                                        </div>
                                    </div>
                                </div>
                                <div className="pt-4 border-t border-zinc-800/50">
                                    <label className="block text-xs font-bold text-zinc-500 uppercase tracking-wider mb-2 flex items-center gap-1.5">
                                        {i18n.t('agent_config.label_mcp_tools', { count: state.mcpTools.length })}
                                        <Tooltip content={i18n.t('agent_config.tooltip_mcp_tools')} position="top">
                                            <Info size={10} className="text-zinc-700 hover:text-cyan-500 cursor-help transition-colors" />
                                        </Tooltip>
                                    </label>
                                    <div className="flex-1 overflow-y-auto custom-scrollbar border border-zinc-800 rounded-lg bg-zinc-900/30 p-2 grid grid-cols-2 gap-2">
                                        {allMcpTools.map(toolName => {
                                            const isSelected = state.mcpTools.includes(toolName as string);
                                            const mcpTool = mcpTools.find(t => t.name === toolName);
                                            
                                            return (
                                                <div
                                                    key={toolName}
                                                    role="button"
                                                    tabIndex={0}
                                                    onClick={() => dispatch({ type: 'TOGGLE_MCP_TOOL', value: toolName as string })}
                                                    onKeyDown={(e) => (e.key === 'Enter' || e.key === ' ') && dispatch({ type: 'TOGGLE_MCP_TOOL', value: toolName as string })}
                                                    className={`flex flex-col p-2 rounded-lg border text-left cursor-pointer transition-all ${isSelected ? 'bg-cyan-500/10 border-cyan-500/30' : 'bg-transparent border-transparent hover:bg-zinc-800/50'}`}
                                                >
                                                    <div className="flex items-center gap-2 overflow-hidden">
                                                        <div className={`w-2.5 h-2.5 rounded-full border shrink-0 ${isSelected ? 'bg-cyan-500 border-cyan-500' : 'border-zinc-600 bg-zinc-950'}`} />
                                                        <span className={`text-[11px] font-bold truncate ${isSelected ? 'text-cyan-400' : 'text-zinc-400'}`}>
                                                            {toolName}
                                                        </span>
                                                    </div>
                                                    {mcpTool?.description && (
                                                        <p className="mt-1 pl-4 text-[9px] text-zinc-500/60 leading-tight line-clamp-1">
                                                            {mcpTool.description}
                                                        </p>
                                                    )}
                                                </div>
                                            );
                                        })}
                                    </div>
                                </div>
                            </div>

                            <div className="pt-4 border-t border-zinc-800/50">
                                <label className="block text-xs font-bold text-zinc-500 uppercase tracking-wider mb-1.5">{i18n.t('agent_config.inject_message')}</label>
                                <div className="flex gap-2">
                                    <input
                                        type="text"
                                        value={ui.directMessage}
                                        onChange={(e) => dispatch({ type: 'SET_UI', field: 'directMessage', value: e.target.value })}
                                        onKeyDown={(e) => e.key === 'Enter' && handleSendMessage()}
                                        placeholder={i18n.t('agent_config.placeholder_instruction')}
                                        className="flex-1 bg-zinc-900 border border-zinc-800 rounded-lg px-3 py-2 text-xs text-zinc-100 focus:outline-none focus:border-emerald-500/50"
                                        aria-label={i18n.t('agent_config.aria_direct_message')}
                                    />
                                    <Tooltip content={i18n.t('agent_config.tooltip_inject')} position="left">
                                        <button 
                                            onClick={handleSendMessage} 
                                            className="p-3 w-11 h-11 flex items-center justify-center rounded-lg bg-blue-500/10 text-blue-400"
                                            aria-label={i18n.t('agent_config.aria_send_message')}
                                        >
                                            <Send size={14} />
                                        </button>
                                    </Tooltip>
                                </div>
                            </div>

                            <div className="pt-4 border-t border-zinc-800/50 pb-4">
                                <div className="flex items-center justify-between mb-3">
                                    <label className="block text-xs font-bold text-zinc-500 uppercase tracking-wider flex items-center gap-2">
                                        {i18n.t('agent_config.role_blueprint')}
                                        <Tooltip content={i18n.t('agent_config.tooltip_role_blueprint')} position="top">
                                            <Info size={11} className="text-zinc-700 hover:text-blue-400 cursor-help transition-colors" />
                                        </Tooltip>
                                    </label>
                                    <button
                                        onClick={() => dispatch({ type: 'SET_UI', field: 'showPromote', value: !ui.showPromote })}
                                        className="text-[10px] text-blue-400 hover:text-blue-300 font-bold uppercase min-h-[44px] min-w-[44px] flex items-center justify-center p-2"
                                    >
                                        {ui.showPromote ? i18n.t('agent_config.btn_cancel') : i18n.t('agent_config.btn_promote')}
                                    </button>
                                </div>

                                {ui.showPromote ? (
                                    <div className="space-y-3 bg-blue-500/5 border border-blue-500/20 p-4 rounded-xl animate-in zoom-in-95 duration-200">
                                        <p className="text-[10px] text-blue-400/80 leading-tight">
                                            {i18n.t('agent_config.promote_desc')}
                                        </p>
                                        <div className="flex gap-2">
                                            <input
                                                type="text"
                                                value={ui.newRoleName}
                                                onChange={(e) => dispatch({ type: 'SET_UI', field: 'newRoleName', value: e.target.value })}
                                                placeholder={i18n.t('agent_config.placeholder_role_name')}
                                                className="flex-1 bg-black/40 border border-blue-500/30 rounded-lg px-3 py-2 text-xs text-blue-100 focus:outline-none focus:border-blue-400 placeholder:text-blue-900/50"
                                                aria-label={i18n.t('agent_config.aria_new_role_name')}
                                            />
                                            <button
                                                onClick={handlePromote}
                                                className="px-4 py-2 min-h-[44px] min-w-[44px] bg-blue-600 hover:bg-blue-500 text-white text-[10px] font-bold rounded-lg transition-all"
                                                aria-label={i18n.t('agent_config.aria_promote_action')}
                                            >
                                                {i18n.t('agent_config.btn_promote_action')}
                                            </button>
                                        </div>
                                    </div>
                                ) : (
                                    <div className="text-[9px] text-zinc-600 italic">
                                        {i18n.t('agent_config.promote_hint')}
                                    </div>
                                )}
                            </div>
                        </div >
                    </>
                ) : (
                    <div className="flex-1 overflow-y-auto p-0 flex flex-col bg-zinc-950">
                        <div className="p-4 border-b border-zinc-900 flex items-center justify-between bg-zinc-900/30">
                            <div className="flex items-center gap-2">
                                <Brain size={16} className="text-blue-500" />
                                <h3 className="text-xs font-bold text-zinc-200 tracking-wider">{i18n.t('agent_config.memory_title')}</h3>
                            </div>
                            <div className="flex items-center gap-3">
                                <span className="text-[10px] text-zinc-500 font-mono">
                                    {i18n.t('agent_config.memory_rows')} <span className="text-blue-400">{memories.length}</span>
                                </span>
                                <Tooltip content={i18n.t('agent_config.tooltip_sync_memory')} position="left">
                                    <button
                                        onClick={() => agent?.id && fetchMemories(agent.id)}
                                        disabled={isMemoryLoading}
                                        className="w-11 h-11 flex items-center justify-center rounded bg-zinc-800 hover:bg-zinc-700 text-zinc-400 transition-colors disabled:opacity-50"
                                    >
                                        <RefreshCw size={12} className={isMemoryLoading ? 'animate-spin' : ''} />
                                    </button>
                                </Tooltip>
                            </div>
                        </div>

                        <div className="p-4 border-b border-zinc-900 bg-zinc-900/20 space-y-3">
                            <label className="block text-[10px] font-bold text-zinc-500 uppercase tracking-widest flex items-center gap-2">
                                <Brain size={12} className="text-blue-400" />
                                {i18n.t('agent_config.manual_injection')}
                            </label>
                            <div className="flex gap-2">
                                <textarea
                                    value={memoryInput}
                                    onChange={(e) => setMemoryInput(e.target.value)}
                                    placeholder={i18n.t('agent_config.placeholder_memory')}
                                    className="flex-1 bg-zinc-900 border border-zinc-800 rounded-lg px-3 py-2 text-xs text-zinc-200 focus:outline-none focus:border-blue-500 font-mono resize-none h-20 custom-scrollbar"
                                    aria-label={i18n.t('agent_config.aria_memory_input')}
                                />
                                <Tooltip content={i18n.t('agent_config.tooltip_persist_memory')} position="left">
                                    <button
                                        onClick={async () => {
                                            if (!memoryInput.trim() || !agent?.id) return;
                                            await saveMemory(agent.id, memoryInput);
                                            setMemoryInput('');
                                        }}
                                        disabled={isMemoryLoading || !memoryInput.trim()}
                                        className="w-11 h-11 flex items-center justify-center rounded-lg bg-blue-500/10 text-blue-400 hover:bg-blue-500/20 disabled:opacity-30 self-end transition-all"
                                    >
                                        <Send size={14} />
                                    </button>
                                </Tooltip>
                            </div>
                        </div>

                        <div className="flex-1 overflow-y-auto custom-scrollbar p-4 space-y-3">
                            {isMemoryLoading && memories.length === 0 ? (
                                <div className="flex items-center justify-center h-32">
                                    <span className="text-xs text-zinc-500 font-mono animate-pulse">{i18n.t('agent_config.querying_memory')}</span>
                                </div>
                            ) : memories.length === 0 ? (
                                <div className="flex flex-col items-center justify-center h-48 border border-dashed border-zinc-800 rounded-lg bg-zinc-900/10">
                                    <Brain size={24} className="text-zinc-700 mb-2 opacity-50" />
                                    <span className="text-xs text-zinc-500 font-mono tracking-wider">{i18n.t('agent_config.no_memory')}</span>
                                    <span className="text-[10px] text-zinc-600 mt-1 max-w-[200px] text-center">{i18n.t('agent_config.no_memory_desc')}</span>
                                </div>
                            ) : (
                                memories.map((mem) => (
                                    <div key={mem.id} className="group flex flex-col p-3 rounded-lg border border-zinc-800 bg-zinc-900/30 hover:border-blue-500/30 transition-colors relative overflow-hidden">
                                        <div className="absolute top-0 left-0 w-1 h-full bg-blue-500/20 group-hover:bg-blue-500/50 transition-colors" />
                                        <div className="flex items-start justify-between mb-2 pl-3">
                                            <div className="flex items-center gap-2">
                                                <span className="text-[10px] font-mono text-zinc-500 uppercase flex items-center gap-1">
                                                    <CheckSquare size={10} className="text-emerald-500" />
                                                    {new Date(mem.timestamp * 1000).toLocaleString()}
                                                </span>
                                                <span className="text-[9px] font-bold tracking-widest text-blue-400 bg-blue-500/10 px-1.5 py-0.5 rounded uppercase">
                                                    {i18n.t('agent_config.label_mission')} {mem.mission_id.substring(0, 8)}
                                                </span>
                                            </div>
                                            <Tooltip content={i18n.t('agent_config.tooltip_delete_memory')} position="left">
                                                <button
                                                    onClick={() => agent?.id && deleteMemory(agent.id, mem.id)}
                                                    className="text-zinc-600 hover:text-red-400 opacity-0 group-hover:opacity-100 transition-opacity w-11 h-11 flex items-center justify-center"
                                                >
                                                    <Trash2 size={12} />
                                                </button>
                                            </Tooltip>

                                        </div>
                                        <p className="text-xs text-zinc-300 leading-relaxed pl-3 whitespace-pre-wrap">
                                            {mem.text}
                                        </p>
                                    </div>
                                ))
                            )}
                        </div>
                    </div>
                )}

                <div className="p-4 border-t border-zinc-800 bg-zinc-900 shrink-0">
                    <Tooltip content={i18n.t('agent_config.tooltip_save')} position="top">
                        <button onClick={handleSave} disabled={ui.saving} className="w-full flex items-center justify-center gap-2 px-4 py-2.5 rounded-lg bg-blue-600 text-white text-xs font-bold hover:bg-blue-500 disabled:opacity-50">
                            <Save size={14} />
                            {ui.saving ? i18n.t('agent_config.saving') : (isNew ? i18n.t('agent_config.btn_create') : i18n.t('agent_config.btn_save'))}
                        </button>
                    </Tooltip>
                </div>
            </div >
        </div >
    );
}


