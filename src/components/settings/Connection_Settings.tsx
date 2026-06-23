/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Connection_Settings]` in observability traces.
 */

import React from 'react';
import { Server } from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';
import type { Tadpole_Settings } from '../../stores/settings_store';

interface ConnectionSettingsProps {
    settings: Tadpole_Settings;
    handle_change: (e: React.ChangeEvent<HTMLInputElement>) => void;
    validation_error: string | null;
}

export const Connection_Settings: React.FC<ConnectionSettingsProps> = ({ settings, handle_change, validation_error }) => {
    return (
        <div className="space-y-4">
            <h2 className="text-sm font-bold text-zinc-500 uppercase tracking-widest flex items-center gap-2">
                {i18n.t('settings.header_connection')}
            </h2>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-8 bg-[color:var(--color-surface)] py-8 pl-8 pr-32 rounded-xl border border-[color:var(--color-border)] shadow-sm relative overflow-hidden group">
                <div className="absolute top-0 right-0 p-6 opacity-5 group-hover:opacity-10 transition-opacity pointer-events-none">
                    <Server size={80} />
                </div>

                <div className="space-y-3 z-10 relative">
                    <Tooltip content={i18n.t('settings.tooltip_api_url')} position="top">
                        <label htmlFor="tadpole_os_url" className="text-sm font-bold text-zinc-300 block cursor-help w-max">{i18n.t('settings.label_api_url')}</label>
                    </Tooltip>
                    <input
                        id="tadpole_os_url"
                        type="text"
                        name="tadpole_os_url"
                        value={settings.tadpole_os_url}
                        onChange={handle_change}
                        className="w-full bg-[color:var(--color-background)] border border-zinc-700 rounded-lg p-3 text-sm text-zinc-100 focus:outline-none focus:border-green-500 focus:ring-1 focus:ring-green-500/20 transition-all font-mono shadow-inner"
                        placeholder={i18n.t('settings.placeholder_api_url')}
                    />
                    <p className="text-xs text-zinc-500 leading-relaxed">{i18n.t('settings.desc_api_url')}</p>
                    {validation_error && (
                        <p className="text-xs text-red-400 font-medium mt-1">{validation_error}</p>
                    )}
                </div>
                <div className="space-y-3 z-10 relative">
                    <Tooltip content={i18n.t('settings.tooltip_api_token')} position="top">
                        <label htmlFor="tadpole_os_api_key" className="text-sm font-bold text-zinc-300 block cursor-help w-max">{i18n.t('settings.label_api_token')}</label>
                    </Tooltip>
                    <input
                        id="tadpole_os_api_key"
                        type="password"
                        name="tadpole_os_api_key"
                        value={settings.tadpole_os_api_key}
                        onChange={handle_change}
                        className="w-full bg-[color:var(--color-background)] border border-zinc-700 rounded-lg p-3 text-sm text-zinc-100 focus:outline-none focus:border-green-500 focus:ring-1 focus:ring-green-500/20 transition-all font-mono shadow-inner"
                        placeholder={i18n.t('settings.placeholder_api_token')}
                    />
                    <p className="text-xs text-zinc-500 leading-relaxed">{i18n.t('settings.desc_api_token')}</p>
                </div>
            </div>
        </div>
    );
};

// Metadata: [Connection_Settings]
