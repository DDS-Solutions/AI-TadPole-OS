/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Appearance_Settings]` in observability traces.
 */

import React from 'react';
import { Monitor } from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';
import type { Tadpole_Settings } from '../../stores/settings_store';

interface AppearanceSettingsProps {
    settings: Tadpole_Settings;
    handle_change: (e: React.ChangeEvent<HTMLSelectElement>) => void;
}

export const Appearance_Settings: React.FC<AppearanceSettingsProps> = ({ settings, handle_change }) => {
    return (
        <div className="space-y-4">
            <h2 className="text-sm font-bold text-zinc-500 uppercase tracking-widest flex items-center gap-2">
                {i18n.t('settings.header_appearance')}
            </h2>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-8 bg-[color:var(--color-surface)] py-8 pl-8 pr-32 rounded-xl border border-[color:var(--color-border)] shadow-sm relative overflow-hidden group">
                <div className="absolute top-0 right-0 p-6 opacity-5 group-hover:opacity-10 transition-opacity pointer-events-none">
                    <Monitor size={80} />
                </div>

                <div className="space-y-3 z-10 relative">
                    <Tooltip content={i18n.t('settings.tooltip_theme')} position="top">
                        <label htmlFor="theme" className="text-sm font-bold text-zinc-300 block cursor-help w-max">{i18n.t('settings.label_theme')}</label>
                    </Tooltip>
                    <select
                        id="theme"
                        name="theme"
                        value={settings.theme}
                        onChange={handle_change}
                        className="w-full bg-[color:var(--color-background)] border border-zinc-700 rounded-lg p-3 text-sm text-zinc-100 focus:outline-none focus:border-emerald-500 transition-colors cursor-pointer shadow-sm"
                    >
                        <option value="zinc">{i18n.t('settings.theme_zinc')}</option>
                        <option value="slate">{i18n.t('settings.theme_slate')}</option>
                        <option value="neutral">{i18n.t('settings.theme_neutral')}</option>
                    </select>
                </div>
                <div className="space-y-3 z-10 relative">
                    <Tooltip content={i18n.t('settings.tooltip_density')} position="top">
                        <label htmlFor="density" className="text-sm font-bold text-zinc-300 block cursor-help w-max">{i18n.t('settings.label_density')}</label>
                    </Tooltip>
                    <select
                        id="density"
                        name="density"
                        value={settings.density}
                        onChange={handle_change}
                        className="w-full bg-[color:var(--color-background)] border border-zinc-700 rounded-lg p-3 text-sm text-zinc-100 focus:outline-none focus:border-emerald-500 transition-colors cursor-pointer shadow-sm"
                    >
                        <option value="compact">{i18n.t('settings.density_compact')}</option>
                        <option value="comfortable">{i18n.t('settings.density_comfortable')}</option>
                    </select>
                </div>
            </div>
        </div>
    );
};

// Metadata: [Appearance_Settings]
