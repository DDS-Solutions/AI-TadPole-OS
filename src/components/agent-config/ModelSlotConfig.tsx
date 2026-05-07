/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Deep configuration layer for individual model slots. 
 * Manages provider-specific model selections, temperature hyper-parameters, and capability (Skill/Workflow) toggling.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Provider list empty (loading race condition), skill selection desync when toggling multiple rapidly, or temperature slider rounding errors.
 * - **Telemetry Link**: Search for `[Model_Slot_Config]` or `PROVIDER_SWITCH` in browser tracing.
 */

import { Info } from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';
import type { Model_Entry, Provider_Config } from '../../stores/provider_store';
import type { Skill_Manifest } from '../../services/tadpoleos_service';
import type { Skill_Definition, Mcp_Tool_Hub_Definition } from '../../stores/skill_store';
import type { Agent_Model_Slot_Key, Agent_Model_Slot_State } from '../../types';

interface ModelSlotConfigProps {
    slotKey: Agent_Model_Slot_Key;
    slot: Agent_Model_Slot_State;
    providers: Provider_Config[];
    models: Model_Entry[];
    allSkills: string[];
    allWorkflows: string[];
    manifests: Skill_Manifest[];
    scripts: Skill_Definition[];
    mcpTools: Mcp_Tool_Hub_Definition[]; 
    themeColor: string;
    onUpdateField: <K extends keyof Agent_Model_Slot_State>(field: K, value: Agent_Model_Slot_State[K]) => void;
    onToggleCapability: (kind: 'skills' | 'workflows', value: string) => void;
    onProviderChange: (val: string) => void;
}

/**
 * Model_Slot_Config
 * Handles the configuration of a specific model slot (Primary, Secondary, Tertiary).
 * Manages provider selection, model selection, temperature, and capability (skills/workflows) toggling.
 */
export function ModelSlotConfig({
    slotKey,
    slot,
    providers,
    models,
    allSkills,
    allWorkflows,
    manifests,
    scripts,
    mcpTools,
    themeColor,
    onUpdateField,
    onToggleCapability,
    onProviderChange
}: ModelSlotConfigProps) {
    const filteredModels = models
        .filter(m => m.provider === slot.provider)
        .sort((a, b) => a.name.localeCompare(b.name));

    return (
        <div className="space-y-5 animate-in fade-in duration-300" key={slotKey}>
            <div className="grid grid-cols-2 gap-3">
                <div className="space-y-1.5">
                    <label className="block text-[10px] font-bold text-zinc-500 uppercase tracking-[0.2em] flex items-center gap-1.5">
                        {i18n.t('agent_config.label_provider')}
                        <Tooltip content={i18n.t('agent_config.tooltip_provider')} position="top">
                            <Info size={9} className="text-zinc-700 hover:text-zinc-300 cursor-help transition-colors" />
                        </Tooltip>
                    </label>
                    <select
                        value={slot.provider}
                        onChange={(e) => onProviderChange(e.target.value)}
                        className="w-full bg-zinc-900 border border-zinc-800 rounded-lg px-3 py-2 text-xs text-zinc-200 focus:outline-none focus:border-zinc-700 cursor-pointer font-bold appearance-none transition-all"
                        style={{ borderLeft: `2px solid ${themeColor}40` }}
                    >
                        {providers.map(p => (
                            <option key={p.id} value={p.id} className="bg-zinc-950">
                                {p.name.toUpperCase()}
                            </option>
                        ))}
                    </select>
                </div>

                <div className="space-y-1.5">
                    <label className="block text-[10px] font-bold text-zinc-500 uppercase tracking-[0.2em] flex items-center gap-1.5">
                        {i18n.t('agent_config.label_model')}
                        <Tooltip content={i18n.t('agent_config.tooltip_model')} position="top">
                            <Info size={9} className="text-zinc-700 hover:text-zinc-300 cursor-help transition-colors" />
                        </Tooltip>
                    </label>
                    <select
                        value={slot.model}
                        onChange={(e) => onUpdateField('model', e.target.value)}
                        className="w-full bg-zinc-900 border border-zinc-800 rounded-lg px-3 py-2 text-xs text-zinc-200 focus:outline-none focus:border-zinc-700 font-mono cursor-pointer appearance-none transition-all"
                        style={{ borderLeft: `2px solid ${themeColor}40` }}
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
                <label className="block text-[10px] font-bold text-zinc-500 uppercase tracking-[0.2em] mb-1.5 flex items-center gap-1.5">
                    {i18n.t('agent_config.label_temperature')} <span className="font-mono text-[10px]" style={{ color: themeColor }}>{slot.temperature.toFixed(2)}</span>
                    <Tooltip content={i18n.t('agent_config.tooltip_temperature')} position="top">
                        <Info size={10} className="text-zinc-700 hover:text-zinc-400 cursor-help transition-colors" />
                    </Tooltip>
                </label>
                <input 
                    type="range" 
                    min="0" 
                    max="2" 
                    step="0.05" 
                    value={slot.temperature} 
                    onChange={(e) => onUpdateField('temperature', parseFloat(e.target.value))} 
                    className="w-full accent-zinc-500" 
                    style={{ accentColor: themeColor }}
                />
            </div>

            <div className="grid grid-cols-2 gap-4">
                <div className="space-y-1.5">
                    <label className="block text-[10px] font-bold text-zinc-500 uppercase tracking-[0.2em] mb-1.5 flex items-center gap-1.5">
                        {i18n.t('agent_config.label_reasoning_depth')} <span className="font-mono text-[10px]" style={{ color: themeColor }}>{slot.reasoning_depth || 1}x</span>
                        <Tooltip content={i18n.t('agent_config.tooltip_reasoning_depth')} position="top">
                            <Info size={10} className="text-zinc-700 hover:text-zinc-400 cursor-help transition-colors" />
                        </Tooltip>
                    </label>
                    <input 
                        type="range" 
                        min="1" 
                        max="16" 
                        step="1" 
                        value={slot.reasoning_depth || 1} 
                        onChange={(e) => onUpdateField('reasoning_depth', parseInt(e.target.value))} 
                        className="w-full accent-zinc-500" 
                        style={{ accentColor: themeColor }}
                    />
                </div>

                <div className="space-y-1.5">
                    <label className="block text-[10px] font-bold text-zinc-500 uppercase tracking-[0.2em] mb-1.5 flex items-center gap-1.5">
                        {i18n.t('agent_config.label_act_threshold')} <span className="font-mono text-[10px]" style={{ color: themeColor }}>{((slot.act_threshold || 0.9) * 100).toFixed(0)}%</span>
                        <Tooltip content={i18n.t('agent_config.tooltip_act_threshold')} position="top">
                            <Info size={10} className="text-zinc-700 hover:text-zinc-400 cursor-help transition-colors" />
                        </Tooltip>
                    </label>
                    <input 
                        type="range" 
                        min="0" 
                        max="1" 
                        step="0.01" 
                        value={slot.act_threshold || 0.9} 
                        onChange={(e) => onUpdateField('act_threshold', parseFloat(e.target.value))} 
                        className="w-full accent-zinc-500" 
                        style={{ accentColor: themeColor }}
                    />
                </div>
            </div>

            <div>
                <label className="block text-[10px] font-bold text-zinc-500 uppercase tracking-[0.2em] mb-1.5 flex items-center gap-1.5">
                    {i18n.t('agent_config.label_system_prompt')}
                    <Tooltip content={i18n.t('agent_config.tooltip_system_prompt')} position="top">
                        <Info size={10} className="text-zinc-700 hover:text-zinc-400 cursor-help transition-colors" />
                    </Tooltip>
                </label>
                <textarea
                    value={slot.system_prompt}
                    onChange={(e) => onUpdateField('system_prompt', e.target.value)}
                    rows={6}
                    className="w-full bg-zinc-900 border border-zinc-800 rounded-lg px-3 py-2 text-xs text-zinc-200 focus:outline-none focus:border-zinc-700 font-mono resize-none custom-scrollbar transition-all"
                    style={{ borderLeft: `3px solid ${themeColor}20` }}
                />
            </div>

            {/* Skills & Workflows Row */}
            <div className="grid grid-cols-2 gap-4 pt-4 border-t border-zinc-800/50">
                <div className="flex flex-col h-48">
                    <label className="block text-[10px] font-bold text-zinc-500 uppercase tracking-[0.2em] mb-2 flex items-center gap-1.5">
                        {i18n.t('agent_config.label_skills', { count: slot.skills.length })}
                        <Tooltip content={i18n.t('agent_config.tooltip_skills')} position="top">
                            <Info size={10} className="text-zinc-700 cursor-help transition-colors" style={{ color: themeColor }} />
                        </Tooltip>
                    </label>
                    <div className="flex-1 overflow-y-auto custom-scrollbar border border-zinc-800 rounded-lg bg-zinc-900/30 p-2 space-y-2">
                        {allSkills.map(skillName => {
                            const manifest = manifests.find((m) => m.name === skillName);
                            const script = scripts.find((s) => s.name === skillName);
                            const mcpTool = mcpTools.find(t => t.name === skillName);
                            const isSelected = slot.skills.includes(skillName);

                            const getDangerColor = (level?: string) => {
                                if (level === 'critical') return 'text-red-500 bg-red-500/10 border-red-500/20';
                                if (level === 'high') return 'text-orange-500 bg-orange-500/10 border-orange-500/20';
                                if (level === 'medium') return 'text-yellow-500 bg-yellow-500/10 border-yellow-500/20';
                                return ''; // Default handled by manifest existence
                            };
                            const dangerClasses = getDangerColor(manifest?.danger_level);

                            return (
                                <div
                                    key={skillName}
                                    onClick={() => onToggleCapability('skills', skillName)}
                                    className={`w-full flex flex-col p-2.5 rounded-lg border text-left cursor-pointer transition-all ${isSelected ? 'bg-zinc-800 border-zinc-600' : 'bg-transparent border-transparent hover:bg-zinc-800/50'}`}
                                >
                                    <div className="flex items-start justify-between gap-2 overflow-hidden">
                                        <div className="flex items-center gap-2 min-w-0">
                                            <div 
                                                className={`w-3 h-3 rounded-full border shrink-0 transition-colors ${isSelected ? '' : 'border-zinc-600 bg-zinc-950'}`}
                                                style={isSelected ? { backgroundColor: themeColor, borderColor: themeColor } : {}}
                                            />
                                            <span className={`text-xs font-bold truncate ${isSelected ? 'text-zinc-100' : 'text-zinc-400'}`}>
                                                {manifest?.display_name || script?.name || skillName}
                                            </span>
                                        </div>
                                        <div className="flex items-center gap-1.5 shrink-0">
                                            {mcpTool && <span className="text-[8px] font-bold px-1 py-0.5 rounded bg-green-500/10 text-green-400 border border-green-500/20 uppercase tracking-tighter">{i18n.t('agent_config.label_mcp')}</span>}
                                            {manifest ? (
                                                <span className={`text-[9px] font-mono px-1.5 py-0.5 rounded border uppercase tracking-tighter ${dangerClasses}`}>
                                                    {manifest.danger_level}
                                                </span>
                                            ) : script ? (
                                                <span className="text-[8px] font-bold px-1 py-0.5 rounded bg-zinc-800 text-zinc-400 border border-zinc-700 uppercase tracking-tighter">Script</span>
                                            ) : !mcpTool && <span className="text-[8px] font-bold px-1 py-0.5 rounded bg-zinc-800 text-zinc-500 border border-zinc-700 uppercase tracking-tighter">{i18n.t('agent_config.label_core')}</span>}
                                        </div>
                                    </div>
                                    {(manifest?.description || script?.description || mcpTool?.description) && (
                                        <div className="mt-1.5 pl-5">
                                            <p className="text-[10px] text-zinc-500/60 leading-tight mb-1">
                                                {manifest?.description || script?.description || mcpTool?.description}
                                            </p>
                                        </div>
                                    )}
                                </div>
                            );
                        })}
                    </div>
                </div>
                <div className="flex flex-col h-48">
                    <label className="block text-[10px] font-bold text-zinc-500 uppercase tracking-[0.2em] mb-2 flex items-center gap-1.5">
                        {i18n.t('agent_config.label_workflows', { count: slot.workflows.length })}
                        <Tooltip content={i18n.t('agent_config.tooltip_workflows')} position="top">
                            <Info size={10} className="text-zinc-700 hover:text-amber-500 cursor-help transition-colors" />
                        </Tooltip>
                    </label>
                    <div className="flex-1 overflow-y-auto custom-scrollbar border border-zinc-800 rounded-lg bg-zinc-900/30 p-2 space-y-1">
                        {allWorkflows.map(wf => (
                            <button
                                key={wf}
                                onClick={() => onToggleCapability('workflows', wf)}
                                className={`w-full flex items-center min-h-[44px] gap-2 p-2 rounded text-xs transition-colors ${slot.workflows.includes(wf) ? 'bg-amber-500/10 text-amber-400' : 'text-zinc-400 hover:bg-zinc-800'}`}
                            >
                                <div className={`w-2 h-2 rounded-full border ${slot.workflows.includes(wf) ? 'bg-amber-500 border-amber-500' : 'border-zinc-600'}`} />
                                {wf}
                            </button>
                        ))}
                    </div>
                </div>
            </div>
        </div>
    );
}


// Metadata: [ModelSlotConfig]

// Metadata: [ModelSlotConfig]
