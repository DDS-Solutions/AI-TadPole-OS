/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Agent-Config / CognitionSection
 * - **Primary Entrypoints**: `CognitionSection`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { Pause, Play, Shield, Zap, Brain } from 'lucide-react';
import { ModelSlotConfig } from './ModelSlotConfig';
import { Tooltip } from '../ui';
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
    activeModelSlot?: number;
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
    activeModelSlot,
    onSetTab,
    onUpdateSlotField,
    onToggleSkill,
    onProviderChange,
    onPause,
    onResume
}: CognitionSectionProps) {
    const isPaused = agentStatus === 'suspended';

    const isWorkingSlot = (id: Agent_Model_Slot_Key) => {
        const slotIdx = activeModelSlot ?? 1;
        if (id === 'primary') return slotIdx === 1;
        if (id === 'secondary') return slotIdx === 2;
        if (id === 'tertiary') return slotIdx === 3;
        return false;
    };

    const renderTabButton = (
        id: Agent_Model_Slot_Key,
        slotNumber: string,
        label: string,
        subLabel: string,
        tooltipText: string,
        icon: React.ReactNode
    ) => {
        const isWorking = isWorkingSlot(id);
        const isActive = activeTab === id;

        return (
            <Tooltip content={tooltipText} position="top">
                <button
                    onClick={() => onSetTab(id)}
                    className={`w-full flex flex-col items-center gap-1.5 py-2.5 px-3 rounded-xl border transition-all relative overflow-hidden group ${
                        isActive
                            ? 'bg-zinc-800/90 border-zinc-700 shadow-[0_0_15px_rgba(0,0,0,0.4)] backdrop-blur-md'
                            : 'bg-[color:var(--color-surface)]/50 border-[color:var(--color-border)] text-zinc-500 hover:text-zinc-300 hover:bg-zinc-800/40 hover:border-zinc-700'
                    }`}
                >
                    {isActive && (
                        <div 
                            className="absolute top-0 left-0 w-full h-0.5" 
                            style={{ background: `linear-gradient(to right, transparent, ${themeColor}cc, transparent)` }}
                        />
                    )}
                    <div className="flex items-center gap-1.5 w-full justify-between">
                        <span className="text-[9px] font-mono font-bold tracking-widest text-zinc-500 uppercase">
                            {slotNumber}
                        </span>
                        <div 
                            className={`p-1 rounded-md transition-colors relative ${isActive ? '' : 'bg-zinc-900/60 group-hover:bg-zinc-800'}`}
                            style={isActive ? { backgroundColor: `${themeColor}20`, color: themeColor } : {}}
                        >
                            {icon}
                            {isWorking && (
                                <div 
                                    className="absolute -top-0.5 -right-0.5 w-1.5 h-1.5 rounded-full bg-emerald-500 shadow-[0_0_6px_#10b981] animate-pulse" 
                                    title={i18n.t('agent_config.slot_active_indicator') || 'Currently active execution slot'}
                                />
                            )}
                        </div>
                    </div>
                    <div className="flex flex-col items-center w-full">
                        <span 
                            className="text-[11px] font-bold uppercase tracking-wider leading-none" 
                            style={isActive ? { color: themeColor } : { color: '#e4e4e7' }}
                        >
                            {label}
                        </span>
                        <span className="text-[8px] font-medium text-zinc-400/80 tracking-tight leading-tight mt-1 truncate max-w-full">
                            {subLabel}
                        </span>
                    </div>
                </button>
            </Tooltip>
        );
    };

    return (
        <div className="p-4 space-y-6 animate-in fade-in duration-300">
            <div className="space-y-4">
                <div className="grid grid-cols-3 gap-3">
                    {renderTabButton(
                        'primary',
                        i18n.t('agent_config.tab_primary_slot') || 'SLOT 1',
                        i18n.t('agent_config.tab_primary'),
                        i18n.t('agent_config.tab_primary_desc') || 'Core Intelligence',
                        i18n.t('agent_config.tooltip_tab_primary'),
                        <Shield size={13} />
                    )}
                    {renderTabButton(
                        'secondary',
                        i18n.t('agent_config.tab_secondary_slot') || 'SLOT 2',
                        i18n.t('agent_config.tab_secondary'),
                        i18n.t('agent_config.tab_secondary_desc') || 'Fast Tool & Swarm',
                        i18n.t('agent_config.tooltip_tab_secondary'),
                        <Zap size={13} />
                    )}
                    {renderTabButton(
                        'tertiary',
                        i18n.t('agent_config.tab_tertiary_slot') || 'SLOT 3',
                        i18n.t('agent_config.tab_tertiary'),
                        i18n.t('agent_config.tab_tertiary_desc') || 'Deep Reasoning & Audit',
                        i18n.t('agent_config.tooltip_tab_tertiary'),
                        <Brain size={13} />
                    )}
                </div>

                <div className="p-5 bg-[color:var(--color-surface)] border border-[color:var(--color-border)] rounded-2xl overflow-hidden group">
                    <div className="flex items-center justify-between mb-6 pb-4 border-b border-[color:var(--color-border)]/50">
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
