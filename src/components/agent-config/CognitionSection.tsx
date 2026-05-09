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
import { ModelSlotConfig } from './ModelSlotConfig';
import { i18n } from '../../i18n';
import type { Model_Entry, Provider_Config } from '../../stores/provider_store';
import type { Skill_Manifest } from '../../services/tadpoleos_service';
import type { Skill_Definition, Mcp_Tool_Hub_Definition } from '../../stores/skill_store';
import type { Agent_Model_Slot_Key, Agent_Model_Slot_State } from '../../types';

interface CognitionSectionProps {
    activeTab: Agent_Model_Slot_Key;
    slots: Record<Agent_Model_Slot_Key, Agent_Model_Slot_State>;
    agentStatus: string;
    providers: Provider_Config[];
    models: Model_Entry[];
    allSkills: string[];
    allWorkflows: string[];
    manifests: Skill_Manifest[];
    scripts: Skill_Definition[];
    mcpTools: Mcp_Tool_Hub_Definition[];
    themeColor: string;
    onSetTab: (tab: Agent_Model_Slot_Key) => void;
    onUpdateSlotField: <K extends keyof Agent_Model_Slot_State>(slot: Agent_Model_Slot_Key, field: K, value: Agent_Model_Slot_State[K]) => void;
    onToggleSkill: (slot: Agent_Model_Slot_Key, kind: 'skills' | 'workflows', value: string) => void;
    onProviderChange: (slot: Agent_Model_Slot_Key, val: string) => void;
    onPause: () => void;
    onResume: () => void;
}

/**
 * Cognition_Section
 * Handles the cognitive configuration of an agent, including model slots and core logic.
 * Manages the high-level operational state and model orchestration.
 */
export function CognitionSection({
    activeTab,
    slots,
    agentStatus,
    providers,
    models,
    allSkills,
    allWorkflows,
    manifests,
    scripts,
    mcpTools,
    themeColor,
    onSetTab,
    onUpdateSlotField,
    onToggleSkill,
    onProviderChange,
    onPause,
    onResume
}: CognitionSectionProps) {
    const isPaused = agentStatus === 'suspended';

    const renderTabButton = (id: Agent_Model_Slot_Key, label: string, icon: React.ReactNode) => (
        <button
            onClick={() => onSetTab(id)}
            className={`flex-1 flex flex-col items-center gap-1.5 py-3 rounded-xl border transition-all relative overflow-hidden group ${activeTab === id ? 'bg-zinc-800 border-zinc-700 shadow-lg' : 'bg-transparent border-transparent text-zinc-600 hover:text-zinc-400 hover:bg-zinc-800/30'}`}
        >
            {activeTab === id && (
                <div 
                    className="absolute top-0 left-0 w-full h-0.5" 
                    style={{ background: `linear-gradient(to right, transparent, ${themeColor}80, transparent)` }}
                />
            )}
            <div 
                className={`p-1.5 rounded-lg transition-colors ${activeTab === id ? '' : 'bg-zinc-900 group-hover:bg-zinc-800'}`}
                style={activeTab === id ? { backgroundColor: `${themeColor}15`, color: themeColor } : {}}
            >
                {icon}
            </div>
            <span className="text-[10px] font-bold uppercase tracking-[0.2em] leading-none" style={activeTab === id ? { color: themeColor } : {}}>{label}</span>
        </button>
    );

    return (
        <div className="p-4 space-y-6 animate-in fade-in duration-300">
            <div className="space-y-4">
                <div className="flex items-center gap-3">
                    {renderTabButton('primary', i18n.t('agent_config.tab_primary'), <Shield size={14} />)}
                    {renderTabButton('secondary', i18n.t('agent_config.tab_secondary'), <Globe size={14} />)}
                    {renderTabButton('tertiary', i18n.t('agent_config.tab_tertiary'), <Award size={14} />)}
                </div>

                <div className="p-5 bg-zinc-900 border border-zinc-800 rounded-2xl overflow-hidden group">
                    <div className="flex items-center justify-between mb-6 pb-4 border-b border-zinc-800/50">
                        <div className="flex items-center gap-2">
                            <h3 className="text-[10px] font-bold text-zinc-500 uppercase tracking-[0.2em]">
                                {i18n.t(`agent_config.slot_${activeTab}`)}
                            </h3>
                        </div>
                        <div className="flex items-center gap-1.5">
                            <div className={`w-1.5 h-1.5 rounded-full animate-pulse shadow-[0_0_8px] ${isPaused ? 'bg-amber-500 shadow-amber-500/50' : 'bg-emerald-500 shadow-emerald-500/50'}`} />
                            <span className="text-[10px] font-bold text-zinc-300 uppercase tracking-[0.2em]">{isPaused ? i18n.t('agent_config.status_suspended') : i18n.t('agent_config.status_active')}</span>
                            <div className="h-4 w-px bg-zinc-800 mx-1.5" />
                             <button
                            onClick={isPaused ? onResume : onPause}
                            aria-label={isPaused ? i18n.t('agent_config.btn_resume') : i18n.t('agent_config.btn_pause')}
                            title={isPaused ? i18n.t('agent_config.btn_resume') : i18n.t('agent_config.btn_pause')}
                            className={`p-1.5 rounded-lg transition-all ${isPaused ? 'bg-emerald-500/10 text-emerald-500 hover:bg-emerald-500/20' : 'bg-amber-500/10 text-amber-500 hover:bg-amber-500/20'}`}
                        >
                            {isPaused ? <Play size={14} /> : <Pause size={14} />}
                        </button>
                        </div>
                    </div>

                    <ModelSlotConfig
                        slotKey={activeTab}
                        slot={slots[activeTab]}
                        providers={providers}
                        models={models}
                        allSkills={allSkills}
                        allWorkflows={allWorkflows}
                        manifests={manifests}
                        scripts={scripts}
                        mcpTools={mcpTools}
                        themeColor={themeColor}
                        onUpdateField={(field, value) => onUpdateSlotField(activeTab, field, value)}
                        onToggleCapability={(kind: 'skills' | 'workflows', value: string) => onToggleSkill(activeTab, kind, value)}
                        onProviderChange={(val: string) => onProviderChange(activeTab, val)}
                    />
                </div>
            </div>
        </div>
    );
}


// Metadata: [Cognition_Section]

// Metadata: [CognitionSection]

// Metadata: [CognitionSection]
