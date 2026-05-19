/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Agent_Defaults_Settings]` in observability traces.
 */

import React from 'react';
import { Cpu } from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';
import type { Tadpole_Settings } from '../../stores/settings_store';

interface AgentDefaultsSettingsProps {
    settings: Tadpole_Settings;
    models: { id: string; name: string }[];
    handle_change: (e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>) => void;
}

export const Agent_Defaults_Settings: React.FC<AgentDefaultsSettingsProps> = ({ settings, models, handle_change }) => {
    return (
        <div className="space-y-4">
            <h2 className="text-sm font-bold text-zinc-500 uppercase tracking-widest flex items-center gap-2">
                {i18n.t('settings.header_agent_defaults')}
            </h2>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-8 bg-[color:var(--color-surface)] py-8 pl-8 pr-32 rounded-xl border border-[color:var(--color-border)] shadow-sm relative overflow-hidden group">
                <div className="absolute top-0 right-0 p-6 opacity-5 group-hover:opacity-10 transition-opacity pointer-events-none">
                    <Cpu size={80} />
                </div>

                <div className="space-y-3 z-10 relative">
                    <Tooltip content={i18n.t('settings.tooltip_default_model')} position="top">
                        <label htmlFor="default_model" className="text-sm font-bold text-zinc-300 block cursor-help w-max">{i18n.t('settings.label_default_model')}</label>
                    </Tooltip>
                    <select
                        id="default_model"
                        name="default_model"
                        value={settings.default_model}
                        onChange={handle_change}
                        className="w-full bg-[color:var(--color-background)] border border-zinc-700 rounded-lg p-3 text-sm text-zinc-100 focus:outline-none focus:border-green-500 transition-colors cursor-pointer font-mono shadow-sm"
                    >
                        {models.map(model => (
                            <option key={model.id} value={model.name}>{model.name}</option>
                        ))}
                    </select>
                    <p className="text-xs text-zinc-500 leading-relaxed">{i18n.t('settings.desc_default_model')}</p>
                </div>

                <div className="space-y-3 z-10 relative">
                    <div className="flex justify-between items-center">
                        <Tooltip content={i18n.t('settings.tooltip_temperature')} position="top">
                            <label htmlFor="default_temperature" className="text-sm font-bold text-zinc-300 block cursor-help w-max">{i18n.t('settings.label_temperature')}</label>
                        </Tooltip>
                        <span className="text-xs font-mono text-zinc-400 bg-zinc-800/50 px-2 py-1 rounded">{settings.default_temperature}</span>
                    </div>
                    <input
                        id="default_temperature"
                        type="range"
                        name="default_temperature"
                        min="0"
                        max="2"
                        step="0.1"
                        value={settings.default_temperature}
                        onChange={handle_change}
                        className="w-full h-2 bg-zinc-800 rounded-lg appearance-none cursor-pointer accent-green-500 mt-2 hover:bg-zinc-700 transition-colors"
                    />
                    <div className="flex justify-between text-[10px] text-zinc-500 uppercase font-bold tracking-widest mt-1">
                        <span>{i18n.t('settings.temp_precise')}</span>
                        <span>{i18n.t('settings.temp_balanced')}</span>
                        <span>{i18n.t('settings.temp_creative')}</span>
                    </div>
                </div>
            </div>
        </div>
    );
};

// Metadata: [Agent_Defaults_Settings]
