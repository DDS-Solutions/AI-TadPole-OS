/**
 * @docs ARCHITECTURE:Interface
 * @docs OPERATIONS_MANUAL:Agents
 * 
 * ### AI Assist Note
 * **UI Component**: Main orchestration component for agent configuration. 
 * Manages tab state (Cognition, Memory, Governance), portal detachment logic, and capability synchronization (Skills, MCP Tools, Workflows). 
 * Integrates `useAgentConfig` for reducer-driven state management and `tadpole_os_service` for memory persistence.
 * 
 * ### 🧬 Logic Flow (Mermaid)
 * ```mermaid
 * graph TD
 *     A[Agent Prop] --> H[useAgentConfig Hook]
 *     H --> S[Local State: Identity, Slots, Voice, Governance]
 *     S --> T{Tab Selection}
 *     T -->|Cognition| C[Cognition_Section: Models & Slots]
 *     T -->|Memory| M[Memory_Section: Vector Store & SME]
 *     T -->|Governance| G[Governance_Section: Budget & Oversight]
 *     
 *     H --> P[Persistence: handle_save]
 *     P --> API[agent_api_service.update_agent]
 * ```
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Memory load failure (API 500), detachment sync loss (Portal context drop), or skill store exhaustion (missing manifests).
 * - **Telemetry Link**: Search for `[AgentConfigPanel]` in UI traces or check `tadpole_os_service.get_agent_memory` calls.
 */

import { useEffect, useState, useMemo, useCallback } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { use_model_store } from '../stores/model_store';
import type { Model_State } from '../stores/model_store';
import { use_provider_store } from '../stores/provider_store';
import type { Provider_State } from '../stores/provider_store';
import { use_role_store } from '../stores/role_store';
import type { Role_State } from '../stores/role_store';
import { use_skill_store } from '../stores/skill_store';
import { useShallow } from 'zustand/react/shallow';
import { event_bus } from '../services/event_bus';
import type { 
    Agent, 
    Agent_Model_Slot_Key, 
    Agent_Model_Slot_State, 
    Agent_Voice_Engine, 
    Agent_Stt_Engine 
} from '../types';
import { i18n } from '../i18n';

// Decomposed Components
import {
    AgentConfigHeader,
    CognitionSection,
    VoiceSection,
    GovernanceSection,
    MemorySection,
    DirectMessageConsole,
    useAgentConfig,
    useAgentMemory
} from './agent-config';
import { Portal_Window } from './ui/Portal_Window';
import { ExternalLink } from 'lucide-react';

interface AgentConfigPanelProps {
    agent: Agent | undefined;
    onClose: () => void;
    onUpdate: (id: string, updates: Partial<Agent>) => void;
    isNew?: boolean;
    isDetachedMode?: boolean;
}

/**
 * AgentConfigPanel
 * Main orchestration component for agent configuration.
 * Manages tab state, detached window logic, and capability synchronization.
 */
export default function AgentConfigPanel({ agent, onClose, onUpdate, isNew = false, isDetachedMode = false }: AgentConfigPanelProps) {
    const {
        state,
        dispatch,
        handleRoleChange,
        handleProviderChange,
        handleSave,
        handlePause,
        handleResume,
        handleSendMessage,
        handlePromote
    } = useAgentConfig(agent, onUpdate, onClose);

    const { identity, slots, voice, ui, governance, main_tab: mainTab, active_tab: activeTab } = state;

    // External Stores
    const providers = use_provider_store((s: Provider_State) => s.providers);
    const models = use_model_store((s: Model_State) => s.models);
    const roles = use_role_store((s: Role_State) => s.roles);

    // Stable selectors for skill store
    const {
        manifests,
        scripts,
        workflows,
        mcp_tools,
        fetch_skills,
        fetch_mcp_tools,
        is_loading,
        initialized_skills,
        initialized_mcp
    } = use_skill_store(
        useShallow((s) => ({
            manifests: s.manifests,
            scripts: s.scripts,
            workflows: s.workflows,
            mcp_tools: s.mcp_tools,
            fetch_skills: s.fetch_skills,
            fetch_mcp_tools: s.fetch_mcp_tools,
            is_loading: s.is_loading,
            initialized_skills: s.initialized_skills,
            initialized_mcp: s.initialized_mcp
        }))
    );

    const [isDetached, setIsDetached] = useState(false);
    const agentId = agent?.id;

    const {
        memories,
        isLoadingMemories,
        memoryInput,
        setMemoryInput,
        loadMemories,
        handleSaveMemory,
        handleDeleteMemory
    } = useAgentMemory(agentId, mainTab);

    /**
     * Capability Synchronization Hook
     * 
     * ### 🛰️ Orchestration: Skill & Tool Discovery
     * 1. **Lazy Loading**: Skips execution if a load is already in flight.
     * 2. **Skill Refresh**: Fetches neural skill manifests and script definitions 
     *    if the registry is empty.
     * 3. **Tool Hub Discovery**: Triggers `fetch_mcp_tools` to resolve real-world 
     *    interactions (File Search, Browsing, etc.) for IMR-01.
     */
    useEffect(() => {
        if (is_loading) return;

        // Fetch capabilities if they haven't been initialized yet
        if (!initialized_skills) {
            fetch_skills();
        }
        if (!initialized_mcp) {
            fetch_mcp_tools();
        }
    }, [fetch_skills, fetch_mcp_tools, initialized_skills, initialized_mcp, is_loading]);

    const formatAgentId = useCallback((id?: string): string => {
        if (!id || id === 'new') return 'NEW';
        return id.length >= 8 ? id.substring(0, 8).toUpperCase() : id.toUpperCase();
    }, []);



    const allSkills = useMemo(() => {
        const names = new Set<string>();
        manifests?.forEach(m => names.add(m.name));
        scripts?.forEach(s => names.add(s.name));
        mcp_tools?.forEach(t => names.add(t.name));
        return Array.from(names).sort((a, b) => a.localeCompare(b));
    }, [manifests, scripts, mcp_tools]);

    const allWorkflows = useMemo(() => (workflows || []).map((w: { name: string }) => w.name), [workflows]);

    // Deconstruct and stabilize prop callbacks to prevent dynamic inline references on child rerenders
    const handleUpdateIdentity = useCallback((field: 'name' | 'role' | 'department', value: string) => {
        dispatch({ type: 'UPDATE_IDENTITY', field, value });
    }, [dispatch]);

    const handleUpdateThemeColor = useCallback((color: string) => {
        dispatch({ type: 'SET_UI', field: 'theme_color', value: color });
    }, [dispatch]);

    const handleSetTab = useCallback((tab: Agent_Model_Slot_Key) => {
        dispatch({ type: 'SET_TAB', payload: tab });
    }, [dispatch]);

    type SlotFieldUpdater = <K extends keyof Agent_Model_Slot_State>(
        slot: Agent_Model_Slot_Key,
        field: K,
        value: Agent_Model_Slot_State[K]
    ) => void;

    const handleUpdateSlotField = useCallback<SlotFieldUpdater>((slot, field, value) => {
        if (field === 'skills' || field === 'workflows') {
            dispatch({ type: 'UPDATE_SLOT', slot, field, value: value as string[] });
        } else if (field === 'temperature' || field === 'reasoning_depth' || field === 'act_threshold') {
            dispatch({ type: 'UPDATE_SLOT', slot, field, value: value as number });
        } else {
            dispatch({ type: 'UPDATE_SLOT', slot, field, value: value as string });
        }
    }, [dispatch]);

    const handleToggleSkill = useCallback((slot: Agent_Model_Slot_Key, kind: 'skills' | 'workflows', value: string) => {
        dispatch({ type: 'TOGGLE_SKILL', slot, kind, value });
    }, [dispatch]);

    const handleAddConnector = useCallback((uri: string) => {
        const type = uri.startsWith('http') ? 'http' : uri.startsWith('ws') ? 'ws' : 'fs';
        dispatch({ type: 'ADD_CONNECTOR', payload: { type, uri } });
        event_bus.emit_log({
            text: i18n.t('agent_config.connector_added', { uri }) || `FileSystem connection established: ${uri}`,
            severity: 'info',
            source: 'System'
        });
    }, [dispatch]);

    const handleRemoveConnector = useCallback((uri: string) => {
        dispatch({ type: 'REMOVE_CONNECTOR', uri });
        event_bus.emit_log({
            text: i18n.t('agent_config.connector_removed', { uri }) || `FileSystem connection purged: ${uri}`,
            severity: 'info',
            source: 'System'
        });
    }, [dispatch]);

    type GovernanceField = 'budget_usd' | 'requires_oversight' | 'economic_zone' | 'daily_spend_limit';
    type GovernanceValue<F extends GovernanceField> = F extends 'budget_usd' | 'daily_spend_limit' ? number : F extends 'requires_oversight' ? boolean : string;

    const handleUpdateGovernance = useCallback(
        <F extends GovernanceField>(field: F, value: GovernanceValue<F>) => {
            if (field === 'budget_usd' || field === 'daily_spend_limit') {
                dispatch({ type: 'UPDATE_GOVERNANCE', field, value: value as number });
            } else if (field === 'requires_oversight') {
                dispatch({ type: 'UPDATE_GOVERNANCE', field, value: value as boolean });
            } else if (field === 'economic_zone') {
                dispatch({ type: 'UPDATE_GOVERNANCE', field, value: value as string });
            }
        },
        [dispatch]
    );

    const handleUpdateVoice = useCallback((field: 'voice_id' | 'voice_engine' | 'stt_engine', value: string) => {
        dispatch({ type: 'UPDATE_VOICE', field, value });
    }, [dispatch]);

    const voiceProp = useMemo(() => ({
        voice_id: voice.voice_id as string,
        voice_engine: voice.voice_engine as Agent_Voice_Engine
    }), [voice.voice_id, voice.voice_engine]);

    const safeThemeColor = useMemo(() => {
        return (ui.theme_color && /^#[0-9A-F]{6}$/i.test(ui.theme_color)) ? ui.theme_color : '#10b981';
    }, [ui.theme_color]);

    if (!agent && !ui.saving) return null;

    const panelContent = (
        <div className="flex-1 flex flex-col min-h-0 bg-[color:var(--color-background)]/40 backdrop-blur-xl">
            <AgentConfigHeader
                name={identity.name}
                role={identity.role}
                department={identity.department}
                themeColor={safeThemeColor}
                isNew={isNew || !agent?.id}
                agentId={agent?.id}
                availableRoles={Object.keys(roles)}
                onClose={onClose}
                onDetach={() => setIsDetached(true)}
                isDetached={isDetached}
                onUpdateIdentity={handleUpdateIdentity}
                onUpdateThemeColor={handleUpdateThemeColor}
                onRoleChange={handleRoleChange}
            />

            <div className="flex border-b border-[color:var(--color-surface)] bg-[color:var(--color-surface)]/40 px-6 shrink-0 z-10">
                <button
                    onClick={() => dispatch({ type: 'SET_MAIN_TAB', payload: 'cognition' })}
                    className={`px-4 py-3 text-[10px] font-bold uppercase tracking-[0.2em] transition-all relative ${mainTab === 'cognition' ? 'text-emerald-400' : 'text-zinc-500 hover:text-zinc-300'}`}
                >
                    {i18n.t('agent_config.tab_cognition')}
                    {mainTab === 'cognition' && <div className="absolute bottom-0 left-0 w-full h-0.5 bg-emerald-500" />}
                </button>
                <button
                    onClick={() => dispatch({ type: 'SET_MAIN_TAB', payload: 'memory' })}
                    className={`px-4 py-3 text-[10px] font-bold uppercase tracking-[0.2em] transition-all relative ${mainTab === 'memory' ? 'text-green-400' : 'text-zinc-500 hover:text-zinc-300'}`}
                >
                    {i18n.t('agent_config.tab_memory')}
                    {mainTab === 'memory' && <div className="absolute bottom-0 left-0 w-full h-0.5 bg-green-500" />}
                </button>
                <button
                    onClick={() => dispatch({ type: 'SET_MAIN_TAB', payload: 'governance' })}
                    className={`px-4 py-3 text-[10px] font-bold uppercase tracking-[0.2em] transition-all relative ${mainTab === 'governance' ? 'text-amber-400' : 'text-zinc-500 hover:text-zinc-300'}`}
                >
                    {i18n.t('agent_config.tab_governance')}
                    {mainTab === 'governance' && <div className="absolute bottom-0 left-0 w-full h-0.5 bg-amber-500" />}
                </button>
            </div>

            <div className="flex-1 overflow-hidden flex flex-col relative min-h-0">
                <div className="flex-1 overflow-y-auto custom-scrollbar min-h-0">
                    {mainTab === 'cognition' && (
                        <CognitionSection
                            activeTab={activeTab}
                            slots={slots}
                            agentStatus={agent?.status || 'idle'}
                            providers={providers}
                            models={models}
                            allSkills={allSkills}
                            allWorkflows={allWorkflows}
                            manifests={manifests}
                            scripts={scripts}
                            mcpTools={mcp_tools}
                            themeColor={safeThemeColor}
                            activeModelSlot={agent?.active_model_slot}
                            onSetTab={handleSetTab}
                            onUpdateSlotField={handleUpdateSlotField}
                            onToggleSkill={handleToggleSkill}
                            onProviderChange={(slot, val) => handleProviderChange(slot, val)}
                            onPause={handlePause}
                            onResume={handleResume}
                        />
                    )}
                    {mainTab === 'memory' && (
                        <MemorySection
                            memories={memories}
                            connectorConfigs={state.connector_configs || []}
                            isLoading={isLoadingMemories}
                            memoryInput={memoryInput}
                            themeColor={safeThemeColor}
                            onMemoryInputChange={setMemoryInput}
                            onSaveMemory={handleSaveMemory}
                            onDeleteMemory={handleDeleteMemory}
                            onRefresh={loadMemories}
                            onAddConnector={handleAddConnector}
                            onRemoveConnector={handleRemoveConnector}
                        />
                    )}
                    {mainTab === 'governance' && (
                        <GovernanceSection
                            key={agent?.id || 'new'}
                            budget_usd={governance.budget_usd}
                            requires_oversight={governance.requires_oversight}
                            economic_zone={governance.economic_zone || 'DEV'}
                            daily_spend_limit={governance.daily_spend_limit || 0}
                            daily_spent_accumulated={agent?.daily_spent_accumulated || 0}
                            balance={agent?.balance || 0}
                            inventory={agent?.inventory || []}
                            cost_usd={agent?.cost_usd || 0}
                            theme_color={safeThemeColor}
                            onUpdateGovernance={handleUpdateGovernance}
                        />
                    )}

                    <div className="px-6 py-4 space-y-6 bg-[color:var(--color-surface)]/20 border-t border-[color:var(--color-surface)] shrink-0">
                        <VoiceSection
                            voice={voiceProp}
                            stt_engine={(voice.stt_engine as Agent_Stt_Engine) || 'groq'}
                            theme_color={safeThemeColor}
                            on_update_voice={handleUpdateVoice}
                        />
                    </div>
                </div>

                <DirectMessageConsole
                    value={ui.direct_message}
                    onUpdateValue={(val) => dispatch({ type: 'SET_UI', field: 'direct_message', value: val })}
                    onSend={handleSendMessage}
                    agentName={identity.name}
                    themeColor={safeThemeColor}
                />
            </div>

            <div className="p-6 bg-[color:var(--color-surface)]/50 border-t border-[color:var(--color-border)] flex items-center justify-between gap-4 shrink-0 z-10">
                <button
                    onClick={() => dispatch({ type: 'SET_UI', field: 'show_promote', value: !ui.show_promote })}
                    className="px-4 py-2 rounded-xl border border-[color:var(--color-border)] text-[10px] font-bold text-zinc-500 uppercase tracking-[0.2em] hover:border-zinc-700 hover:text-zinc-300 transition-all"
                >
                    {ui.show_promote ? i18n.t('agent_config.btn_cancel') : i18n.t('agent_config.btn_save_as_role')}
                </button>

                <div className="flex items-center gap-3">
                    {ui.show_promote && (
                        <div className="flex items-center gap-2 animate-in slide-in-from-right-4 duration-300">
                            <input
                                placeholder={i18n.t('agent_config.placeholder_role_name')}
                                aria-label={i18n.t('agent_config.placeholder_role_name')}
                                value={ui.new_role_name}
                                onChange={(e) => dispatch({ type: 'SET_UI', field: 'new_role_name', value: e.target.value })}
                                className="bg-[color:var(--color-background)] border border-[color:var(--color-border)] rounded-lg px-3 py-1.5 text-xs text-zinc-300 focus:outline-none focus:border-emerald-500/50 w-40"
                            />
                            <button
                                onClick={handlePromote}
                                className="px-4 py-1.5 bg-emerald-500/10 text-emerald-500 border border-emerald-500/20 rounded-lg text-[10px] font-bold uppercase tracking-[0.2em] hover:bg-emerald-500/20 transition-all"
                            >
                                {i18n.t('agent_config.btn_confirm')}
                            </button>
                        </div>
                    )}

                    <button
                        onClick={handleSave}
                        disabled={ui.saving}
                        className="px-8 py-2.5 rounded-xl text-[10px] font-bold uppercase tracking-[0.2em] transition-all disabled:opacity-50 disabled:grayscale flex items-center gap-2 shadow-lg"
                        style={{
                            backgroundColor: safeThemeColor,
                            color: 'black',
                            boxShadow: `0 0 20px ${safeThemeColor}40`
                        }}
                    >
                        {ui.saving ? (
                            <>
                                <div className="w-3 h-3 border-2 border-black/30 border-t-black rounded-full animate-spin" />
                                {i18n.t('agent_config.btn_saving')}
                            </>
                        ) : (isNew ? i18n.t('agent_config.btn_create_agent') : i18n.t('agent_config.btn_save_config'))}
                    </button>
                </div>
            </div>
        </div>
    );

    /**
     * Portal Detachment Phase
     * 
     * ### 🪟 Interface: External Config Window
     * Moves the agent configuration experience to a dedicated browser portal.
     * This allows the Overlord to configure agents while observing the 
     * Swarm Visualizer or Chat in the main workspace.
     */
    if (isDetached) {
        return (
            <AnimatePresence>
                <div className="fixed inset-0 z-[100] flex items-center justify-center p-4 pointer-events-none">
                    <Portal_Window
                        id={`agent-config-${agent?.id || 'new'}`}
                        title={`${identity.name || i18n.t('agent_config.new_agent')} (${formatAgentId(agent?.id)})`}
                        url={`/detached-view?type=agent-config&id=${encodeURIComponent(agent?.id || 'new')}`}
                        on_close={() => setIsDetached(false)}
                    >
                        {panelContent}
                    </Portal_Window>

                    <motion.div
                        initial={{ opacity: 0, scale: 0.9 }}
                        animate={{ opacity: 1, scale: 1 }}
                        exit={{ opacity: 0, scale: 0.9 }}
                        className="relative w-full max-w-2xl aspect-video bg-[color:color-mix(in_srgb,var(--color-background)_80%,transparent)] backdrop-blur-2xl border border-[color:var(--color-surface)] rounded-[2.5rem] shadow-2xl flex flex-col items-center justify-center space-y-6 pointer-events-auto overflow-hidden"
                    >
                        <div className="absolute inset-0 neural-grid opacity-[0.05] pointer-events-none" />
                        <div className="relative">
                            <ExternalLink size={64} className="text-zinc-800 animate-pulse" />
                            <div className="absolute inset-0 bg-green-500/10 blur-2xl rounded-full" />
                        </div>
                        <div className="text-center space-y-2 px-6">
                            <h3 className="text-xl font-bold tracking-tight text-zinc-200">{i18n.t('layout.sector_detached')}</h3>
                            <p className="text-sm text-zinc-500 font-mono uppercase tracking-[0.2em]">
                                {i18n.t('layout.link_established')} :: ID_{formatAgentId(agent?.id)}
                            </p>
                        </div>
                        <button
                            onClick={() => setIsDetached(false)}
                            className="px-6 py-2 bg-zinc-100 text-black text-xs font-bold uppercase tracking-[0.2em] rounded-lg hover:bg-white transition-all shadow-lg active:scale-95 z-10"
                        >
                            {i18n.t('layout.recall_sector')}
                        </button>
                    </motion.div>
                </div>
            </AnimatePresence>
        );
    }

    if (isDetachedMode) {
        return (
            <div className="w-full h-full p-4 overflow-hidden flex flex-col">
                 <div className="flex-1 min-h-0 bg-[color:var(--color-background)]/20 rounded-3xl border border-[color:var(--color-surface)] overflow-hidden shadow-2xl backdrop-blur-3xl relative">
                    <div className="absolute inset-0 neural-grid opacity-[0.05] pointer-events-none" />
                    {panelContent}
                </div>
            </div>
        );
    }

    return (
        <AnimatePresence>
            <div className="fixed inset-0 z-[100] flex items-center justify-center p-2 sm:p-6 md:p-10 pointer-events-none">
                <motion.div
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    exit={{ opacity: 0 }}
                    onClick={onClose}
                    onKeyDown={(e) => {
                        if (e.key === 'Enter' || e.key === ' ') {
                            onClose();
                            e.preventDefault();
                        }
                    }}
                    tabIndex={0}
                    role="button"
                    aria-label="Close panel"
                    className="absolute inset-0 bg-black/60 backdrop-blur-md pointer-events-auto"
                />

                <motion.div
                    initial={{ opacity: 0, scale: 0.95, y: 20 }}
                    animate={{ opacity: 1, scale: 1, y: 0 }}
                    exit={{ opacity: 0, scale: 0.95, y: 20 }}
                    transition={{ type: 'spring', damping: 25, stiffness: 300 }}
                    className="relative w-full max-w-2xl max-h-[95vh] sm:max-h-[90vh] bg-[color:var(--color-background)]/90 backdrop-blur-2xl rounded-[2.5rem] border border-white/5 shadow-[0_40px_100px_-20px_rgba(0,0,0,0.8)] flex flex-col overflow-hidden pointer-events-auto"
                >
                    <div className="absolute inset-0 neural-grid opacity-[0.03] pointer-events-none" />
                    <div className="flex-1 flex flex-col min-h-0">
                        {panelContent}
                    </div>
                </motion.div>
            </div>
        </AnimatePresence>
    );
}

// Metadata: [AgentConfigPanel]
