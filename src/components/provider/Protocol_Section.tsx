/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Sub-module**: Transmission protocol and header coordinator. 
 * Manages the selection of API dialects (OpenAI/Anthropic/Google/Ollama) and the injection of custom HTTP headers.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Invalid JSON in `custom_headers`, or mismatched protocol/endpoint combination (405).
 * - **Telemetry Link**: Search for `Handshake: Invalid JSON headers` in UI logs.
 */

import React from 'react';
import { Zap, Activity, Check, Info } from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';

/**
 * Protocol_Section_Props
 * Defines the props for the Protocol_Section component.
 */
interface Protocol_Section_Props {
    /** Current selected protocol */
    protocol: string;
    /** JSON string for custom headers */
    custom_headers: string;
    /** Whether this provider supports OBLITERATUS Activation Addition steering vectors */
    supports_steering_vectors?: boolean;
    /** Whether a connection test is currently running */
    is_testing: boolean;
    /** Result of the last connection test */
    /** Whether a synchronization is currently running */
    is_syncing?: boolean;
    /** Result of the last connection test */
    test_result: 'idle' | 'success' | 'failed';
    /** Update handler for form fields */
    on_change: (field: string, value: string | boolean) => void;
    /** Callback to trigger the connection test */
    on_test_connection: () => void;
    /** Callback to trigger model synchronization */
    on_sync_models?: () => void;
}

/**
 * Protocol_Section
 * Handles API transmission protocols and advanced HTTP header configurations.
 */
export function Protocol_Section({
    protocol,
    custom_headers,
    supports_steering_vectors,
    is_testing,
    is_syncing,
    test_result,
    on_change,
    on_test_connection,
    on_sync_models
}: Protocol_Section_Props): React.ReactElement {
    return (
        <section className="space-y-4">
            <h3 className="text-[11px] font-bold text-zinc-500 uppercase tracking-[0.2em] flex items-center gap-2">
                <Zap size={12} className="text-blue-500/50" />
                {i18n.t('provider.transmission_protocol')}
            </h3>

            <div className="grid grid-cols-12 gap-4">
                <div className="col-span-6 space-y-1.5">
                    <label className="text-[11px] font-bold text-zinc-600 uppercase tracking-widest px-1 flex items-center gap-2">
                        {i18n.t('provider.field_protocol')}
                        <Tooltip content={i18n.t('provider.protocol_tooltip')} position="top">
                            <Info size={11} className="text-zinc-700 hover:text-blue-500 cursor-help transition-colors" />
                        </Tooltip>
                    </label>
                    <select
                        value={protocol}
                        onChange={(e) => on_change('protocol', e.target.value)}
                        className="w-full bg-zinc-950 border border-zinc-800 rounded-xl px-4 py-2.5 text-xs text-zinc-200 focus:outline-none focus:border-blue-500/40 font-mono cursor-pointer appearance-none"
                        aria-label={i18n.t('provider.field_protocol_label')}
                    >
                        <option value="openai">{i18n.t('provider.protocol_openai')}</option>
                        <option value="anthropic">{i18n.t('provider.protocol_anthropic')}</option>
                        <option value="google">{i18n.t('provider.protocol_google')}</option>
                        <option value="ollama">{i18n.t('provider.protocol_ollama')}</option>
                        <option value="deepseek">{i18n.t('provider.protocol_deepseek')}</option>
                        <option value="inception">{i18n.t('provider.protocol_inception')}</option>
                        <option value="groq">{i18n.t('provider.protocol_groq', { defaultValue: 'Groq' })}</option>
                    </select>
                </div>
                <div className="col-span-3 flex items-end">
                    <Tooltip content={i18n.t('provider.test_trace_tooltip')} position="top">
                        <button
                            onClick={on_test_connection}
                            disabled={is_testing || is_syncing}
                            className={`w-full py-2.5 rounded-xl border flex items-center justify-center gap-2 text-[11px] font-bold uppercase tracking-widest transition-all ${test_result === 'success'
                                ? 'bg-emerald-500/20 border-emerald-500/40 text-emerald-400'
                                : 'bg-zinc-900 border-zinc-800 text-zinc-500 hover:text-zinc-200'
                                } disabled:opacity-30`}
                            aria-label={i18n.t('provider.aria_test_connection')}
                        >
                            {is_testing ? (
                                <Activity size={12} className="animate-spin" />
                            ) : test_result === 'success' ? (
                                <Check size={12} />
                            ) : (
                                <Activity size={12} />
                            )}
                        </button>
                    </Tooltip>
                </div>
                <div className="col-span-3 flex items-end">
                     <Tooltip content={i18n.t('provider.sync_discovery_tooltip', { defaultValue: 'Discover & Enrich Models' })} position="top">
                        <button
                            onClick={on_sync_models}
                            disabled={is_testing || is_syncing}
                            className="w-full py-2.5 rounded-xl border border-blue-500/30 bg-blue-500/10 text-blue-400 hover:bg-blue-500/20 flex items-center justify-center gap-2 text-[11px] font-bold uppercase tracking-widest transition-all disabled:opacity-30"
                        >
                            {is_syncing ? (
                                <Zap size={12} className="animate-pulse" />
                            ) : (
                                <Zap size={12} />
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
                    value={custom_headers}
                    onChange={(e) => on_change('custom_headers', e.target.value)}
                    className="w-full bg-zinc-950 border border-zinc-800 rounded-xl px-4 py-3 text-[11px] text-zinc-400 focus:outline-none focus:border-blue-500/40 h-24 font-mono resize-none custom-scrollbar"
                    placeholder={i18n.t('provider.placeholder_headers')}
                    aria-label={i18n.t('provider.field_headers_label')}
                />
            </div>

            <div className="p-4 bg-purple-500/5 border border-purple-500/10 rounded-2xl space-y-3">
                <div className="flex items-center justify-between">
                    <div className="space-y-0.5">
                        <label className="text-[11px] font-bold text-purple-500/80 uppercase tracking-widest flex items-center gap-2">
                            {i18n.t('provider.field_steering_vectors', { defaultValue: 'Activation Addition (Vectors)' })}
                            <Tooltip content={i18n.t('provider.steering_vectors_tooltip', { defaultValue: 'If supported by the local backend (OBLITERATUS methodology), strips heavy system prompts and injects persona dynamically via algebraic weights.' })} position="top">
                                <Info size={11} className="text-purple-700 hover:text-purple-500 cursor-help transition-colors" />
                            </Tooltip>
                        </label>
                        <p className="text-[10px] text-zinc-500 leading-tight">
                            {i18n.t('provider.steering_vectors_desc', { defaultValue: 'Enable for Local LLMs to optimize context window and ensure strict persona adherence.' })}
                        </p>
                    </div>
                    <div className="flex items-center gap-3">
                        {supports_steering_vectors && (
                            <span className="text-[9px] text-purple-500 font-bold uppercase tracking-tighter bg-purple-500/10 border border-purple-500/20 px-2 py-0.5 rounded flex items-center gap-1 animate-pulse">
                                <Zap size={10} />
                                ON
                            </span>
                        )}
                        <input
                            type="checkbox"
                            id="supports-steering-vectors"
                            checked={!!supports_steering_vectors}
                            onChange={(e) => on_change('supports_steering_vectors', e.target.checked)}
                            className="w-4 h-4 rounded border-zinc-800 bg-zinc-950 text-purple-500 focus:ring-purple-500/20 cursor-pointer"
                        />
                    </div>
                </div>
            </div>
        </section>
    );
}
