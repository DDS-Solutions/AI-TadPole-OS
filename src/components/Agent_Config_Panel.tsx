import { useEffect, useState, useMemo, useCallback } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { use_model_store } from '../stores/model_store';
import { use_provider_store } from '../stores/provider_store';
import { use_role_store } from '../stores/role_store';
import { use_skill_store } from '../stores/skill_store';
import { tadpole_os_service } from '../services/tadpoleos_service';
import type { Agent } from '../types';
import { i18n } from '../i18n';
import type { Skill_Manifest } from '../services/mission_api_service';
import type { Skill_Definition, Mcp_Tool_Hub_Definition } from '../stores/skill_store';

interface Memory_Entry {
    id: string;
    content?: string;
    text?: string;
}

// Decomposed Components
import {
    Agent_Config_Header,
    Cognition_Section,
    Voice_Section,
    Governance_Section,
    Memory_Section,
    Direct_Message_Console,
    use_agent_config
} from './agent-config';
import { Portal_Window } from './ui/Portal_Window';
import { ExternalLink } from 'lucide-react';

interface Agent_Config_Panel_Props {
    agent: Agent | undefined;
    on_close: () => void;
    on_update: (id: string, updates: Partial<Agent>) => void;
    is_new?: boolean;
}

/**
 * Agent_Config_Panel
 * Main orchestration component for agent configuration.
 * Manages tab state, detached window logic, and capability synchronization.
 */
export default function Agent_Config_Panel({ agent, on_close, on_update, is_new = false }: Agent_Config_Panel_Props) {
    const {
        state,
        dispatch,
        handle_role_change,
        handle_provider_change,
        handle_save,
        handle_pause,
        handle_resume,
        handle_send_message,
        handle_promote
    } = use_agent_config(agent, on_update, on_close);

    const { identity, slots, voice, ui, governance, main_tab, active_tab } = state;

    // External Stores
    const providers = use_provider_store(s => s.providers);
    const models = use_model_store(s => s.models);
    const roles = use_role_store(s => s.roles);

    // Stable selectors for skill store
    const manifests = use_skill_store(s => s.manifests);
    const scripts = use_skill_store(s => s.scripts);
    const workflows = use_skill_store(s => s.workflows);
    const mcp_tools = use_skill_store(s => s.mcp_tools);
    const fetch_skills = use_skill_store(s => s.fetch_skills);
    const fetch_mcp_tools = use_skill_store(s => s.fetch_mcp_tools);
    const is_loading = use_skill_store(s => s.is_loading);

    // Local state for memories (separate from config form)
    const [memories, set_memories] = useState<Memory_Entry[]>([]);
    const [loading_memories, set_loading_memories] = useState(false);
    const [memory_input, set_memory_input] = useState('');
    const [is_detached, set_is_detached] = useState(false);

    useEffect(() => {
        if (is_loading) return;

        // Fetch capabilities if they haven't been loaded yet
        if (manifests.length === 0 && scripts.length === 0) {
            fetch_skills();
        }
        if (mcp_tools.length === 0) {
            fetch_mcp_tools();
        }
    }, [fetch_skills, fetch_mcp_tools, manifests.length, scripts.length, mcp_tools.length, is_loading]);

    const load_memories = useCallback(async () => {
        if (!agent?.id) return;
        set_loading_memories(true);
        try {
            const response = await tadpole_os_service.get_agent_memory(agent.id) as { entries: Memory_Entry[] };
            set_memories(response.entries || []);
        } finally {
            set_loading_memories(false);
        }
    }, [agent?.id]);

    useEffect(() => {
        if (agent?.id && main_tab === 'memory') {
            load_memories();
        }
    }, [agent?.id, main_tab, load_memories]);

    const handle_save_memory = async () => {
        if (!memory_input.trim() || !agent?.id) return;
        await tadpole_os_service.save_agent_memory(agent.id, memory_input);
        set_memory_input('');
        load_memories();
    };

    const handle_delete_memory = async (id: string) => {
        if (!agent?.id) return;
        await tadpole_os_service.delete_agent_memory(agent.id, id);
        load_memories();
    };

    const all_skills = useMemo(() => {
        const names = new Set([
            ...manifests.map((m: Skill_Manifest) => m.name),
            ...scripts.map((s: Skill_Definition) => s.name),
            ...mcp_tools.map((t: Mcp_Tool_Hub_Definition) => t.name)
        ]);
        return Array.from(names).sort((a, b) => a.localeCompare(b));
    }, [manifests, scripts, mcp_tools]);

    const all_workflows = useMemo(() => workflows.map((w: { name: string }) => w.name), [workflows]);

    if (!agent && !ui.saving) return null;

    const panel_content = (
        <div className="flex-1 flex flex-col min-h-0 bg-zinc-950/40 backdrop-blur-xl">
            <Agent_Config_Header
                name={identity.name}
                role={identity.role}
                theme_color={ui.theme_color}
                is_new={is_new || !agent?.id}
                agent_id={agent?.id}
                available_roles={Object.keys(roles)}
                on_close={on_close}
                on_detach={() => set_is_detached(true)}
                is_detached={is_detached}
                on_update_identity={(field, value) => dispatch({ type: 'UPDATE_IDENTITY', field, value: value as string })}
                on_update_theme_color={(color) => dispatch({ type: 'SET_UI', field: 'theme_color', value: color })}
                handle_role_change={handle_role_change}
            />

            <div className="flex border-b border-zinc-900 bg-zinc-900/40 px-6 shrink-0 z-10">
                <button
                    onClick={() => dispatch({ type: 'SET_MAIN_TAB', payload: 'cognition' })}
                    className={`px-4 py-3 text-[10px] font-bold uppercase tracking-[0.2em] transition-all relative ${main_tab === 'cognition' ? 'text-emerald-400' : 'text-zinc-500 hover:text-zinc-300'}`}
                >
                    {i18n.t('agent_config.tab_cognition')}
                    {main_tab === 'cognition' && <div className="absolute bottom-0 left-0 w-full h-0.5 bg-emerald-500" />}
                </button>
                <button
                    onClick={() => dispatch({ type: 'SET_MAIN_TAB', payload: 'memory' })}
                    className={`px-4 py-3 text-[10px] font-bold uppercase tracking-[0.2em] transition-all relative ${main_tab === 'memory' ? 'text-blue-400' : 'text-zinc-500 hover:text-zinc-300'}`}
                >
                    {i18n.t('agent_config.tab_memory')}
                    {main_tab === 'memory' && <div className="absolute bottom-0 left-0 w-full h-0.5 bg-blue-500" />}
                </button>
                <button
                    onClick={() => dispatch({ type: 'SET_MAIN_TAB', payload: 'governance' })}
                    className={`px-4 py-3 text-[10px] font-bold uppercase tracking-[0.2em] transition-all relative ${main_tab === 'governance' ? 'text-amber-400' : 'text-zinc-500 hover:text-zinc-300'}`}
                >
                    {i18n.t('agent_config.tab_governance')}
                    {main_tab === 'governance' && <div className="absolute bottom-0 left-0 w-full h-0.5 bg-amber-500" />}
                </button>
            </div>

            <div className="flex-1 overflow-hidden flex flex-col relative min-h-0">
                <div className="flex-1 overflow-y-auto custom-scrollbar min-h-0">
                    {main_tab === 'cognition' && (
                        <Cognition_Section
                            active_tab={active_tab}
                            slots={slots}
                            agent_status={agent?.status || 'idle'}
                            providers={providers}
                            models={models}
                            all_skills={all_skills}
                            all_workflows={all_workflows}
                            manifests={manifests}
                            scripts={scripts}
                            mcp_tools={mcp_tools}
                            theme_color={ui.theme_color}
                            on_set_tab={(tab) => dispatch({ type: 'SET_TAB', payload: tab })}
                            on_update_slot_field={(slot, field, value) => {
                                if (field === 'model' || field === 'provider' || field === 'system_prompt') {
                                    dispatch({ type: 'UPDATE_SLOT', slot, field, value: value as string });
                                } else if (field === 'temperature') {
                                    dispatch({ type: 'UPDATE_SLOT', slot, field, value: value as number });
                                } else if (field === 'skills' || field === 'workflows') {
                                    dispatch({ type: 'UPDATE_SLOT', slot, field, value: value as string[] });
                                }
                            }}
                            on_toggle_skill={(slot, kind, value) => dispatch({ type: 'TOGGLE_SKILL', slot, kind, value })}
                            on_provider_change={(slot, val) => handle_provider_change(slot, val)}
                            on_pause={handle_pause}
                            on_resume={handle_resume}
                        />
                    )}
                    {main_tab === 'memory' && (
                        <Memory_Section
                            memories={memories}
                            connector_configs={state.connector_configs}
                            is_loading={loading_memories}
                            memory_input={memory_input}
                            theme_color={ui.theme_color}
                            on_memory_input_change={set_memory_input}
                            on_save_memory={handle_save_memory}
                            on_delete_memory={handle_delete_memory}
                            on_refresh={load_memories}
                            on_add_connector={(uri) => dispatch({ type: 'ADD_CONNECTOR', payload: { type: 'fs', uri } })}
                            on_remove_connector={(uri) => dispatch({ type: 'REMOVE_CONNECTOR', uri })}
                        />
                    )}
                    {main_tab === 'governance' && (
                        <Governance_Section
                            budget_usd={governance.budget_usd}
                            requires_oversight={governance.requires_oversight}
                            cost_usd={agent?.cost_usd || 0}
                            theme_color={ui.theme_color}
                            on_update_governance={(field, value) => {
                                if (field === 'budget_usd') {
                                    dispatch({ type: 'UPDATE_GOVERNANCE', field, value: value as number });
                                } else if (field === 'requires_oversight') {
                                    dispatch({ type: 'UPDATE_GOVERNANCE', field, value: value as boolean });
                                }
                            }}
                        />
                    )}

                    <div className="px-6 py-4 space-y-6 bg-zinc-900/20 border-t border-zinc-900 shrink-0">
                        <Voice_Section
                            voice={voice}
                            stt_engine={voice.stt_engine || 'groq'}
                            theme_color={ui.theme_color}
                            on_update_voice={(field, value) => dispatch({ type: 'UPDATE_VOICE', field: field as 'voice_id' | 'voice_engine' | 'stt_engine', value })}
                        />
                    </div>
                </div>

                <Direct_Message_Console
                    value={ui.direct_message}
                    on_update_value={(val) => dispatch({ type: 'SET_UI', field: 'direct_message', value: val })}
                    on_send={handle_send_message}
                    agent_name={identity.name}
                    theme_color={ui.theme_color}
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
                                value={ui.new_role_name}
                                onChange={(e) => dispatch({ type: 'SET_UI', field: 'new_role_name', value: e.target.value })}
                                className="bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-1.5 text-xs text-zinc-300 focus:outline-none focus:border-emerald-500/50 w-40"
                            />
                            <button
                                onClick={handle_promote}
                                className="px-4 py-1.5 bg-emerald-500/10 text-emerald-500 border border-emerald-500/20 rounded-lg text-[10px] font-bold uppercase tracking-[0.2em] hover:bg-emerald-500/20 transition-all"
                            >
                                {i18n.t('agent_config.btn_confirm')}
                            </button>
                        </div>
                    )}

                    <button
                        onClick={handle_save}
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
                        ) : (is_new ? i18n.t('agent_config.btn_create_agent') : i18n.t('agent_config.btn_save_config'))}
                    </button>
                </div>
            </div>
        </div>
    );

    if (is_detached) {
        return (
            <AnimatePresence>
                <div className="fixed inset-0 z-[100] flex items-center justify-center p-4 pointer-events-none">
                    <Portal_Window
                        id={`agent-config-${agent?.id || 'new'}`}
                        title={identity.name || 'New Agent'}
                        on_close={() => set_is_detached(false)}
                    >
                        {panel_content}
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
                            <div className="absolute inset-0 bg-blue-500/10 blur-2xl rounded-full" />
                        </div>
                        <div className="text-center space-y-2 px-6">
                            <h3 className="text-xl font-bold tracking-tight text-zinc-200">{i18n.t('layout.sector_detached')}</h3>
                            <p className="text-sm text-zinc-500 font-mono uppercase tracking-[0.2em]">
                                {i18n.t('layout.link_established')} :: ID_{agent?.id?.substring(0, 8).toUpperCase() || 'NEW'}
                            </p>
                        </div>
                        <button
                            onClick={() => set_is_detached(false)}
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
                <motion.div
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    exit={{ opacity: 0 }}
                    onClick={on_close}
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
                        {panel_content}
                    </div>
                </motion.div>
            </div>
        </AnimatePresence>
    );
}
