/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Governance_Settings]` in observability traces.
 */

import React from 'react';
import { Shield } from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';
import type { Tadpole_Settings } from '../../stores/settings_store';

interface GovernanceSettingsProps {
    settings: Tadpole_Settings;
    handle_change: (e: React.ChangeEvent<HTMLInputElement>) => void;
}

export const Governance_Settings: React.FC<GovernanceSettingsProps> = ({ settings, handle_change }) => {
    return (
        <div className="space-y-4">
            <h2 className="text-sm font-bold text-zinc-500 uppercase tracking-widest flex items-center gap-2">
                {i18n.t('settings.header_governance')}
            </h2>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-8 bg-[color:var(--color-surface)] py-8 pl-8 pr-32 rounded-xl border border-[color:var(--color-border)] shadow-sm relative overflow-hidden group">
                <div className="absolute top-0 right-0 p-6 opacity-5 group-hover:opacity-10 transition-opacity pointer-events-none">
                    <Shield size={80} />
                </div>

                <div className="space-y-3 z-10 relative">
                    <div className="flex items-center justify-between">
                        <Tooltip content={i18n.t('settings.tooltip_auto_approve')} position="top">
                            <label htmlFor="auto_approve_safe_skills" className="text-sm font-bold text-zinc-300 cursor-help w-max">{i18n.t('settings.label_auto_approve')}</label>
                        </Tooltip>
                        <input
                            id="auto_approve_safe_skills"
                            type="checkbox"
                            name="auto_approve_safe_skills"
                            checked={settings.auto_approve_safe_skills}
                            onChange={handle_change}
                            className="w-5 h-5 rounded border-zinc-700 bg-[color:var(--color-background)] text-green-500 focus:ring-green-500/20 cursor-pointer"
                        />
                    </div>
                    <p className="text-xs text-zinc-500 leading-relaxed">
                        {i18n.t('settings.desc_auto_approve', { skills: 'weather, reasoning' })}
                    </p>
                </div>

                <div className="space-y-3 z-10 relative p-4 bg-[color:color-mix(in_srgb,var(--color-background)_50%,transparent)] rounded-lg border border-[color:var(--color-border)] group/shield">
                    <div className="flex items-center justify-between">
                        <div className="flex items-center gap-2">
                            <div className={`p-1.5 rounded ${settings.privacy_mode ? 'bg-green-500/20 text-green-400' : 'bg-zinc-800 text-zinc-500'}`}>
                                <Shield size={14} className={settings.privacy_mode ? 'animate-pulse' : ''} />
                            </div>
                            <Tooltip content={i18n.t('settings.tooltip_privacy_mode')} position="top">
                                <label htmlFor="privacy_mode" className="text-sm font-bold text-zinc-300 cursor-help w-max">{i18n.t('settings.label_privacy_mode')}</label>
                            </Tooltip>
                            {settings.privacy_mode && (
                                <span className="text-[10px] font-bold bg-green-500/10 text-green-500 border border-green-500/20 px-1.5 py-0.5 rounded uppercase tracking-tighter">
                                    {i18n.t('settings.badge_air_gap')}
                                </span>
                            )}
                        </div>
                        <input
                            id="privacy_mode"
                            type="checkbox"
                            name="privacy_mode"
                            checked={settings.privacy_mode}
                            onChange={handle_change}
                            className="w-5 h-5 rounded border-zinc-700 bg-[color:var(--color-background)] text-green-500 focus:ring-green-500/20 cursor-pointer"
                        />
                    </div>
                    <p className="text-[11px] text-zinc-500 leading-relaxed italic">
                        {settings.privacy_mode
                            ? i18n.t('settings.desc_privacy_mode_on')
                            : i18n.t('settings.desc_privacy_mode_off')}
                    </p>
                    {settings.privacy_mode && (
                        <div className="mt-2 text-[10px] flex items-center gap-1.5 text-emerald-500/80 font-bold uppercase tracking-widest">
                            <div className="w-1 h-1 rounded-full bg-emerald-500 animate-ping" />
                            {i18n.t('settings.status_active_verification')}
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
};

// Metadata: [Governance_Settings]
