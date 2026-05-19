/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Architecture_Settings]` in observability traces.
 */

import React from 'react';
import { Cpu } from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';
import type { Tadpole_Settings } from '../../stores/settings_store';

interface ArchitectureSettingsProps {
    settings: Tadpole_Settings;
    handle_numeric_change: (name: string, value: number) => void;
}

export const Architecture_Settings: React.FC<ArchitectureSettingsProps> = ({ settings, handle_numeric_change }) => {
    return (
        <div className="space-y-4 pb-12">
            <h2 className="text-sm font-bold text-zinc-500 uppercase tracking-widest flex items-center gap-2">
                {i18n.t('settings.header_architecture')}
            </h2>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-8 bg-[color:var(--color-surface)] py-8 pl-8 pr-32 rounded-xl border border-[color:var(--color-border)] shadow-sm relative overflow-hidden group">
                <div className="absolute top-0 right-0 p-6 opacity-5 group-hover:opacity-10 transition-opacity pointer-events-none">
                    <Cpu size={80} />
                </div>

                <div className="space-y-4 z-10 relative">
                    <div className="space-y-2">
                        <div className="flex justify-between items-center">
                            <Tooltip content={i18n.t('settings.tooltip_max_agents')} position="top">
                                <label htmlFor="max_agents" className="text-sm font-bold text-zinc-300 cursor-help w-max">{i18n.t('settings.label_max_agents')}</label>
                            </Tooltip>
                            <span className="text-xs font-mono text-zinc-400 bg-zinc-800/50 px-2 py-1 rounded">{settings.max_agents}</span>
                        </div>
                        <input
                            id="max_agents"
                            type="range"
                            name="max_agents"
                            min="1"
                            max="200"
                            value={settings.max_agents}
                            onChange={(e) => handle_numeric_change('max_agents', parseInt(e.target.value))}
                            className="w-full h-1.5 bg-zinc-800 rounded-lg appearance-none cursor-pointer accent-green-500"
                        />
                    </div>

                    <div className="space-y-2">
                        <div className="flex justify-between items-center">
                            <Tooltip content={i18n.t('settings.tooltip_max_clusters')} position="top">
                                <label htmlFor="max_clusters" className="text-sm font-bold text-zinc-300 cursor-help w-max">{i18n.t('settings.label_max_clusters')}</label>
                            </Tooltip>
                            <span className="text-xs font-mono text-zinc-400 bg-zinc-800/50 px-2 py-1 rounded">{settings.max_clusters}</span>
                        </div>
                        <input
                            id="max_clusters"
                            type="range"
                            name="max_clusters"
                            min="1"
                            max="50"
                            value={settings.max_clusters}
                            onChange={(e) => handle_numeric_change('max_clusters', parseInt(e.target.value))}
                            className="w-full h-1.5 bg-zinc-800 rounded-lg appearance-none cursor-pointer accent-cyan-500"
                        />
                    </div>

                    <div className="space-y-2">
                        <div className="flex justify-between items-center">
                            <Tooltip content={i18n.t('settings.tooltip_max_depth')} position="top">
                                <label htmlFor="max_swarm_depth" className="text-sm font-bold text-zinc-300 cursor-help w-max">{i18n.t('settings.label_max_depth')}</label>
                            </Tooltip>
                            <span className="text-xs font-mono text-zinc-400 bg-zinc-800/50 px-2 py-1 rounded">{settings.max_swarm_depth}</span>
                        </div>
                        <input
                            id="max_swarm_depth"
                            type="range"
                            name="max_swarm_depth"
                            min="1"
                            max="10"
                            value={settings.max_swarm_depth}
                            onChange={(e) => handle_numeric_change('max_swarm_depth', parseInt(e.target.value))}
                            className="w-full h-1.5 bg-zinc-800 rounded-lg appearance-none cursor-pointer accent-amber-500"
                        />
                    </div>
                </div>

                <div className="space-y-4 z-10 relative">
                    <div className="space-y-2">
                        <Tooltip content={i18n.t('settings.tooltip_max_tokens')} position="top">
                            <label htmlFor="max_task_length" className="text-sm font-bold text-zinc-300 cursor-help w-max">{i18n.t('settings.label_max_tokens')}</label>
                        </Tooltip>
                        <input
                            id="max_task_length"
                            type="number"
                            name="max_task_length"
                            value={settings.max_task_length}
                            onChange={(e) => handle_numeric_change('max_task_length', parseInt(e.target.value))}
                            className="w-full bg-[color:var(--color-background)] border border-zinc-700 rounded-lg p-2 text-sm text-zinc-100 focus:outline-none focus:border-green-500 font-mono"
                        />
                    </div>

                    <div className="space-y-2">
                        <Tooltip content={i18n.t('settings.tooltip_mission_budget')} position="top">
                            <label htmlFor="default_budget_usd" className="text-sm font-bold text-zinc-300 cursor-help w-max">{i18n.t('settings.label_mission_budget', { symbol: i18n.t('agent_config.fiscal_symbol') })}</label>
                        </Tooltip>
                        <div className="relative">
                            <span className="absolute left-3 top-1/2 -translate-y-1/2 text-zinc-500 text-xs">{i18n.t('agent_config.fiscal_symbol')}</span>
                            <input
                                id="default_budget_usd"
                                type="number"
                                name="default_budget_usd"
                                step="0.1"
                                value={settings.default_budget_usd}
                                onChange={(e) => handle_numeric_change('default_budget_usd', parseFloat(e.target.value))}
                                className="w-full bg-[color:var(--color-background)] border border-zinc-700 rounded-lg p-2 pl-6 text-sm text-zinc-100 focus:outline-none focus:border-emerald-500 font-mono"
                            />
                        </div>
                    </div>

                    <div className="p-3 bg-zinc-800/50 rounded-lg border border-zinc-700/50 mt-2">
                        <p className="text-[10px] text-zinc-500 leading-tight uppercase tracking-wider font-bold">{i18n.t('settings.alert_architecture')}</p>
                        <p className="text-[10px] text-zinc-400 mt-1 leading-relaxed">
                            {i18n.t('settings.desc_architecture_alert')}
                        </p>
                    </div>
                </div>
            </div>
        </div>
    );
};

// Metadata: [Architecture_Settings]
