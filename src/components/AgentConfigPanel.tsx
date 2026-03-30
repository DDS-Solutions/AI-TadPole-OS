import { useEffect, useState, useMemo, useCallback } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { useModelStore } from '../stores/modelStore';
import { useProviderStore } from '../stores/providerStore';
import { useRoleStore } from '../stores/roleStore';
import { useSkillStore } from '../stores/skillStore';
import { TadpoleOSService } from '../services/tadpoleosService';
import type { Agent } from '../types';
import { i18n } from '../i18n';
import type { SkillManifest } from '../services/MissionApiService';
import type { SkillDefinition, McpToolHubDefinition } from '../stores/skillStore';

interface MemoryEntry {
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
import { PortalWindow } from './ui/PortalWindow';
import { ExternalLink } from 'lucide-react';

interface AgentConfigPanelProps {
    agent: Agent | undefined;
    onClose: () => void;
    onUpdate: (id: string, updates: Partial<Agent>) => void;
    isNew?: boolean;
}

export default function AgentConfigPanel({ agent, onClose, onUpdate, isNew = false }: AgentConfigPanelProps) {
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

    const { identity, slots, voice, ui, governance, mainTab, activeTab } = state;

    // External Stores
    const providers = useProviderStore(s => s.providers);
    const models = useModelStore(s => s.models);
    const roles = useRoleStore(s => s.roles);
    
    // Stable selectors for skill store
    // Stable selectors for skill store
    const manifests = useSkillStore(s => s.manifests);
    const scripts = useSkillStore(s => s.scripts);
    const workflows = useSkillStore(s => s.workflows);
    const mcpTools = useSkillStore(s => s.mcpTools);
    const fetchSkills = useSkillStore(s => s.fetchSkills);
    const fetchMcpTools = useSkillStore(s => s.fetchMcpTools);
    const isLoading = useSkillStore(s => s.isLoading);

    // Local state for memories (separate from config form)
    const [memories, setMemories] = useState<MemoryEntry[]>([]);
    const [loadingMemories, setLoadingMemories] = useState(false);
    const [memoryInput, setMemoryInput] = useState('');
    const [isDetached, setIsDetached] = useState(false);

    useEffect(() => {
        if (isLoading) return;

        // Fetch capabilities if they haven't been loaded yet
        // We check both manifests and scripts as they come from the same fetch
        if (manifests.length === 0 && scripts.length === 0) {
            fetchSkills();
        }
        if (mcpTools.length === 0) {
            fetchMcpTools();
        }
    }, [fetchSkills, fetchMcpTools, manifests.length, scripts.length, mcpTools.length, isLoading]);

    const loadMemories = useCallback(async () => {
        if (!agent?.id) return;
        setLoadingMemories(true);
        try {
            const response = await TadpoleOSService.getAgentMemory(agent.id) as { entries: MemoryEntry[] };
            setMemories(response.entries || []);
        } finally {
            setLoadingMemories(false);
        }
    }, [agent?.id]);

    useEffect(() => {
        if (agent?.id && mainTab === 'memory') {
            loadMemories();
        }
    }, [agent?.id, mainTab, loadMemories]);

    const handleSaveMemory = async () => {
        if (!memoryInput.trim() || !agent?.id) return;
        await TadpoleOSService.saveAgentMemory(agent.id, memoryInput);
        setMemoryInput('');
        loadMemories();
    };

    const handleDeleteMemory = async (id: string) => {
        if (!agent?.id) return;
        await TadpoleOSService.deleteAgentMemory(agent.id, id);
        loadMemories();
    };

    const allSkills = useMemo(() => {
        const names = new Set([
            ...manifests.map((m: SkillManifest) => m.name),
            ...scripts.map((s: SkillDefinition) => s.name),
            ...mcpTools.map((t: McpToolHubDefinition) => t.name)
        ]);
        return Array.from(names).sort((a, b) => a.localeCompare(b));
    }, [manifests, scripts, mcpTools]);

    const allWorkflows = useMemo(() => workflows.map((w: { name: string }) => w.name), [workflows]);

    if (!agent && !ui.saving) return null;

    const panelContent = (
        <div className="flex-1 flex flex-col min-h-0 bg-zinc-950/40 backdrop-blur-xl">
            <AgentConfigHeader
                name={identity.name}
                role={identity.role}
                themeColor={ui.themeColor}
                isNew={isNew || !agent?.id}
                agentId={agent?.id}
                availableRoles={Object.keys(roles)}
                onClose={onClose}
                onDetach={() => setIsDetached(true)}
                isDetached={isDetached}
                onUpdateIdentity={(field, value) => dispatch({ type: 'UPDATE_IDENTITY', field, value })}
                onUpdateThemeColor={(color) => dispatch({ type: 'SET_UI', field: 'themeColor', value: color })}
                handleRoleChange={handleRoleChange}
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
                    className={`px-4 py-3 text-[10px] font-bold uppercase tracking-[0.2em] transition-all relative ${mainTab === 'memory' ? 'text-blue-400' : 'text-zinc-500 hover:text-zinc-300'}`}
                >
                    {i18n.t('agent_config.tab_memory')}
                    {mainTab === 'memory' && <div className="absolute bottom-0 left-0 w-full h-0.5 bg-blue-500" />}
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
                            mcpTools={mcpTools}
                            themeColor={ui.themeColor}
                            onSetTab={(tab) => dispatch({ type: 'SET_TAB', payload: tab })}
                            onUpdateSlotField={(slot, field, value) => {
                                if (field === 'model' || field === 'provider' || field === 'systemPrompt') {
                                    dispatch({ type: 'UPDATE_SLOT', slot, field, value: value as string });
                                } else if (field === 'temperature') {
                                    dispatch({ type: 'UPDATE_SLOT', slot, field, value: value as number });
                                } else if (field === 'skills' || field === 'workflows') {
                                    dispatch({ type: 'UPDATE_SLOT', slot, field, value: value as string[] });
                                }
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
                            connectorConfigs={state.connectorConfigs}
                            isLoading={loadingMemories}
                            memoryInput={memoryInput}
                            themeColor={ui.themeColor}
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
                            budgetUsd={governance.budgetUsd}
                            requiresOversight={governance.requiresOversight}
                            costUsd={agent?.costUsd || 0}
                            themeColor={ui.themeColor}
                            onUpdateGovernance={(field, value) => {
                                if (field === 'budgetUsd') {
                                    dispatch({ type: 'UPDATE_GOVERNANCE', field, value: value as number });
                                } else if (field === 'requiresOversight') {
                                    dispatch({ type: 'UPDATE_GOVERNANCE', field, value: value as boolean });
                                }
                            }}
                        />
                    )}

                    <div className="px-6 py-4 space-y-6 bg-zinc-900/20 border-t border-zinc-900 shrink-0">
                        <VoiceSection
                            voice={voice}
                            sttEngine={voice.sttEngine || 'groq'}
                            themeColor={ui.themeColor}
                            onUpdateVoice={(field, value) => dispatch({ type: 'UPDATE_VOICE', field: field as 'voiceId' | 'voiceEngine' | 'sttEngine', value })}
                        />
                    </div>
                </div>

                <DirectMessageConsole
                    value={ui.directMessage}
                    onUpdateValue={(val) => dispatch({ type: 'SET_UI', field: 'directMessage', value: val })}
                    onSend={handleSendMessage}
                    agentName={identity.name}
                    themeColor={ui.themeColor}
                />
            </div>

            <div className="p-6 bg-zinc-900/50 border-t border-zinc-800 flex items-center justify-between gap-4 shrink-0 z-10">
                <button
                    onClick={() => dispatch({ type: 'SET_UI', field: 'showPromote', value: !ui.showPromote })}
                    className="px-4 py-2 rounded-xl border border-zinc-800 text-[10px] font-bold text-zinc-500 uppercase tracking-[0.2em] hover:border-zinc-700 hover:text-zinc-300 transition-all"
                >
                    {ui.showPromote ? i18n.t('agent_config.btn_cancel') : i18n.t('agent_config.btn_save_as_role')}
                </button>

                <div className="flex items-center gap-3">
                    {ui.showPromote && (
                        <div className="flex items-center gap-2 animate-in slide-in-from-right-4 duration-300">
                            <input
                                placeholder={i18n.t('agent_config.placeholder_role_name')}
                                value={ui.newRoleName}
                                onChange={(e) => dispatch({ type: 'SET_UI', field: 'newRoleName', value: e.target.value })}
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
                            backgroundColor: ui.themeColor, 
                            color: 'black',
                            boxShadow: `0 0 20px ${ui.themeColor}40`
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

    if (isDetached) {
        return (
            <AnimatePresence>
                <div className="fixed inset-0 z-[100] flex items-center justify-center p-4 pointer-events-none">
                    <PortalWindow
                        id={`agent-config-${agent?.id || 'new'}`}
                        title={identity.name || 'New Agent'}
                        onClose={() => setIsDetached(false)}
                    >
                        {panelContent}
                    </PortalWindow>

                    {/* Placeholder in Main Layout - Centered similar to panel for consistency */}
                    <motion.div 
                        initial={{ opacity: 0, scale: 0.9 }}
                        animate={{ opacity: 1, scale: 1 }}
                        exit={{ opacity: 0, scale: 0.9 }}
                        className="relative w-full max-w-2xl aspect-video bg-zinc-950/80 backdrop-blur-2xl border border-zinc-900 rounded-[2.5rem] shadow-2xl flex flex-col items-center justify-center space-y-6 pointer-events-auto overflow-hidden"
                    >
                        <div className="absolute inset-0 neural-grid opacity-[0.05] pointer-events-none" />
                        <div className="relative">
                            <ExternalLink size={64} className="text-zinc-800 animate-pulse" />
                            <div className="absolute inset-0 bg-blue-500/10 blur-2xl rounded-full" />
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

    return (
        <AnimatePresence>
            <div className="fixed inset-0 z-[100] flex items-center justify-center p-2 sm:p-6 md:p-10 pointer-events-none">
                {/* Neural Backdrop */}
                <motion.div
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    exit={{ opacity: 0 }}
                    onClick={onClose}
                    className="absolute inset-0 bg-black/60 backdrop-blur-md pointer-events-auto"
                />

                {/* Floating Modal Panel */}
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
