/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Neural orchestration hub for model slot management. 
 * Controls the active slot lifecycle (Pause/Resume) and facilitates sub-navigation between primary, secondary, and tertiary cognition layers.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: State flicker when switching active tabs while an agent is 'thinking', model list empty due to provider API failure, or 'suspended' status not reflecting in the LED indicator.
 * - **Telemetry Link**: Search for `[Cognition_Section]` or `status_active` in UI tracing.
 */

import { Pause, Play, Shield, Globe, Award } from 'lucide-react';
import { Model_Slot_Config } from './Model_Slot_Config';
import { i18n } from '../../i18n';
import type { Model_Entry, Provider_Config } from '../../stores/provider_store';
import type { Skill_Manifest } from '../../services/tadpoleos_service';
import type { Skill_Definition, Mcp_Tool_Hub_Definition } from '../../stores/skill_store';

interface Slot_Definition {
    provider: string;
    model: string;
    temperature: number;
    system_prompt: string;
    skills: string[];
    workflows: string[];
}

interface Cognition_Section_Props {
    active_tab: 'primary' | 'secondary' | 'tertiary';
    slots: Record<'primary' | 'secondary' | 'tertiary', Slot_Definition>;
    agent_status: string;
    providers: Provider_Config[];
    models: Model_Entry[];
    all_skills: string[];
    all_workflows: string[];
    manifests: Skill_Manifest[];
    scripts: Skill_Definition[];
    mcp_tools: Mcp_Tool_Hub_Definition[];
    theme_color: string;
    on_set_tab: (tab: 'primary' | 'secondary' | 'tertiary') => void;
    on_update_slot_field: (slot: 'primary' | 'secondary' | 'tertiary', field: string, value: string | number | string[]) => void;
    on_toggle_skill: (slot: 'primary' | 'secondary' | 'tertiary', kind: 'skills' | 'workflows', value: string) => void;
    on_provider_change: (slot: 'primary' | 'secondary' | 'tertiary', val: string) => void;
    on_pause: () => void;
    on_resume: () => void;
}

/**
 * Cognition_Section
 * Handles the cognitive configuration of an agent, including model slots and core logic.
 * Manages the high-level operational state and model orchestration.
 */
export function Cognition_Section({
    active_tab,
    slots,
    agent_status,
    providers,
    models,
    all_skills,
    all_workflows,
    manifests,
    scripts,
    mcp_tools,
    theme_color,
    on_set_tab,
    on_update_slot_field,
    on_toggle_skill,
    on_provider_change,
    on_pause,
    on_resume
}: Cognition_Section_Props) {
    const is_paused = agent_status === 'suspended';

    const render_tab_button = (id: 'primary' | 'secondary' | 'tertiary', label: string, icon: React.ReactNode) => (
        <button
            onClick={() => on_set_tab(id)}
            className={`flex-1 flex flex-col items-center gap-1.5 py-3 rounded-xl border transition-all relative overflow-hidden group ${active_tab === id ? 'bg-zinc-800 border-zinc-700 shadow-lg' : 'bg-transparent border-transparent text-zinc-600 hover:text-zinc-400 hover:bg-zinc-800/30'}`}
        >
            {active_tab === id && (
                <div 
                    className="absolute top-0 left-0 w-full h-0.5" 
                    style={{ background: `linear-gradient(to right, transparent, ${theme_color}80, transparent)` }}
                />
            )}
            <div 
                className={`p-1.5 rounded-lg transition-colors ${active_tab === id ? '' : 'bg-zinc-900 group-hover:bg-zinc-800'}`}
                style={active_tab === id ? { backgroundColor: `${theme_color}15`, color: theme_color } : {}}
            >
                {icon}
            </div>
            <span className="text-[10px] font-bold uppercase tracking-[0.2em] leading-none" style={active_tab === id ? { color: theme_color } : {}}>{label}</span>
        </button>
    );

    return (
        <div className="p-4 space-y-6 animate-in fade-in duration-300">
            <div className="space-y-4">
                <div className="flex items-center gap-3">
                    {render_tab_button('primary', i18n.t('agent_config.tab_primary'), <Shield size={14} />)}
                    {render_tab_button('secondary', i18n.t('agent_config.tab_secondary'), <Globe size={14} />)}
                    {render_tab_button('tertiary', i18n.t('agent_config.tab_tertiary'), <Award size={14} />)}
                </div>

                <div className="p-5 bg-zinc-900 border border-zinc-800 rounded-2xl overflow-hidden group">
                    <div className="flex items-center justify-between mb-6 pb-4 border-b border-zinc-800/50">
                        <div className="flex items-center gap-2">
                            <h3 className="text-[10px] font-bold text-zinc-500 uppercase tracking-[0.2em]">
                                {i18n.t(`agent_config.slot_${active_tab}`)}
                            </h3>
                        </div>
                        <div className="flex items-center gap-1.5">
                            <div className={`w-1.5 h-1.5 rounded-full animate-pulse shadow-[0_0_8px] ${is_paused ? 'bg-amber-500 shadow-amber-500/50' : 'bg-emerald-500 shadow-emerald-500/50'}`} />
                            <span className="text-[10px] font-bold text-zinc-300 uppercase tracking-[0.2em]">{is_paused ? i18n.t('agent_config.status_suspended') : i18n.t('agent_config.status_active')}</span>
                            <div className="h-4 w-px bg-zinc-800 mx-1.5" />
                             <button
                            onClick={is_paused ? on_resume : on_pause}
                            aria-label={is_paused ? i18n.t('agent_config.btn_resume') : i18n.t('agent_config.btn_pause')}
                            title={is_paused ? i18n.t('agent_config.btn_resume') : i18n.t('agent_config.btn_pause')}
                            className={`p-1.5 rounded-lg transition-all ${is_paused ? 'bg-emerald-500/10 text-emerald-500 hover:bg-emerald-500/20' : 'bg-amber-500/10 text-amber-500 hover:bg-amber-500/20'}`}
                        >
                            {is_paused ? <Play size={14} /> : <Pause size={14} />}
                        </button>
                        </div>
                    </div>

                    <Model_Slot_Config
                        slot_key={active_tab}
                        slot={slots[active_tab]}
                        providers={providers}
                        models={models}
                        all_skills={all_skills}
                        all_workflows={all_workflows}
                        manifests={manifests}
                        scripts={scripts}
                        mcp_tools={mcp_tools}
                        theme_color={theme_color}
                        on_update_field={(field: string, value: string | number | string[]) => on_update_slot_field(active_tab, field, value)}
                        on_toggle_capability={(kind: 'skills' | 'workflows', value: string) => on_toggle_skill(active_tab, kind, value)}
                        on_provider_change={(val: string) => on_provider_change(active_tab, val)}
                    />
                </div>
            </div>
        </div>
    );
}

