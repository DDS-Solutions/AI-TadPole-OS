/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **Root View**: Global configuration and environmental tuning hub. 
 * Orchestrates system-wide settings, UI themes, and persistence layer connectivity via `settings_store`.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Settings persistence failure (local storage quota), or theme flashing on page reload.
 * - **Telemetry Link**: Search for `[Settings_View]` or `SETTINGS_UPDATE` in UI logs.
 */

import React, { useState } from 'react';
import { Save, Server, Monitor, Cpu, Shield, Store, ExternalLink } from 'lucide-react';
import { useNavigate } from 'react-router-dom';
import { use_model_store } from '../stores/model_store';
import { get_settings, save_settings } from '../stores/settings_store';
import { system_api_service } from '../services/system_api_service';
import { Tooltip } from '../components/ui';

import { i18n } from '../i18n';

/**
 * Settings Page
 * 
 * ### ⚙️ System Configuration
 * The central orchestration hub for environmental variables and engine tuning. 
 * - **Persistence**: Synchronizes local state with `localStorage` and the 
 *   backend `GovernanceHub` via `system_api_service`.
 * - **Telemetry Tuning**: Adjusts budget caps, max swarm depth, and 
 *   model temperatures.
 */
export default function Settings(): React.ReactElement {
    const navigate = useNavigate();
    const models = use_model_store(state => state.models);

    // Local state for settings form
    const [settings_state, set_settings_state] = useState(() => get_settings());

    const [is_saved, set_is_saved] = useState(false);
    const [validation_error, set_validation_error] = useState<string | null>(null);
    const [is_swarm_scanning] = useState(false); // Default state for Swarm Discovery indicator

    const handle_change = (e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>): void => {
        const { name, value, type } = e.target;
        const val = type === 'checkbox' ? (e.target as HTMLInputElement).checked : value;

        set_settings_state({
            ...settings_state,
            [name]: val
        });
        set_is_saved(false);
        set_validation_error(null);
    };

    /** Typed handler for numeric inputs (range, number) that coerces values before updating state. */
    const handle_numeric_change = (name: string, value: number): void => {
        set_settings_state({ ...settings_state, [name]: value });
        set_is_saved(false);
        set_validation_error(null);
    };

    /**
     /// Persists settings to local storage and synchronizes with the engine.
     /// 
     /// ### 🔄 Governance Sync
     /// Updates the the backend's `GovernanceHub` with updated operational
     /// limits (Max Agents, Budget Caps) to ensure full system alignment.
     */
    const handle_save = async (): Promise<void> => {
        const error = save_settings(settings_state);
        if (error) {
            set_validation_error(error);
            return;
        }

        // Synchronize governance settings with the backend
        try {
            await system_api_service.update_governance_settings({
                auto_approve_safe_skills: settings_state.auto_approve_safe_skills,
                privacy_mode: settings_state.privacy_mode,
                max_agents: settings_state.max_agents,
                max_clusters: settings_state.max_clusters,
                max_swarm_depth: settings_state.max_swarm_depth,
                max_task_length: settings_state.max_task_length,
                default_budget_usd: settings_state.default_budget_usd,
                default_model: settings_state.default_model
            });
        } catch (e) {
            console.error("Failed to sync governance settings with engine", e);
            set_validation_error(i18n.t('settings.error_sync_failed', { defaultValue: 'System synchronization failed. Local changes saved.' }));
            return;
        }

        // Apply appearance engine preferences immediately
        document.documentElement.setAttribute('data-theme', settings_state.theme);
        document.documentElement.setAttribute('data-density', settings_state.density);

        set_is_saved(true);
        set_validation_error(null);
        setTimeout(() => set_is_saved(false), 2000);
    };

    return (
        <div className="max-w-4xl mx-auto p-6 space-y-8 h-full overflow-y-auto custom-scrollbar relative">
            {/* GEO Optimization: Structured Data & Semantic Header */}
            <script type="application/ld+json">
            {JSON.stringify({
              "@context": "https://schema.org",
              "@type": "SoftwareApplication",
              "name": "Tadpole OS System Settings",
              "description": "Global configuration and environmental tuning hub for the Tadpole OS. Manage neural preferences, API vault synchronization, and governance caps.",
              "author": { "@type": "Organization", "name": "Sovereign Engineering" },
              "applicationCategory": "Configuration Tool",
              "operatingSystem": "Tadpole OS"
            })}
            </script>
            <h1 className="sr-only">Tadpole OS System Configuration & Environmental Governance</h1>
            <div className="flex justify-end pr-2">
                <Tooltip content={i18n.t('settings.tooltip_save')} position="left">
                    <button
                        onClick={handle_save}
                        className={`flex items-center gap-2 px-4 py-2 rounded-lg font-bold transition-all ${is_saved ? 'bg-emerald-500 text-white' : validation_error ? 'bg-red-600 text-white' : 'bg-zinc-100 text-zinc-900 hover:bg-white'}`}
                    >
                        <Save size={16} />
                        {is_saved ? i18n.t('settings.saved') : validation_error ? i18n.t('settings.fix_errors') : i18n.t('settings.save_changes')}
                    </button>
                </Tooltip>
            </div>

            {/* Connection Settings */}
            <div className="space-y-4">
                <h2 className="text-sm font-bold text-zinc-500 uppercase tracking-widest flex items-center gap-2">
                    {i18n.t('settings.header_connection')}
                </h2>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-8 bg-zinc-900 py-8 pl-8 pr-32 rounded-xl border border-zinc-800 shadow-sm relative overflow-hidden group">
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
                            value={settings_state.tadpole_os_url}
                            onChange={handle_change}
                            className="w-full bg-zinc-950 border border-zinc-700 rounded-lg p-3 text-sm text-zinc-100 focus:outline-none focus:border-green-500 focus:ring-1 focus:ring-green-500/20 transition-all font-mono shadow-inner"
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
                            value={settings_state.tadpole_os_api_key}
                            onChange={handle_change}
                            className="w-full bg-zinc-950 border border-zinc-700 rounded-lg p-3 text-sm text-zinc-100 focus:outline-none focus:border-green-500 focus:ring-1 focus:ring-green-500/20 transition-all font-mono shadow-inner"
                            placeholder={i18n.t('settings.placeholder_api_token')}

                        />
                        <p className="text-xs text-zinc-500 leading-relaxed">{i18n.t('settings.desc_api_token')}</p>
                    </div>
                </div>
            </div>

            {/* Appearance Settings */}
            <div className="space-y-4">
                <h2 className="text-sm font-bold text-zinc-500 uppercase tracking-widest flex items-center gap-2">
                    {i18n.t('settings.header_appearance')}
                </h2>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-8 bg-zinc-900 py-8 pl-8 pr-32 rounded-xl border border-zinc-800 shadow-sm relative overflow-hidden group">
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
                            value={settings_state.theme}
                            onChange={handle_change}
                            className="w-full bg-zinc-950 border border-zinc-700 rounded-lg p-3 text-sm text-zinc-100 focus:outline-none focus:border-emerald-500 transition-colors cursor-pointer shadow-sm"
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
                            value={settings_state.density}
                            onChange={handle_change}
                            className="w-full bg-zinc-950 border border-zinc-700 rounded-lg p-3 text-sm text-zinc-100 focus:outline-none focus:border-emerald-500 transition-colors cursor-pointer shadow-sm"
                        >
                            <option value="compact">{i18n.t('settings.density_compact')}</option>
                            <option value="comfortable">{i18n.t('settings.density_comfortable')}</option>
                        </select>
                    </div>
                </div>
            </div>

            {/* Agent Defaults */}
            <div className="space-y-4">
                <h2 className="text-sm font-bold text-zinc-500 uppercase tracking-widest flex items-center gap-2">
                    {i18n.t('settings.header_agent_defaults')}
                </h2>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-8 bg-zinc-900 py-8 pl-8 pr-32 rounded-xl border border-zinc-800 shadow-sm relative overflow-hidden group">
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
                            value={settings_state.default_model}
                            onChange={handle_change}
                            className="w-full bg-zinc-950 border border-zinc-700 rounded-lg p-3 text-sm text-zinc-100 focus:outline-none focus:border-green-500 transition-colors cursor-pointer font-mono shadow-sm"
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
                            <span className="text-xs font-mono text-zinc-400 bg-zinc-800/50 px-2 py-1 rounded">{settings_state.default_temperature}</span>
                        </div>
                        <input
                            id="default_temperature"
                            type="range"
                            name="default_temperature"
                            min="0"
                            max="2"
                            step="0.1"
                            value={settings_state.default_temperature}
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

            {/* Governance & Oversight */}
            <div className="space-y-4">
                <h2 className="text-sm font-bold text-zinc-500 uppercase tracking-widest flex items-center gap-2">
                    {i18n.t('settings.header_governance')}
                </h2>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-8 bg-zinc-900 py-8 pl-8 pr-32 rounded-xl border border-zinc-800 shadow-sm relative overflow-hidden group">
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
                                checked={settings_state.auto_approve_safe_skills}
                                onChange={handle_change}
                                className="w-5 h-5 rounded border-zinc-700 bg-zinc-950 text-green-500 focus:ring-green-500/20 cursor-pointer"
                            />
                        </div>
                        <p className="text-xs text-zinc-500 leading-relaxed">
                            {i18n.t('settings.desc_auto_approve', { skills: 'weather, reasoning' })}
                        </p>
                    </div>

                    {/* Privacy Shield Logic
                        Enables "Air-Gap Simulation" by stripping bulky system 
                        prompts and enforcing strict data-residency policies 
                        within the agent execution loop. */}
                    <div className="space-y-3 z-10 relative p-4 bg-zinc-950/50 rounded-lg border border-zinc-800 group/shield">
                        <div className="flex items-center justify-between">
                            <div className="flex items-center gap-2">
                                <div className={`p-1.5 rounded ${settings_state.privacy_mode ? 'bg-green-500/20 text-green-400' : 'bg-zinc-800 text-zinc-500'}`}>
                                    <Shield size={14} className={settings_state.privacy_mode ? 'animate-pulse' : ''} />
                                </div>
                                <Tooltip content={i18n.t('settings.tooltip_privacy_mode')} position="top">
                                    <label htmlFor="privacy_mode" className="text-sm font-bold text-zinc-300 cursor-help w-max">{i18n.t('settings.label_privacy_mode')}</label>
                                </Tooltip>
                                {settings_state.privacy_mode && (
                                    <span className="text-[10px] font-bold bg-green-500/10 text-green-500 border border-green-500/20 px-1.5 py-0.5 rounded uppercase tracking-tighter">
                                        {i18n.t('settings.badge_air_gap')}
                                    </span>
                                )}
                            </div>
                            <input
                                id="privacy_mode"
                                type="checkbox"
                                name="privacy_mode"
                                checked={settings_state.privacy_mode}
                                onChange={handle_change}
                                className="w-5 h-5 rounded border-zinc-700 bg-zinc-950 text-green-500 focus:ring-green-500/20 cursor-pointer"
                            />
                        </div>
                        <p className="text-[11px] text-zinc-500 leading-relaxed italic">
                            {settings_state.privacy_mode
                                ? i18n.t('settings.desc_privacy_mode_on')
                                : i18n.t('settings.desc_privacy_mode_off')}
                        </p>
                        {settings_state.privacy_mode && (
                            <div className="mt-2 text-[10px] flex items-center gap-1.5 text-emerald-500/80 font-bold uppercase tracking-widest">
                                <div className="w-1 h-1 rounded-full bg-emerald-500 animate-ping" />
                                {i18n.t('settings.status_active_verification')}
                            </div>
                        )}
                    </div>
                </div>
            </div>

            {/* Swarm Architecture Configuration */}
            <div className="space-y-4 pb-12">
                <h2 className="text-sm font-bold text-zinc-500 uppercase tracking-widest flex items-center gap-2">
                    {i18n.t('settings.header_architecture')}
                </h2>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-8 bg-zinc-900 py-8 pl-8 pr-32 rounded-xl border border-zinc-800 shadow-sm relative overflow-hidden group">
                    <div className="absolute top-0 right-0 p-6 opacity-5 group-hover:opacity-10 transition-opacity pointer-events-none">
                        <Cpu size={80} />
                    </div>

                    <div className="space-y-4 z-10 relative">
                        <div className="space-y-2">
                            <div className="flex justify-between items-center">
                                <Tooltip content={i18n.t('settings.tooltip_max_agents')} position="top">
                                    <label htmlFor="max_agents" className="text-sm font-bold text-zinc-300 cursor-help w-max">{i18n.t('settings.label_max_agents')}</label>
                                </Tooltip>
                                <span className="text-xs font-mono text-zinc-400 bg-zinc-800/50 px-2 py-1 rounded">{settings_state.max_agents}</span>
                            </div>
                            <input
                                id="max_agents"
                                type="range"
                                name="max_agents"
                                min="1"
                                max="200"
                                value={settings_state.max_agents}
                                onChange={(e) => handle_numeric_change('max_agents', parseInt(e.target.value))}
                                className="w-full h-1.5 bg-zinc-800 rounded-lg appearance-none cursor-pointer accent-green-500"
                            />
                        </div>

                        <div className="space-y-2">
                            <div className="flex justify-between items-center">
                                <Tooltip content={i18n.t('settings.tooltip_max_clusters')} position="top">
                                    <label htmlFor="max_clusters" className="text-sm font-bold text-zinc-300 cursor-help w-max">{i18n.t('settings.label_max_clusters')}</label>
                                </Tooltip>
                                <span className="text-xs font-mono text-zinc-400 bg-zinc-800/50 px-2 py-1 rounded">{settings_state.max_clusters}</span>
                            </div>
                            <input
                                id="max_clusters"
                                type="range"
                                name="max_clusters"
                                min="1"
                                max="50"
                                value={settings_state.max_clusters}
                                onChange={(e) => handle_numeric_change('max_clusters', parseInt(e.target.value))}
                                className="w-full h-1.5 bg-zinc-800 rounded-lg appearance-none cursor-pointer accent-cyan-500"
                            />
                        </div>

                        <div className="space-y-2">
                            <div className="flex justify-between items-center">
                                <Tooltip content={i18n.t('settings.tooltip_max_depth')} position="top">
                                    <label htmlFor="max_swarm_depth" className="text-sm font-bold text-zinc-300 cursor-help w-max">{i18n.t('settings.label_max_depth')}</label>
                                </Tooltip>
                                <span className="text-xs font-mono text-zinc-400 bg-zinc-800/50 px-2 py-1 rounded">{settings_state.max_swarm_depth}</span>
                            </div>
                            <input
                                id="max_swarm_depth"
                                type="range"
                                name="max_swarm_depth"
                                min="1"
                                max="10"
                                value={settings_state.max_swarm_depth}
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
                                value={settings_state.max_task_length}
                                onChange={(e) => handle_numeric_change('max_task_length', parseInt(e.target.value))}
                                className="w-full bg-zinc-950 border border-zinc-700 rounded-lg p-2 text-sm text-zinc-100 focus:outline-none focus:border-green-500 font-mono"
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
                                    value={settings_state.default_budget_usd}
                                    onChange={(e) => handle_numeric_change('default_budget_usd', parseFloat(e.target.value))}
                                    className="w-full bg-zinc-950 border border-zinc-700 rounded-lg p-2 pl-6 text-sm text-zinc-100 focus:outline-none focus:border-emerald-500 font-mono"
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

            {/* Sovereign Intelligence Store (SEC-05) */}
            <div className="space-y-4">
                <h2 className="text-sm font-bold text-zinc-500 uppercase tracking-widest flex items-center gap-2">
                    {i18n.t('settings.header_model_store', { defaultValue: 'Model Marketplace' })}
                </h2>
                <div className="bg-zinc-900/50 border border-zinc-800 rounded-3xl p-6 relative overflow-hidden group">
                    <div className="absolute top-0 right-0 p-8 opacity-[0.03] group-hover:opacity-[0.1] transition-opacity">
                        <Store size={120} />
                    </div>
                    <div className="flex items-start justify-between relative z-10">
                        <div className="space-y-2">
                            <div className="flex items-center gap-2">
                                <h3 className="text-xl font-black text-zinc-100 uppercase tracking-tight">{i18n.t('settings.title_intelligence_store')}</h3>
                                <span className="bg-green-500 text-[9px] font-black text-white px-2 py-0.5 rounded-full uppercase tracking-widest animate-pulse">{i18n.t('settings.badge_alpha')}</span>
                            </div>
                            <p className="text-zinc-500 text-xs max-w-md italic leading-relaxed">
                                Deploy high-fidelity LLMs and vision models directly to your local Swarm nodes with one-click orchestration. VRAM-aware and privacy-first.
                            </p>
                            <div className="flex items-center gap-4 mt-6">
                                <button
                                    onClick={() => navigate('/infra/model-store')}
                                    className="flex items-center gap-2 px-6 py-2.5 bg-green-600 hover:bg-green-500 text-white rounded-2xl text-[10px] font-bold uppercase tracking-widest transition-all shadow-lg shadow-green-500/20 active:scale-95"
                                >
                                    <Store size={14} /> {i18n.t('settings.btn_browse_models', { defaultValue: 'Browse Intelligence' })}
                                </button>
                                <div className="h-4 w-px bg-zinc-800" />
                                <div className="text-[10px] text-zinc-500 font-mono uppercase tracking-tighter">
                                    {is_swarm_scanning ? 'Scanning Cluster...' : 'Swarm Discovery Active'}
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            {/* Template Distribution */}
            <div className="space-y-4 pb-12">
                <h2 className="text-sm font-bold text-zinc-500 uppercase tracking-widest flex items-center gap-2">
                    {i18n.t('settings.header_templates')}
                </h2>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-8 bg-zinc-900 py-8 pl-8 pr-32 rounded-xl border border-zinc-800 shadow-sm relative overflow-hidden group">
                    <div className="absolute top-0 right-0 p-6 opacity-5 group-hover:opacity-10 transition-opacity pointer-events-none">
                        <Store size={80} />
                    </div>

                    <div className="space-y-4 z-10 relative md:col-span-2">
                        <div className="flex items-center gap-4">
                            <div className="p-3 bg-green-500/10 rounded-lg text-green-400">
                                <Store size={24} />
                            </div>
                            <div>
                                <h3 className="text-sm font-bold text-zinc-300">{i18n.t('settings.title_template_store')}</h3>
                                <p className="text-xs text-zinc-500 leading-relaxed max-w-xl">
                                    {i18n.t('settings.desc_template_store')}
                                </p>
                            </div>
                            <button
                                onClick={() => navigate('/store')}
                                className="ml-auto flex items-center gap-2 px-4 py-2 bg-green-600 hover:bg-green-500 text-white rounded-lg font-bold text-sm transition-colors shadow-lg"
                            >
                                <ExternalLink size={16} />
                                {i18n.t('settings.btn_open_store')}
                            </button>
                        </div>
                    </div>
                </div>
            </div>

        </div>
    );
}


// Metadata: [Settings]

// Metadata: [Settings]
