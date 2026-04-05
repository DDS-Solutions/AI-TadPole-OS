/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: High-fidelity administrative bridge for model inventory management. 
 * Orchestrates API handshake tests, model forging (Limits/Modality), and persistence syncing with the Rust backend vault.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: API key vault mismatch (500), model forge schema collision, or test trace timeout (unreachable endpoint).
 * - **Telemetry Link**: Search for `[Provider_Config_Panel]` or `test_provider` in UI tracing.
 */

import React, { useState, useReducer, useMemo } from 'react';
import { X, Save, Plus, Check, ShieldCheck, Globe, Key, Zap, Info, Cpu, Activity, AlertTriangle, Play, Square, Server } from 'lucide-react';
import { Tooltip } from './ui';
import { use_provider_store } from '../stores/provider_store';
import { use_vault_store } from '../stores/vault_store';
import { use_model_store } from '../stores/model_store';
import type { Provider_Config } from '../stores/provider_store';
import type { Model_Entry } from '../stores/model_store';
import { event_bus } from '../services/event_bus';
import { MODEL_OPTIONS } from '../data/models';
import { resolve_provider } from '../utils/model_utils';
import { tadpole_os_service } from '../services/tadpoleos_service';
import { i18n } from '../i18n';
import { panel_reducer } from '../hooks/use_provider_form';
import { Forge_Item } from './provider/Forge_Item';

/**
 * Provider_Config_Panel_Props
 * Defines the interface for the Provider_Config_Panel component.
 * Refactored for strict snake_case compliance for backend parity.
 */
interface Provider_Config_Panel_Props {
    /** The provider configuration to edit */
    provider: Provider_Config;
    /** Callback to close the panel */
    on_close: () => void;
}

/**
 * Provider_Config_Panel
 * A high-fidelity administrative panel for configuring AI Providers and their associated models.
 * Features the "Model Forge" for real-time model catalog management.
 */
export default function Provider_Config_Panel({ provider, on_close }: Provider_Config_Panel_Props): React.ReactElement {
    const { edit_provider, set_provider_config } = use_provider_store();
    const { encrypted_configs } = use_vault_store();
    const { models, add_model, edit_model, delete_model } = use_model_store();
    const has_saved_key = !!encrypted_configs[provider.id];

    // Models for this provider directly from store
    const provider_models = useMemo(() =>
        models.filter(m => m.provider === provider.id)
            .sort((a, b) => a.name.localeCompare(b.name)),
        [models, provider.id]
    );

    const [state, dispatch] = useReducer(panel_reducer, {
        name: provider.name,
        icon: provider.icon || '⚡',
        api_key: '', // Start empty for security
        base_url: provider.base_url || '',
        external_id: provider.external_id || '',
        protocol: provider.protocol || 'openai',
        custom_headers: JSON.stringify(provider.custom_headers || {}, null, 2),
        audio_model: provider.audio_model || '',
        persist_to_engine: provider.persist_to_engine || false,
        is_testing: false,
        test_result: 'idle',
        test_message: ''
    });

    const [is_forge_adding, set_is_forge_adding] = useState(false);
    const [forge_new_model, set_forge_new_model] = useState({
        name: '',
        modality: 'llm' as Model_Entry['modality'],
        rpm: 10,
        tpm: 100000,
        rpd: 1000,
        tpd: 10000000
    });
    const [is_forge_custom_modality, set_is_forge_custom_modality] = useState(false);
    const [forge_custom_modality, set_forge_custom_modality] = useState('');
    const [error_message, set_error_message] = useState<string>('');
    const [editing_model_id, set_editing_model_id] = useState<string | null>(null);
    const [local_server_path, set_local_server_path] = useState<string>((provider.metadata?.local_server_path as string) || '');

    const handle_start_server = (): void => {
        event_bus.emit_log({
            source: 'System',
            text: `Manual boot sequence initiated for: ${local_server_path || 'local model driver'}`,
            severity: 'info'
        });
    };

    const handle_stop_server = (): void => {
         event_bus.emit_log({
            source: 'System',
            text: 'System termination signal sent to local driver.',
            severity: 'warning'
        });
    };

    const handle_save = async (): Promise<void> => {
        dispatch({ type: 'UPDATE_FIELD', field: 'is_testing', value: true });

        try {
            // 1. Update Provider Identity (Name/Icon)
            edit_provider(provider.id, state.name, state.icon);

            // 2. Update Provider Config (Vault)
            let parsed_headers = {};
            try {
                parsed_headers = JSON.parse(state.custom_headers);
            } catch {
                console.error('Invalid JSON headers');
            }

            await set_provider_config(
                provider.id,
                state.api_key,
                state.base_url,
                state.external_id,
                state.protocol,
                parsed_headers,
                state.audio_model,
                state.persist_to_engine
            );

            event_bus.emit_log({
                source: 'System',
                text: i18n.t('provider.save_success', { name: state.name }),
                severity: 'success'
            });

            on_close();
        } catch (error) {
            const message = error instanceof Error ? error.message : 'Unknown vault error';
            event_bus.emit_log({
                source: 'System',
                text: `${i18n.t('provider.vault_error')}: ${message}`,
                severity: 'error'
            });
        } finally {
            dispatch({ type: 'UPDATE_FIELD', field: 'is_testing', value: false });
        }
    };

    const handle_test_connection = async (): Promise<void> => {
        const missing = [];
        if (!state.api_key.trim() && !has_saved_key) missing.push(i18n.t('provider.field_api_key'));
        if (!state.protocol) missing.push(i18n.t('provider.field_protocol'));

        if (missing.length > 0) {
            event_bus.emit_log({
                source: 'System',
                text: i18n.t('provider.handshake_blocked', { missing: missing.join(", ") }),
                severity: 'error'
            });
            return;
        }

        dispatch({ type: 'UPDATE_FIELD', field: 'is_testing', value: true });
        dispatch({ type: 'UPDATE_FIELD', field: 'test_result', value: 'idle' });
        dispatch({ type: 'UPDATE_FIELD', field: 'test_message', value: '' });

        try {
            let parsed_headers = {};
            try {
                parsed_headers = JSON.parse(state.custom_headers);
            } catch {
                console.error('Handshake: Invalid JSON headers, using defaults.');
            }

            const result = await tadpole_os_service.test_provider({
                id: provider.id,
                name: state.name,
                icon: state.icon,
                api_key: state.api_key,
                base_url: state.base_url,
                external_id: state.external_id,
                protocol: state.protocol || 'openai',
                custom_headers: parsed_headers,
                audio_model: state.audio_model,
                persist_to_engine: state.persist_to_engine
            });

            const is_success = result.status === 'success';
            dispatch({ type: 'UPDATE_FIELD', field: 'is_testing', value: false });
            dispatch({ type: 'UPDATE_FIELD', field: 'test_result', value: is_success ? 'success' : 'failed' });
            dispatch({ type: 'UPDATE_FIELD', field: 'test_message', value: result.message || '' });

            event_bus.emit_log({
                source: 'System',
                text: result.message || (is_success ? i18n.t('provider.handshake_success') : i18n.t('provider.handshake_failed')),
                severity: is_success ? 'success' : 'error'
            });
            
            if (!is_success) {
                set_error_message(result.message || i18n.t('provider.handshake_failed'));
            } else {
                set_error_message('');
            }
        } catch (error: unknown) {
            dispatch({ type: 'UPDATE_FIELD', field: 'is_testing', value: false });
            dispatch({ type: 'UPDATE_FIELD', field: 'test_result', value: 'failed' });
            const message = error instanceof Error ? error.message : i18n.t('provider.handshake_failed');
            event_bus.emit_log({
                source: 'System',
                text: `${i18n.t('provider.trace_error')}: ${message}`,
                severity: 'error'
            });
        }
    };

    const handle_forge_add = (): void => {
        if (!forge_new_model.name.trim()) return;
        const final_modality = is_forge_custom_modality ? forge_custom_modality : forge_new_model.modality;
        add_model(forge_new_model.name, provider.id, final_modality, {
            rpm: forge_new_model.rpm,
            tpm: forge_new_model.tpm,
            rpd: forge_new_model.rpd,
            tpd: forge_new_model.tpd
        });
        set_forge_new_model({ name: '', modality: 'llm', rpm: 10, tpm: 100000, rpd: 1000, tpd: 10000000 });
        set_is_forge_custom_modality(false);
        set_forge_custom_modality('');
        set_is_forge_adding(false);
        event_bus.emit_log({ source: 'System', text: i18n.t('provider.forge_success', { name: forge_new_model.name }), severity: 'success' });
    };

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
            <div 
                className="absolute inset-0 bg-black/60 backdrop-blur-sm animate-in fade-in duration-300" 
                onClick={on_close}
                onKeyDown={(e) => { if (e.key === 'Escape') on_close(); }}
                role="button"
                tabIndex={-1}
                aria-label={i18n.t('common.close_modal', { defaultValue: 'Close Modal' })}
            />

            <div className="relative w-full max-w-2xl bg-zinc-950 border border-zinc-800 rounded-3xl shadow-2xl flex flex-col overflow-hidden animate-in zoom-in-95 duration-300 max-h-[85vh]">
                {/* Header */}
                <div className="p-6 border-b border-zinc-800 bg-zinc-900/80 backdrop-blur-md flex items-start justify-between shrink-0 relative overflow-hidden">
                    <div className="absolute top-0 left-0 w-full h-px bg-gradient-to-r from-transparent via-emerald-500/20 to-transparent" />

                    <div className="flex items-start gap-4 z-10">
                        <div className="p-3 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-xl flex items-center justify-center italic">
                            {state.icon}
                        </div>
                        <div className="space-y-1">
                            <h2 className="text-xs font-bold text-emerald-500 tracking-[0.2em] uppercase opacity-80">
                                {i18n.t('provider.panel_title')}
                            </h2>
                            <div className="flex items-center gap-2">
                                <input
                                    value={state.name}
                                    onChange={(e) => dispatch({ type: 'UPDATE_FIELD', field: 'name', value: e.target.value })}
                                    className="bg-transparent border-none p-0 font-bold text-zinc-100 text-2xl leading-tight focus:ring-0 w-full hover:bg-white/5 rounded px-1 -ml-1 transition-colors"
                                    aria-label={i18n.t('provider.field_name')}
                                />
                                <span className="text-[11px] text-zinc-500 font-mono tracking-tighter bg-zinc-900 border border-white/5 px-2 py-0.5 rounded">
                                    {provider.id.toUpperCase()}
                                </span>
                            </div>
                        </div>
                    </div>
                    <button onClick={on_close} className="p-2 rounded-lg hover:bg-zinc-800 transition-colors text-zinc-500 hover:text-white z-10">
                        <X size={20} />
                    </button>
                </div>

                {/* Content */}
                <div className="flex-1 overflow-y-auto p-6 space-y-8 custom-scrollbar relative">
                    <div className="neural-grid opacity-[0.03]" />

                    {/* Authorization Layer */}
                    <section className="space-y-4">
                        <h3 className="text-[11px] font-bold text-zinc-500 uppercase tracking-[0.2em] flex items-center gap-2">
                            <ShieldCheck size={12} className="text-emerald-500/50" />
                            {i18n.t('provider.auth_layer')}
                        </h3>

                        <div className="grid grid-cols-1 gap-4">
                            <div className="space-y-1.5">
                                <label htmlFor="provider-api-key" className="text-[11px] font-bold text-zinc-600 uppercase tracking-widest px-1 flex items-center gap-2">
                                    {i18n.t('provider.field_api_key')}
                                    <Tooltip content={i18n.t('provider.api_key_tooltip')} position="top">
                                        <Info size={11} className="text-zinc-700 hover:text-emerald-500 cursor-help transition-colors" />
                                    </Tooltip>
                                </label>
                                <div className="relative">
                                    <Key className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-zinc-700" />
                                    <input
                                        id="provider-api-key"
                                        type="password"
                                        placeholder={i18n.t('provider.placeholder_api_key')}
                                        value={state.api_key}
                                        onChange={(e) => dispatch({ type: 'UPDATE_FIELD', field: 'api_key', value: e.target.value })}
                                        className="w-full bg-zinc-950 border border-zinc-800 rounded-xl pl-10 pr-4 py-2.5 text-xs text-zinc-200 focus:outline-none focus:border-emerald-500/40 transition-all font-mono"
                                        aria-label={i18n.t('provider.field_api_key_label')}
                                    />
                                </div>
                            </div>

                            <div className="space-y-1.5">
                                <label htmlFor="provider-endpoint" className="text-[11px] font-bold text-zinc-600 uppercase tracking-widest px-1 flex items-center gap-2">
                                    {i18n.t('provider.field_endpoint')}
                                    <Tooltip content={i18n.t('provider.endpoint_tooltip')} position="top">
                                        <Info size={11} className="text-zinc-700 hover:text-emerald-500 cursor-help transition-colors" />
                                    </Tooltip>
                                </label>
                                <div className="relative">
                                    <Globe className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-zinc-700" />
                                    <input
                                        id="provider-endpoint"
                                        type="text"
                                        placeholder={i18n.t('provider.placeholder_endpoint')}
                                        value={state.base_url}
                                        onChange={(e) => dispatch({ type: 'UPDATE_FIELD', field: 'base_url', value: e.target.value })}
                                        className="w-full bg-zinc-950 border border-zinc-800 rounded-xl pl-10 pr-4 py-2.5 text-xs text-zinc-200 focus:outline-none focus:border-emerald-500/40 transition-all font-mono"
                                        aria-label={i18n.t('provider.field_endpoint_label')}
                                    />
                                </div>
                            </div>

                            {/* Local Server Orchestration */}
                            {state.base_url.toLowerCase().includes('localhost') && (
                                <div className="space-y-3 p-4 bg-blue-500/5 border border-blue-500/10 rounded-2xl animate-in slide-in-from-top-2 border-dashed">
                                    <div className="flex items-center justify-between">
                                        <label htmlFor="provider-local-path" className="text-[10px] font-bold text-blue-400 uppercase tracking-[0.2em] flex items-center gap-2">
                                            <Server size={12} />
                                            {i18n.t('provider.local_server_orchestration', { defaultValue: 'Local Server Orchestration' })}
                                        </label>
                                        <div className="flex gap-2">
                                            <Tooltip content="Launch Local Engine" position="top">
                                                <button 
                                                    onClick={handle_start_server}
                                                    className="p-1.5 bg-blue-600 hover:bg-blue-500 text-white rounded-lg transition-all shadow-lg shadow-blue-500/20 active:scale-95"
                                                    aria-label={i18n.t('provider.aria_start_server')}
                                                >
                                                    <Play size={12} fill="currentColor" />
                                                </button>
                                            </Tooltip>
                                            <Tooltip content="Stop Engine" position="top">
                                                <button 
                                                    onClick={handle_stop_server}
                                                    className="p-1.5 bg-zinc-800 hover:bg-zinc-700 text-zinc-400 rounded-lg transition-all"
                                                    aria-label={i18n.t('provider.aria_stop_server')}
                                                >
                                                    <Square size={12} fill="currentColor" />
                                                </button>
                                            </Tooltip>
                                        </div>
                                    </div>
                                    <div className="relative">
                                        <input
                                            id="provider-local-path"
                                            type="text"
                                            placeholder={i18n.t('provider.placeholder_local_path', { defaultValue: 'Paste local server link or path...' })}
                                            value={local_server_path}
                                            onChange={(e) => set_local_server_path(e.target.value)}
                                            className="w-full bg-zinc-950/50 border border-zinc-800 rounded-lg px-3 py-2 text-[10px] text-zinc-400 focus:outline-none focus:border-blue-500/40 font-mono"
                                            aria-label={i18n.t('provider.local_server_path_label')}
                                        />
                                    </div>
                                    <p className="text-[9px] text-zinc-500 leading-tight italic px-1">
                                        {i18n.t('provider.local_server_desc', { defaultValue: 'Automate model driver lifecycle. Paste the executable path or launch URL above.' })}
                                    </p>
                                </div>
                            )}

                            {error_message && state.test_result === 'failed' && (
                                <div className="p-3 bg-red-500/10 border border-red-500/20 rounded-xl text-[10px] text-red-400 font-mono animate-in slide-in-from-top-2">
                                    <div className="flex items-center gap-2 mb-1">
                                        <AlertTriangle size={12} />
                                        <span className="font-bold uppercase tracking-wider text-red-500/80">{i18n.t('provider.handshake_failed_title')}</span>
                                    </div>
                                    {error_message}
                                </div>
                            )}

                            {state.test_result === 'success' && state.test_message && (
                                <div className="p-3 bg-emerald-500/10 border border-emerald-500/20 rounded-xl text-[10px] text-emerald-400 font-mono animate-in slide-in-from-top-2">
                                    <div className="flex items-center gap-2 mb-1">
                                        <ShieldCheck size={12} />
                                        <span className="font-bold uppercase tracking-wider text-emerald-500/80">{i18n.t('provider.handshake_success_title')}</span>
                                    </div>
                                    {state.test_message}
                                </div>
                            )}

                            <div className="space-y-1.5">
                                <div className="flex items-center justify-between px-1">
                                    <label htmlFor="provider-external-id" className="text-[11px] font-bold text-zinc-600 uppercase tracking-widest flex items-center gap-2">
                                        {i18n.t('provider.field_external_id')}
                                        <Tooltip content={i18n.t('provider.external_id_tooltip')} position="top">
                                            <Info size={11} className="text-zinc-700 hover:text-emerald-500 cursor-help transition-colors" />
                                        </Tooltip>
                                    </label>
                                </div>
                                <input
                                    id="provider-external-id"
                                    type="text"
                                    placeholder={i18n.t('provider.placeholder_external_id')}
                                    value={state.external_id}
                                    onChange={(e) => dispatch({ type: 'UPDATE_FIELD', field: 'external_id', value: e.target.value })}
                                    className="w-full bg-zinc-950 border border-zinc-800 rounded-xl px-4 py-2.5 text-xs text-zinc-200 focus:outline-none focus:border-emerald-500/40 transition-all font-mono"
                                    aria-label={i18n.t('provider.field_external_id_label')}
                                />
                            </div>

                            {/* Persistence Layer */}
                            <div className="p-4 bg-emerald-500/5 border border-emerald-500/10 rounded-2xl space-y-3">
                                <div className="flex items-center justify-between">
                                    <div className="space-y-0.5">
                                        <label className="text-[11px] font-bold text-emerald-500/80 uppercase tracking-widest flex items-center gap-2">
                                            {i18n.t('provider.field_persistence')}
                                            <Tooltip content={i18n.t('provider.persistence_tooltip')} position="top">
                                                <Info size={11} className="text-emerald-700 hover:text-emerald-500 cursor-help transition-colors" />
                                            </Tooltip>
                                        </label>
                                        <p className="text-[10px] text-zinc-500 leading-tight">
                                            {i18n.t('provider.persistence_desc')}
                                        </p>
                                    </div>
                                    <div className="flex items-center gap-2">
                                        <input
                                            type="checkbox"
                                            id="persist-to-engine"
                                            checked={state.persist_to_engine}
                                            onChange={(e) => dispatch({ type: 'UPDATE_FIELD', field: 'persist_to_engine', value: e.target.checked })}
                                            className="w-4 h-4 rounded border-zinc-800 bg-zinc-950 text-emerald-500 focus:ring-emerald-500/20 cursor-pointer"
                                        />
                                        <label htmlFor="persist-to-engine" className="text-sm font-bold text-zinc-300 cursor-pointer group-hover:text-zinc-100 transition-colors">
                                            {i18n.t('provider.label_persistence_layer')}
                                        </label>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </section>

                    {/* Advanced Configuration */}
                    <section className="space-y-4">
                        <h3 className="text-[11px] font-bold text-zinc-500 uppercase tracking-[0.2em] flex items-center gap-2">
                            <Zap size={12} className="text-blue-500/50" />
                            {i18n.t('provider.transmission_protocol')}
                        </h3>

                        <div className="grid grid-cols-2 gap-4">
                            <div className="space-y-1.5">
                                <label className="text-[11px] font-bold text-zinc-600 uppercase tracking-widest px-1 flex items-center gap-2">
                                    {i18n.t('provider.field_protocol')}
                                    <Tooltip content={i18n.t('provider.protocol_tooltip')} position="top">
                                        <Info size={11} className="text-zinc-700 hover:text-blue-500 cursor-help transition-colors" />
                                    </Tooltip>
                                </label>
                                <select
                                    value={state.protocol}
                                    onChange={(e) => dispatch({ type: 'UPDATE_FIELD', field: 'protocol', value: e.target.value })}
                                    className="w-full bg-zinc-950 border border-zinc-800 rounded-xl px-4 py-2.5 text-xs text-zinc-200 focus:outline-none focus:border-blue-500/40 font-mono cursor-pointer appearance-none"
                                    aria-label={i18n.t('provider.field_protocol_label')}
                                >
                                    <option value="openai">{i18n.t('provider.protocol_openai')}</option>
                                    <option value="anthropic">{i18n.t('provider.protocol_anthropic')}</option>
                                    <option value="google">{i18n.t('provider.protocol_google')}</option>
                                    <option value="ollama">{i18n.t('provider.protocol_ollama')}</option>
                                    <option value="deepseek">{i18n.t('provider.protocol_deepseek')}</option>
                                    <option value="inception">{i18n.t('provider.protocol_inception')}</option>
                                </select>
                            </div>
                            <div className="flex items-end">
                                <Tooltip content={i18n.t('provider.test_trace_tooltip')} position="top">
                                    <button
                                        onClick={handle_test_connection}
                                        disabled={state.is_testing}
                                        className={`w-full py-2.5 rounded-xl border flex items-center justify-center gap-2 text-[11px] font-bold uppercase tracking-widest transition-all ${state.test_result === 'success'
                                            ? 'bg-emerald-500/20 border-emerald-500/40 text-emerald-400'
                                            : 'bg-zinc-900 border-zinc-800 text-zinc-500 hover:text-zinc-200'
                                            }`}
                                        aria-label={i18n.t('provider.aria_test_connection')}
                                    >
                                        {state.is_testing ? (
                                            <>
                                                <Activity size={12} className="animate-spin" />
                                                {i18n.t('provider.tracing')}
                                            </>
                                        ) : state.test_result === 'success' ? (
                                            <>
                                                <Check size={12} />
                                                {i18n.t('provider.status_active')}
                                            </>
                                        ) : (
                                            <>
                                                <Activity size={12} />
                                                {i18n.t('provider.test_trace')}
                                            </>
                                        )}
                                    </button>
                                </Tooltip>
                            </div>
                        </div>

                        <div className="space-y-1.5">
                            <label className="text-[11px] font-bold text-zinc-600 uppercase tracking-widest px-1 flex items-center gap-2">
                                {i18n.t('provider.field_headers')}
                                <Tooltip content={i18n.t('provider.headers_tooltip')} position="top">
                                    <Info size={11} className="text-zinc-700 hover:text-blue-500 cursor-help transition-colors" />
                                </Tooltip>
                            </label>
                            <textarea
                                value={state.custom_headers}
                                onChange={(e) => dispatch({ type: 'UPDATE_FIELD', field: 'custom_headers', value: e.target.value })}
                                className="w-full bg-zinc-950 border border-zinc-800 rounded-xl px-4 py-3 text-[11px] text-zinc-400 focus:outline-none focus:border-blue-500/40 h-24 font-mono resize-none custom-scrollbar"
                                placeholder={i18n.t('provider.placeholder_headers')}
                                aria-label={i18n.t('provider.field_headers_label')}
                            />
                        </div>
                    </section>

                    {/* Audio Transcription Configuration */}
                    <section className="space-y-4">
                        <div className="flex items-center gap-2">
                            <h3 className="text-[11px] font-bold text-zinc-500 uppercase tracking-[0.2em] flex items-center gap-2">
                                <Zap size={12} className="text-amber-400/50" />
                                {i18n.t('provider.audio_service')}
                            </h3>
                        </div>
                        <div className="bg-zinc-950/50 border border-zinc-800 rounded-2xl p-4 space-y-4">
                            <div className="space-y-1.5">
                                <label htmlFor="provider-audio-model" className="text-[10px] font-bold text-zinc-600 uppercase tracking-widest px-1 flex items-center gap-2">
                                    {i18n.t('provider.field_audio_model')}
                                    <Tooltip content={i18n.t('provider.audio_model_tooltip')} position="top">
                                        <Info size={10} className="text-zinc-700 hover:text-blue-400 cursor-help transition-colors" />
                                    </Tooltip>
                                </label>
                                <div className="space-y-2">
                                    <input
                                        id="provider-audio-model"
                                        className="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-1.5 text-xs text-blue-400 focus:outline-none focus:border-blue-500 font-mono"
                                        placeholder={i18n.t('provider.placeholder_audio_model')}
                                        list="voice-inventory-suggestions"
                                        value={state.audio_model}
                                        onChange={e => dispatch({ type: 'UPDATE_FIELD', field: 'audio_model', value: e.target.value })}
                                        aria-label={i18n.t('provider.field_audio_model_label')}
                                    />
                                    <datalist id="voice-inventory-suggestions">
                                        <option value="whisper-large-v3" />
                                        {models
                                            .filter(m => m.provider === provider.id && m.modality?.toLowerCase() === 'voice')
                                            .map(m => (
                                                <option key={m.id} value={m.name}>
                                                    {m.name.toUpperCase()} ({i18n.t('provider.label_inventory')})
                                                </option>
                                            ))}
                                    </datalist>
                                </div>
                                <p className="text-[9px] text-zinc-600 font-medium px-1 italic">
                                    {i18n.t('provider.audio_model_hint')}
                                </p>
                            </div>
                        </div>
                    </section>

                    {/* Model Forge section */}
                    <section className="space-y-4">
                        <div className="flex items-center justify-between">
                            <h3 className="text-[11px] font-bold text-zinc-500 uppercase tracking-[0.2em] flex items-center gap-2">
                                <Cpu size={12} className="text-blue-400/50" />
                                {i18n.t('provider.intelligence_forge')}
                            </h3>
                            <Tooltip content={i18n.t('provider.add_node_tooltip')} position="left">
                                <button
                                    onClick={() => set_is_forge_adding(true)}
                                    className="flex items-center gap-1.5 text-[11px] font-bold text-blue-400 hover:bg-blue-400/10 px-3 py-1.5 rounded-lg border border-blue-400/20 transition-all uppercase tracking-widest"
                                    aria-label={i18n.t('provider.aria_add_node')}
                                >
                                    <Plus size={12} /> {i18n.t('provider.add_node')}
                                </button>
                            </Tooltip>
                        </div>

                        <div className="bg-zinc-950/50 border border-zinc-800 rounded-2xl overflow-hidden divide-y divide-zinc-800/50">
                            {is_forge_adding && (
                                <div className="p-4 bg-blue-500/5 space-y-4 animate-in fade-in slide-in-from-top-2 duration-300">
                                    <div className="flex gap-3">
                                        <input
                                            id="forge-model-name"
                                            className="bg-zinc-950 border border-blue-500/30 rounded-lg px-3 py-1.5 text-xs text-zinc-100 flex-1 focus:outline-none focus:border-blue-500 font-mono"
                                            placeholder={i18n.t('provider.placeholder_forge_model')}
                                            autoFocus
                                            list="model-inventory-suggestions"
                                            value={forge_new_model.name}
                                            onChange={e => set_forge_new_model({ ...forge_new_model, name: e.target.value })}
                                            aria-label={i18n.t('provider.forge_name_label')}
                                        />
                                        <datalist id="model-inventory-suggestions">
                                            {MODEL_OPTIONS
                                                .filter(opt => resolve_provider(opt) === provider.id)
                                                .map(opt => (
                                                    <option key={opt} value={opt} />
                                                ))}
                                        </datalist>
                                        <select
                                            className="bg-zinc-950 border border-blue-500/30 rounded-lg px-3 py-1.5 text-xs text-zinc-100 focus:outline-none cursor-pointer font-mono"
                                            value={is_forge_custom_modality ? 'other' : forge_new_model.modality}
                                            onChange={e => {
                                                if (e.target.value === 'other') {
                                                    set_is_forge_custom_modality(true);
                                                } else {
                                                    set_is_forge_custom_modality(false);
                                                    set_forge_new_model({ ...forge_new_model, modality: e.target.value as Model_Entry['modality'] });
                                                }
                                            }}
                                            aria-label={i18n.t('provider.forge_modality_label')}
                                        >
                                            <option value="llm">{i18n.t('provider.label_modality_llm')}</option>
                                            <option value="vision">{i18n.t('provider.label_modality_vision')}</option>
                                            <option value="voice">{i18n.t('provider.label_modality_voice')}</option>
                                            <option value="reasoning">{i18n.t('provider.label_modality_reasoning')}</option>
                                            <option value="other">{i18n.t('provider.label_modality_other')}</option>
                                        </select>
                                    </div>
                                    {is_forge_custom_modality && (
                                        <input
                                            id="forge-custom-modality"
                                            className="bg-zinc-950 border border-blue-500/30 rounded-lg px-3 py-1.5 text-xs text-zinc-100 w-full focus:outline-none focus:border-blue-500 font-mono"
                                            placeholder={i18n.t('provider.placeholder_custom_modality')}
                                            value={forge_custom_modality}
                                            onChange={e => set_forge_custom_modality(e.target.value)}
                                            aria-label={i18n.t('provider.custom_modality_label')}
                                        />
                                    )}
                                    <div className="grid grid-cols-2 gap-4">
                                        <div className="space-y-1">
                                            <label htmlFor="forge-rpm" className="text-[10px] font-bold text-zinc-600 uppercase tracking-widest px-1 flex items-center gap-1.5">
                                                {i18n.t('provider.forge_item.label_rpm')}
                                                <Tooltip content={i18n.t('provider.rpm_tooltip')} position="top">
                                                    <Info size={9} className="text-zinc-700 hover:text-blue-400 cursor-help transition-colors" />
                                                </Tooltip>
                                            </label>
                                            <input
                                                id="forge-rpm"
                                                type="number"
                                                className="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-1.5 text-xs text-zinc-300 font-mono focus:border-blue-500/40 outline-none"
                                                value={forge_new_model.rpm}
                                                onChange={e => set_forge_new_model({ ...forge_new_model, rpm: parseInt(e.target.value) || 0 })}
                                                aria-label={i18n.t('provider.field_rpm_label')}
                                            />
                                        </div>
                                        <div className="space-y-1">
                                            <label htmlFor="forge-tpm" className="text-[10px] font-bold text-zinc-600 uppercase tracking-widest px-1 flex items-center gap-1.5">
                                                {i18n.t('provider.forge_item.label_tpm')}
                                                <Tooltip content={i18n.t('provider.tpm_tooltip')} position="top">
                                                    <Info size={9} className="text-zinc-700 hover:text-blue-400 cursor-help transition-colors" />
                                                </Tooltip>
                                            </label>
                                            <input
                                                type="number"
                                                className="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-1.5 text-xs text-zinc-300 font-mono focus:border-blue-500/40 outline-none"
                                                value={forge_new_model.tpm}
                                                onChange={e => set_forge_new_model({ ...forge_new_model, tpm: parseInt(e.target.value) || 0 })}
                                                aria-label={i18n.t('provider.field_tpm_label')}
                                            />
                                        </div>
                                        <div className="space-y-1">
                                            <label className="text-[10px] font-bold text-zinc-600 uppercase tracking-widest px-1">{i18n.t('provider.field_rpd')}</label>
                                            <input
                                                type="number"
                                                className="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-1.5 text-xs text-zinc-300 font-mono focus:border-blue-500/40 outline-none"
                                                value={forge_new_model.rpd}
                                                onChange={e => set_forge_new_model({ ...forge_new_model, rpd: parseInt(e.target.value) || 0 })}
                                                aria-label={i18n.t('provider.field_rpd_label')}
                                            />
                                        </div>
                                        <div className="space-y-1">
                                            <label className="text-[10px] font-bold text-zinc-600 uppercase tracking-widest px-1">{i18n.t('provider.field_tpd')}</label>
                                            <input
                                                type="number"
                                                className="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-1.5 text-xs text-zinc-300 font-mono focus:border-blue-500/40 outline-none"
                                                value={forge_new_model.tpd}
                                                onChange={e => set_forge_new_model({ ...forge_new_model, tpd: parseInt(e.target.value) || 0 })}
                                                aria-label={i18n.t('provider.field_tpd_label')}
                                            />
                                        </div>
                                    </div>
                                    <div className="flex justify-end gap-2 pt-2 border-t border-zinc-800/50">
                                        <Tooltip content={i18n.t('provider.commit_node_tooltip')} position="bottom">
                                            <button onClick={handle_forge_add} className="p-1.5 bg-blue-500/20 text-blue-400 rounded hover:bg-blue-500/30 transition-colors">
                                                <Check size={16} />
                                            </button>
                                        </Tooltip>
                                        <Tooltip content={i18n.t('provider.cancel_node_tooltip')} position="bottom">
                                            <button onClick={() => set_is_forge_adding(false)} className="p-1.5 bg-zinc-800 text-zinc-500 rounded hover:bg-zinc-700 transition-colors">
                                                <X size={16} />
                                            </button>
                                        </Tooltip>
                                    </div>
                                </div>
                            )}

                            {provider_models.length === 0 ? (
                                <div className="p-8 text-center">
                                    <div className="inline-flex p-3 rounded-full bg-zinc-900 border border-zinc-800 text-zinc-600 mb-3">
                                        <Cpu size={24} />
                                    </div>
                                    <p className="text-xs text-zinc-500 font-mono uppercase tracking-widest">{i18n.t('provider.no_nodes')}</p>
                                </div>
                            ) : (
                                provider_models.map((m) => (
                                    <Forge_Item
                                        key={m.id}
                                        model={m}
                                        is_editing={editing_model_id === m.id}
                                        on_edit={() => set_editing_model_id(m.id)}
                                        on_cancel={() => set_editing_model_id(null)}
                                        on_save={(id, name, prov, modality, limits) => {
                                            edit_model(id, name, prov, modality, limits);
                                            set_editing_model_id(null);
                                        }}
                                        on_delete={() => delete_model(m.id)}
                                    />
                                ))
                            )}
                        </div>
                    </section>
                </div>

                {/* Footer */}
                <div className="p-4 border-t border-zinc-800 bg-zinc-900 shrink-0">
                    <Tooltip content={i18n.t('provider.sync_tooltip')} position="top">
                        <button
                            onClick={handle_save}
                            disabled={state.is_testing}
                            className="w-full flex items-center justify-center gap-2 px-4 py-3 rounded-xl bg-emerald-600 text-white text-[11px] font-bold hover:bg-emerald-500 shadow-lg shadow-emerald-500/20 disabled:opacity-50 transition-all active:scale-[0.98] uppercase tracking-widest"
                        >
                            <Save size={16} />
                            {state.is_testing ? i18n.t('provider.syncing') : i18n.t('provider.commit_auth')}
                        </button>
                    </Tooltip>
                </div>
            </div>
        </div>
    );
}

