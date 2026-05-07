/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Sub-module**: Core authentication and credential gateway. 
 * Facilitates secure API key entry, vault persistence status, and endpoint validation.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: API key vault write failure (401/403), or invalid endpoint URL format.
 * - **Telemetry Link**: Search for `[Auth_Layer]` or `set_encrypted_config` in UI tracing.
 */

import React from 'react';
import { ShieldCheck, Key, Globe, Info, AlertTriangle } from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';

/**
 * Auth_Section_Props
 * Defines the props for the Auth_Section component.
 */
interface Auth_Section_Props {
    /** Current API key */
    api_key: string;
    /** Current base URL */
    base_url: string;
    /** Current external ID */
    external_id: string;
    /** Persistence toggle status */
    persist_to_engine: boolean;
    /** Whether a key was previously saved in the vault */
    has_saved_key: boolean;
    /** Connection test outcome */
    test_result: 'idle' | 'success' | 'failed';
    /** Message from the connection test */
    test_message: string;
    /** Errors from validation or test failures */
    error_message: string;
    /** Update handler for form fields */
    on_change: (field: string, value: string | boolean) => void;
    /** Optional local server orchestration module */
    children?: React.ReactNode;
}

/**
 * Auth_Section
 * Handles API authentication, endpoints, and vault persistence settings.
 */
export function Auth_Section({
    api_key,
    base_url,
    external_id,
    persist_to_engine,
    has_saved_key,
    test_result,
    test_message,
    error_message,
    on_change,
    children
}: Auth_Section_Props): React.ReactElement {
    return (
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
                            value={api_key}
                            onChange={(e) => on_change('api_key', e.target.value)}
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
                            value={base_url}
                            onChange={(e) => on_change('base_url', e.target.value)}
                            className="w-full bg-zinc-950 border border-zinc-800 rounded-xl pl-10 pr-4 py-2.5 text-xs text-zinc-200 focus:outline-none focus:border-emerald-500/40 transition-all font-mono"
                            aria-label={i18n.t('provider.field_endpoint_label')}
                        />
                    </div>
                </div>

                {children}

                {error_message && test_result === 'failed' && (
                    <div className="p-3 bg-red-500/10 border border-red-500/20 rounded-xl text-[10px] text-red-400 font-mono animate-in slide-in-from-top-2">
                        <div className="flex items-center gap-2 mb-1">
                            <AlertTriangle size={12} />
                            <span className="font-bold uppercase tracking-wider text-red-500/80">{i18n.t('provider.handshake_failed_title')}</span>
                        </div>
                        {error_message}
                    </div>
                )}

                {test_result === 'success' && test_message && (
                    <div className="p-3 bg-emerald-500/10 border border-emerald-500/20 rounded-xl text-[10px] text-emerald-400 font-mono animate-in slide-in-from-top-2">
                        <div className="flex items-center gap-2 mb-1">
                            <ShieldCheck size={12} />
                            <span className="font-bold uppercase tracking-wider text-emerald-500/80">{i18n.t('provider.handshake_success_title')}</span>
                        </div>
                        {test_message}
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
                        value={external_id}
                        onChange={(e) => on_change('external_id', e.target.value)}
                        className="w-full bg-zinc-950 border border-zinc-800 rounded-xl px-4 py-2.5 text-xs text-zinc-200 focus:outline-none focus:border-emerald-500/40 transition-all font-mono"
                        aria-label={i18n.t('provider.field_external_id_label')}
                    />
                </div>

                <div className="p-4 bg-emerald-500/5 border border-emerald-500/10 rounded-2xl space-y-3">
                    <div className="flex items-center justify-between">
                        <div className="space-y-0.5">
                            <label htmlFor="persist-to-engine" className="text-[11px] font-bold text-emerald-500/80 uppercase tracking-widest flex items-center gap-2 cursor-pointer">
                                {i18n.t('provider.field_persistence')}
                                <Tooltip content={i18n.t('provider.persistence_tooltip')} position="top">
                                    <Info size={11} className="text-emerald-700 hover:text-emerald-500 cursor-help transition-colors" />
                                </Tooltip>
                            </label>
                            <p className="text-[10px] text-zinc-500 leading-tight">
                                {i18n.t('provider.persistence_desc')}
                            </p>
                        </div>
                        <div className="flex items-center gap-3">
                            {has_saved_key && (
                                <span className="text-[9px] text-emerald-500 font-bold uppercase tracking-tighter bg-emerald-500/10 border border-emerald-500/20 px-2 py-0.5 rounded flex items-center gap-1">
                                    <ShieldCheck size={10} />
                                    {i18n.t('provider.status_saved')}
                                </span>
                            )}
                            <input
                                type="checkbox"
                                id="persist-to-engine"
                                checked={persist_to_engine}
                                onChange={(e) => on_change('persist_to_engine', e.target.checked)}
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
    );
}

// Metadata: [Auth_Section]

// Metadata: [Auth_Section]
