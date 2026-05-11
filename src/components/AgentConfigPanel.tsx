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
import { tadpole_os_service } from '../services/tadpoleos_service';
import type { 
    Agent, 
    Agent_Model_Slot_Key, 
    Agent_Model_Slot_State, 
    Agent_Voice_Engine, 
    Agent_Stt_Engine 
} from '../types';
import { i18n } from '../i18n';
import type { Agent_Config_Action } from '../hooks/useAgentForm';

interface Memory_Entry {
    id: string;
    content?: string;
    text?: string;
}

// Decomposed Components
import {
    AgentConfigHeader,
    CognitionSection,
    VoiceSection,
    GovernanceSection,
    MemorySection,
    DirectMessageConsole,
    useAgentConfig
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

    // Local state for memories (separate from config form)
    const [memories, setMemories] = useState<Memory_Entry[]>([]);
    const [isLoadingMemories, setIsLoadingMemories] = useState(false);
    const [memoryInput, setMemoryInput] = useState('');
    const [isDetached, setIsDetached] = useState(false);

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

    const loadMemories = useCallback(async () => {
        if (!agent?.id) return;
        setIsLoadingMemories(true);
        try {
            const response = await tadpole_os_service.get_agent_memory(agent.id) as { entries: Memory_Entry[] };
            setMemories(response.entries || []);
        } finally {
            setIsLoadingMemories(false);
        }
    }, [agent?.id]);

    useEffect(() => {
        if (agent?.id && mainTab === 'memory') {
            // eslint-disable-next-line react-hooks/set-state-in-effect
            loadMemories();
        }
    }, [agent?.id, mainTab, loadMemories]);

    const handleSaveMemory = async () => {
        if (!memoryInput.trim() || !agent?.id) return;
        await tadpole_os_service.save_agent_memory(agent.id, memoryInput);
        setMemoryInput('');
        loadMemories();
    };

    const handleDeleteMemory = async (id: string) => {
        if (!agent?.id) return;
        await tadpole_os_service.delete_agent_memory(agent.id, id);
        loadMemories();
    };

    const allSkills = useMemo(() => {
        const names = new Set<string>();
        manifests?.forEach(m => names.add(m.name));
        scripts?.forEach(s => names.add(s.name));
        mcp_tools?.forEach(t => names.add(t.name));
        return Array.from(names).sort((a, b) => a.localeCompare(b));
    }, [manifests, scripts, mcp_tools]);

    const allWorkflows = useMemo(() => (workflows || []).map((w: { name: string }) => w.name), [workflows]);

    if (!agent && !ui.saving) return null;

    const panelContent = (
        <div className="flex-1 flex flex-col min-h-0 bg-zinc-950/40 backdrop-blur-xl">
            <AgentConfigHeader
                name={identity.name}
                role={identity.role}
                department={identity.department}
                themeColor={ui.theme_color}
                isNew={isNew || !agent?.id}
                agentId={agent?.id}
                availableRoles={Object.keys(roles)}
                onClose={onClose}
                onDetach={() => setIsDetached(true)}
                isDetached={isDetached}
                onUpdateIdentity={(field, value) => dispatch({ type: 'UPDATE_IDENTITY', field, value: value as string })}
                onUpdateThemeColor={(color) => dispatch({ type: 'SET_UI', field: 'theme_color', value: color })}
                onRoleChange={handleRoleChange}
            />

            <div className="flex border-b border-zinc-900 bg-zinc-900/40 px-6 shrink-0 z-10">
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
                            themeColor={ui.theme_color}
                            onSetTab={(tab) => dispatch({ type: 'SET_TAB', payload: tab })}
                            onUpdateSlotField={<K extends keyof Agent_Model_Slot_State>(slot: Agent_Model_Slot_Key, field: K, value: Agent_Model_Slot_State[K]) => {
                                dispatch({ type: 'UPDATE_SLOT', slot, field, value } as unknown as Agent_Config_Action);
                            }}
                            onToggleSkill={(slot, kind, value) => dispatch({ type: 'TOGGLE_SKILL', slot, kind, value })}
                            onProviderChange={(slot, val) => handleProviderChange(slot, val)}
                            onPause={handlePause}
                            onResume={handleResume}
                        />
                    )}
                    {mainTab === 'memory' && (
                        <MemorySection
                            memories={memories}
                            connectorConfigs={state.connector_configs}
                            isLoading={isLoadingMemories}
                            memoryInput={memoryInput}
                            themeColor={ui.theme_color}
                            onMemoryInputChange={setMemoryInput}
                            onSaveMemory={handleSaveMemory}
                            onDeleteMemory={handleDeleteMemory}
                            onRefresh={loadMemories}
                            onAddConnector={(uri) => dispatch({ type: 'ADD_CONNECTOR', payload: { type: 'fs', uri } })}
                            onRemoveConnector={(uri) => dispatch({ type: 'REMOVE_CONNECTOR', uri })}
                        />
                    )}
                    {mainTab === 'governance' && (
                        <GovernanceSection
                            budget_usd={governance.budget_usd}
                            requires_oversight={governance.requires_oversight}
                            cost_usd={agent?.cost_usd || 0}
                            theme_color={ui.theme_color}
                            onUpdateGovernance={(field, value) => {
                                if (field === 'budget_usd') {
                                    dispatch({ type: 'UPDATE_GOVERNANCE', field, value: value as number });
                                } else if (field === 'requires_oversight') {
                                    dispatch({ type: 'UPDATE_GOVERNANCE', field, value: value as boolean });
                                }
                            }}
                        />
                    )}

                    <div className="px-6 py-4 space-y-6 bg-zinc-900/20 border-t border-zinc-900 shrink-0">
                        <VoiceSection
                            voice={{
                                voice_id: voice.voice_id as string,
                                voice_engine: voice.voice_engine as Agent_Voice_Engine
                            }}
                            stt_engine={(voice.stt_engine as Agent_Stt_Engine) || 'groq'}
                            theme_color={ui.theme_color}
                            on_update_voice={(field, value) => dispatch({ type: 'UPDATE_VOICE', field: field as 'voice_id' | 'voice_engine' | 'stt_engine', value })}
                        />
                    </div>
                </div>

                <DirectMessageConsole
                    value={ui.direct_message}
                    onUpdateValue={(val) => dispatch({ type: 'SET_UI', field: 'direct_message', value: val })}
                    onSend={handleSendMessage}
                    agentName={identity.name}
                    themeColor={ui.theme_color}
                />
            </div>

            <div className="p-6 bg-zinc-900/50 border-t border-zinc-800 flex items-center justify-between gap-4 shrink-0 z-10">
                <button
                    onClick={() => dispatch({ type: 'SET_UI', field: 'show_promote', value: !ui.show_promote })}
                    className="px-4 py-2 rounded-xl border border-zinc-800 text-[10px] font-bold text-zinc-500 uppercase tracking-[0.2em] hover:border-zinc-700 hover:text-zinc-300 transition-all"
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
                                className="bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-1.5 text-xs text-zinc-300 focus:outline-none focus:border-emerald-500/50 w-40"
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
                            backgroundColor: ui.theme_color,
                            color: 'black',
                            boxShadow: `0 0 20px ${ui.theme_color}40`
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
                        title={identity.name || i18n.t('agent_config.new_agent')}
                        url={`/detached-view?type=agent-config&id=${agent?.id || 'new'}`}
                        on_close={() => setIsDetached(false)}
                    >
                        {panelContent}
                    </Portal_Window>

                    <motion.div
                        initial={{ opacity: 0, scale: 0.9 }}
                        animate={{ opacity: 1, scale: 1 }}
                        exit={{ opacity: 0, scale: 0.9 }}
                        className="relative w-full max-w-2xl aspect-video bg-zinc-950/80 backdrop-blur-2xl border border-zinc-900 rounded-[2.5rem] shadow-2xl flex flex-col items-center justify-center space-y-6 pointer-events-auto overflow-hidden"
                    >
                        <div className="absolute inset-0 neural-grid opacity-[0.05] pointer-events-none" />
                        <div className="relative">
                            <ExternalLink size={64} className="text-zinc-800 animate-pulse" />
                            <div className="absolute inset-0 bg-green-500/10 blur-2xl rounded-full" />
                        </div>
                        <div className="text-center space-y-2 px-6">
                            <h3 className="text-xl font-bold tracking-tight text-zinc-200">{i18n.t('layout.sector_detached')}</h3>
                            <p className="text-sm text-zinc-500 font-mono uppercase tracking-[0.2em]">
                                {i18n.t('layout.link_established')} :: ID_{agent?.id?.substring(0, 8).toUpperCase() || 'NEW'}
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
                 <div className="flex-1 min-h-0 bg-zinc-950/20 rounded-3xl border border-zinc-900 overflow-hidden shadow-2xl backdrop-blur-3xl relative">
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
                    className="absolute inset-0 bg-black/60 backdrop-blur-md pointer-events-auto"
                />

                <motion.div
                    initial={{ opacity: 0, scale: 0.95, y: 20 }}
                    animate={{ opacity: 1, scale: 1, y: 0 }}
                    exit={{ opacity: 0, scale: 0.95, y: 20 }}
                    transition={{ type: 'spring', damping: 25, stiffness: 300 }}
                    className="relative w-full max-w-2xl max-h-[95vh] sm:max-h-[90vh] bg-zinc-950/90 backdrop-blur-2xl rounded-[2.5rem] border border-white/5 shadow-[0_40px_100px_-20px_rgba(0,0,0,0.8)] flex flex-col overflow-hidden pointer-events-auto"
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

// Metadata: [AgentConfigPanel]
