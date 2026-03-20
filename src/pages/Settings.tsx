import React, { useState } from 'react';
import { Save, Server, Monitor, Cpu, Shield, Store, ExternalLink } from 'lucide-react';
import { useNavigate } from 'react-router-dom';
import { getSettings, saveSettings } from '../stores/settingsStore';
import { Tooltip } from '../components/ui';

import { i18n } from '../i18n';

export default function Settings(): React.ReactElement {
    const navigate = useNavigate();
    // Local state for settings form
    const [settings, setSettings] = useState(() => {
        return getSettings();
    });

    const [isSaved, setIsSaved] = useState(false);
    const [validationError, setValidationError] = useState<string | null>(null);

    // State initialized synchronously via getter

    const handleChange = (e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>): void => {
        const { name, value, type } = e.target;
        const val = type === 'checkbox' ? (e.target as HTMLInputElement).checked : value;

        setSettings({
            ...settings,
            [name]: val
        });
        setIsSaved(false);
        setValidationError(null);
    };

    /** Typed handler for numeric inputs (range, number) that coerces values before updating state. */
    const handleNumericChange = (name: string, value: number): void => {
        setSettings({ ...settings, [name]: value });
        setIsSaved(false);
        setValidationError(null);
    };

    const handleSave = async (): Promise<void> => {
        const error = saveSettings(settings);
        if (error) {
            setValidationError(error);
            return;
        }

        // Synchronize governance settings with the backend
        try {
            await fetch(`${settings.TadpoleOSUrl}/oversight/settings`, {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    autoApproveSafeSkills: settings.autoApproveSafeSkills,
                    maxAgents: settings.maxAgents,
                    maxClusters: settings.maxClusters,
                    maxSwarmDepth: settings.maxSwarmDepth,
                    maxTaskLength: settings.maxTaskLength,
                    defaultBudgetUsd: settings.defaultBudgetUsd
                })
            });
        } catch (e) {
            console.error("Failed to sync governance settings with engine", e);
            // We don't block the UI save if the backend is down, 
            // but we log it for debugging.
        }

        // Apply appearance engine preferences immediately
        document.documentElement.setAttribute('data-theme', settings.theme);
        document.documentElement.setAttribute('data-density', settings.density);

        setIsSaved(true);
        setValidationError(null);
        setTimeout(() => setIsSaved(false), 2000);
    };

    return (
        <div className="max-w-4xl mx-auto p-6 space-y-8 h-full overflow-y-auto custom-scrollbar relative">
            <div className="flex justify-end pr-2">
                <Tooltip content={i18n.t('settings.tooltip_save')} position="left">
                    <button
                        onClick={handleSave}
                        className={`flex items-center gap-2 px-4 py-2 rounded-lg font-bold transition-all ${isSaved ? 'bg-emerald-500 text-white' : validationError ? 'bg-red-600 text-white' : 'bg-zinc-100 text-zinc-900 hover:bg-white'}`}
                    >
                        <Save size={16} />
                        {isSaved ? i18n.t('settings.saved') : validationError ? i18n.t('settings.fix_errors') : i18n.t('settings.save_changes')}
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
                            <label htmlFor="TadpoleOSUrl" className="text-sm font-bold text-zinc-300 block cursor-help w-max">{i18n.t('settings.label_api_url')}</label>
                        </Tooltip>
                        <input
                            id="TadpoleOSUrl"
                            type="text"
                            name="TadpoleOSUrl"
                            value={settings.TadpoleOSUrl}
                            onChange={handleChange}
                            className="w-full bg-zinc-950 border border-zinc-700 rounded-lg p-3 text-sm text-zinc-100 focus:outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500/20 transition-all font-mono shadow-inner"
                            placeholder={i18n.t('settings.placeholder_api_url')}
                        />
                        <p className="text-xs text-zinc-500 leading-relaxed">{i18n.t('settings.desc_api_url')}</p>
                        {validationError && (
                            <p className="text-xs text-red-400 font-medium mt-1">{validationError}</p>
                        )}
                    </div>
                    <div className="space-y-3 z-10 relative">
                        <Tooltip content={i18n.t('settings.tooltip_api_token')} position="top">
                            <label htmlFor="TadpoleOSApiKey" className="text-sm font-bold text-zinc-300 block cursor-help w-max">{i18n.t('settings.label_api_token')}</label>
                        </Tooltip>
                        <input
                            id="TadpoleOSApiKey"
                            type="password"
                            name="TadpoleOSApiKey"
                            value={settings.TadpoleOSApiKey}
                            onChange={handleChange}
                            className="w-full bg-zinc-950 border border-zinc-700 rounded-lg p-3 text-sm text-zinc-100 focus:outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500/20 transition-all font-mono shadow-inner"
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
                            value={settings.theme}
                            onChange={handleChange}
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
                            value={settings.density}
                            onChange={handleChange}
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
                            <label htmlFor="defaultModel" className="text-sm font-bold text-zinc-300 block cursor-help w-max">{i18n.t('settings.label_default_model')}</label>
                        </Tooltip>
                        <select
                            id="defaultModel"
                            name="defaultModel"
                            value={settings.defaultModel}
                            onChange={handleChange}
                            className="w-full bg-zinc-950 border border-zinc-700 rounded-lg p-3 text-sm text-zinc-100 focus:outline-none focus:border-blue-500 transition-colors cursor-pointer font-mono shadow-sm"
                        >
                            <option value="GPT-4o">{i18n.t('settings.model_gpt4o')}</option>
                            <option value="Claude 3.5 Sonnet">{i18n.t('settings.model_claude')}</option>
                            <option value="DeepSeek V3">{i18n.t('settings.model_deepseek')}</option>
                            <option value="o1-preview">{i18n.t('settings.model_o1')}</option>
                        </select>
                        <p className="text-xs text-zinc-500 leading-relaxed">{i18n.t('settings.desc_default_model')}</p>
                    </div>

                    <div className="space-y-3 z-10 relative">
                        <div className="flex justify-between items-center">
                            <Tooltip content={i18n.t('settings.tooltip_temperature')} position="top">
                                <label htmlFor="defaultTemperature" className="text-sm font-bold text-zinc-300 block cursor-help w-max">{i18n.t('settings.label_temperature')}</label>
                            </Tooltip>
                            <span className="text-xs font-mono text-zinc-400 bg-zinc-800/50 px-2 py-1 rounded">{settings.defaultTemperature}</span>
                        </div>
                        <input
                            id="defaultTemperature"
                            type="range"
                            name="defaultTemperature"
                            min="0"
                            max="2"
                            step="0.1"
                            value={settings.defaultTemperature}
                            onChange={handleChange}
                            className="w-full h-2 bg-zinc-800 rounded-lg appearance-none cursor-pointer accent-blue-500 mt-2 hover:bg-zinc-700 transition-colors"
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
                                <label htmlFor="autoApproveSafeSkills" className="text-sm font-bold text-zinc-300 cursor-help w-max">{i18n.t('settings.label_auto_approve')}</label>
                            </Tooltip>
                            <input
                                id="autoApproveSafeSkills"
                                type="checkbox"
                                name="autoApproveSafeSkills"
                                checked={settings.autoApproveSafeSkills}
                                onChange={handleChange}
                                className="w-5 h-5 rounded border-zinc-700 bg-zinc-950 text-blue-500 focus:ring-blue-500/20 cursor-pointer"
                            />
                        </div>
                        <p className="text-xs text-zinc-500 leading-relaxed">
                            {i18n.t('settings.desc_auto_approve', { skills: 'weather, reasoning' })}
                        </p>
                    </div>

                    <div className="space-y-3 z-10 flex flex-col justify-center relative">
                        <div className="p-4 bg-blue-500/5 border border-blue-500/20 rounded-lg">
                            <p className="text-xs text-blue-400/90 italic leading-relaxed">
                                {i18n.t('settings.governance_quote')}
                            </p>
                        </div>
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
                                    <label htmlFor="maxAgents" className="text-sm font-bold text-zinc-300 cursor-help w-max">{i18n.t('settings.label_max_agents')}</label>
                                </Tooltip>
                                <span className="text-xs font-mono text-zinc-400 bg-zinc-800/50 px-2 py-1 rounded">{settings.maxAgents}</span>
                            </div>
                            <input
                                id="maxAgents"
                                type="range"
                                name="maxAgents"
                                min="1"
                                max="200"
                                value={settings.maxAgents}
                                onChange={(e) => handleNumericChange('maxAgents', parseInt(e.target.value))}
                                className="w-full h-1.5 bg-zinc-800 rounded-lg appearance-none cursor-pointer accent-blue-500"
                            />
                        </div>

                        <div className="space-y-2">
                            <div className="flex justify-between items-center">
                                <Tooltip content={i18n.t('settings.tooltip_max_clusters')} position="top">
                                    <label htmlFor="maxClusters" className="text-sm font-bold text-zinc-300 cursor-help w-max">{i18n.t('settings.label_max_clusters')}</label>
                                </Tooltip>
                                <span className="text-xs font-mono text-zinc-400 bg-zinc-800/50 px-2 py-1 rounded">{settings.maxClusters}</span>
                            </div>
                            <input
                                id="maxClusters"
                                type="range"
                                name="maxClusters"
                                min="1"
                                max="50"
                                value={settings.maxClusters}
                                onChange={(e) => handleNumericChange('maxClusters', parseInt(e.target.value))}
                                className="w-full h-1.5 bg-zinc-800 rounded-lg appearance-none cursor-pointer accent-cyan-500"
                            />
                        </div>

                        <div className="space-y-2">
                            <div className="flex justify-between items-center">
                                <Tooltip content={i18n.t('settings.tooltip_max_depth')} position="top">
                                    <label htmlFor="maxSwarmDepth" className="text-sm font-bold text-zinc-300 cursor-help w-max">{i18n.t('settings.label_max_depth')}</label>
                                </Tooltip>
                                <span className="text-xs font-mono text-zinc-400 bg-zinc-800/50 px-2 py-1 rounded">{settings.maxSwarmDepth}</span>
                            </div>
                            <input
                                id="maxSwarmDepth"
                                type="range"
                                name="maxSwarmDepth"
                                min="1"
                                max="10"
                                value={settings.maxSwarmDepth}
                                onChange={(e) => handleNumericChange('maxSwarmDepth', parseInt(e.target.value))}
                                className="w-full h-1.5 bg-zinc-800 rounded-lg appearance-none cursor-pointer accent-amber-500"
                            />
                        </div>
                    </div>

                    <div className="space-y-4 z-10 relative">
                        <div className="space-y-2">
                            <Tooltip content={i18n.t('settings.tooltip_max_tokens')} position="top">
                                <label htmlFor="maxTaskLength" className="text-sm font-bold text-zinc-300 cursor-help w-max">{i18n.t('settings.label_max_tokens')}</label>
                            </Tooltip>
                            <input
                                id="maxTaskLength"
                                type="number"
                                name="maxTaskLength"
                                value={settings.maxTaskLength}
                                onChange={(e) => handleNumericChange('maxTaskLength', parseInt(e.target.value))}
                                className="w-full bg-zinc-950 border border-zinc-700 rounded-lg p-2 text-sm text-zinc-100 focus:outline-none focus:border-blue-500 font-mono"
                            />
                        </div>

                        <div className="space-y-2">
                            <Tooltip content={i18n.t('settings.tooltip_mission_budget')} position="top">
                                <label htmlFor="defaultBudgetUsd" className="text-sm font-bold text-zinc-300 cursor-help w-max">{i18n.t('settings.label_mission_budget', { symbol: i18n.t('agent_config.fiscal_symbol') })}</label>
                            </Tooltip>
                            <div className="relative">
                                <span className="absolute left-3 top-1/2 -translate-y-1/2 text-zinc-500 text-xs">{i18n.t('agent_config.fiscal_symbol')}</span>
                                <input
                                    id="defaultBudgetUsd"
                                    type="number"
                                    name="defaultBudgetUsd"
                                    step="0.1"
                                    value={settings.defaultBudgetUsd}
                                    onChange={(e) => handleNumericChange('defaultBudgetUsd', parseFloat(e.target.value))}
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
                            <div className="p-3 bg-blue-500/10 rounded-lg text-blue-400">
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
                                className="ml-auto flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded-lg font-bold text-sm transition-colors shadow-lg"
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

