/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Assist Note
 * - Keep side effects at service boundaries and preserve the public contracts of shared domain transitions.
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / General / Provider_Config_Panel
 * - **Primary Entrypoints**: `Provider_Config_Panel`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import React, { useState, useReducer, useMemo } from 'react';
import { Save } from 'lucide-react';
import { Tooltip } from './ui';
import { provider_configuration } from '../services/provider_configuration';
import { credential_change } from '../domain/providers/configuration';
import { use_vault_store } from '../stores/vault_store';
import { use_model_store } from '../stores/model_store';
import type { Provider_Config } from '../stores/provider_store';
import { event_bus } from '../services/event_bus';
import { i18n } from '../i18n';
import { panel_reducer, type Panel_State } from '../hooks/use_provider_form';

// Modular Sub-components
import { Identity_Header } from './provider/Identity_Header';
import { Auth_Section } from './provider/Auth_Section';
import { Local_Server_Module } from './provider/Local_Server_Module';
import { Protocol_Section } from './provider/Protocol_Section';
import { Audio_Settings } from './provider/Audio_Settings';
import { Model_Forge } from './provider/Model_Forge';

/**
 * Provider_Config_Panel_Props
 */
interface Provider_Config_Panel_Props {
    provider: Provider_Config;
    on_close: () => void;
}

/**
 * Provider_Config_Panel
 * Refactored modular version of the provider configuration panel.
 */
export default function Provider_Config_Panel({ provider, on_close }: Provider_Config_Panel_Props): React.ReactElement {
    // Only use store for state, services for actions
    const { encrypted_configs } = use_vault_store();
    const { models, add_model, edit_model, delete_model } = use_model_store();
    const has_saved_key = !!encrypted_configs[provider.id];

    // Models for this provider
    const provider_models = useMemo(() =>
        models.filter(m => m.provider === provider.id)
            .sort((a, b) => a.name.localeCompare(b.name)),
        [models, provider.id]
    );

    const [state, dispatch] = useReducer(panel_reducer, {
        name: provider.name,
        icon: provider.icon || '⚡',
        api_key: '', 
        base_url: provider.base_url || '',
        external_id: provider.external_id || '',
        protocol: provider.protocol || (
            provider.id === 'ollama' || provider.id === 'local'
                ? 'ollama'
                : (['anthropic', 'google', 'groq', 'inception', 'deepseek'].includes(provider.id) ? provider.id : 'openai')
        ),
        custom_headers: JSON.stringify(provider.custom_headers || {}, null, 2),
        audio_model: provider.audio_model || '',
        persist_to_engine: provider.persist_to_engine || false,
        supports_steering_vectors: provider.supports_steering_vectors || false,
        is_testing: false,
        is_syncing: false,
        test_result: 'idle',
        test_message: ''
    });

    const [error_message, set_error_message] = useState<string>('');
    const [remove_key, set_remove_key] = useState(false);
    const draft = () => ({ ...state, metadata: provider.metadata });
    const credential = () => credential_change(state.api_key, remove_key);
    const [local_server_path, set_local_server_path] = useState<string>((provider.metadata?.local_server_path as string) || '');

    const handle_start_server = (): void => {
        event_bus.emit_log({
            source: 'System',
            text: i18n.t('provider.msg_boot_sequence', { target: local_server_path || i18n.t('provider.msg_boot_driver_fallback') }),
            severity: 'info'
        });
    };

    const handle_stop_server = (): void => {
          event_bus.emit_log({
            source: 'System',
            text: i18n.t('provider.msg_termination_signal'),
            severity: 'warning'
        });
    };

    const handle_save = async (): Promise<void> => {
        dispatch({ type: 'UPDATE_FIELD', field: 'is_testing', value: true });

        try {
            await provider_configuration.save(provider.id, draft(), credential());

            event_bus.emit_log({
                source: 'System',
                text: i18n.t('provider.save_success', { name: state.name }),
                severity: 'success'
            });

            on_close();
        } catch (error) {
            const message = error instanceof Error ? error.message : i18n.t('provider.error_vault_unknown');
            set_error_message(message);
            dispatch({ type: 'UPDATE_FIELD', field: 'test_result', value: 'failed' });
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
        dispatch({ type: 'UPDATE_FIELD', field: 'is_testing', value: true });
        dispatch({ type: 'UPDATE_FIELD', field: 'test_result', value: 'idle' });
        dispatch({ type: 'UPDATE_FIELD', field: 'test_message', value: '' });

        try {
            const result = await provider_configuration.test(provider.id, draft(), credential());

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
        } catch (error) {
            dispatch({ type: 'UPDATE_FIELD', field: 'is_testing', value: false });
            dispatch({ type: 'UPDATE_FIELD', field: 'test_result', value: 'failed' });
            const message = error instanceof Error ? error.message : i18n.t('provider.handshake_failed');
            set_error_message(message);
            event_bus.emit_log({
                source: 'System',
                text: `${i18n.t('provider.trace_error')}: ${message}`,
                severity: 'error'
            });
        }
    };

    const handle_sync_models = async (): Promise<void> => {
        dispatch({ type: 'UPDATE_FIELD', field: 'is_syncing', value: true });
        
        try {
            const result = await provider_configuration.discover(provider.id, draft(), credential());
            set_error_message('');
            
            event_bus.emit_log({
                source: 'System',
                text: result.message,
                severity: 'success'
            });

        } catch (error) {
            const message = error instanceof Error ? error.message : i18n.t('provider.error_discovery_failed');
            set_error_message(message);
            event_bus.emit_log({
                source: 'System',
                text: `${i18n.t('provider.error_discovery_prefix')}: ${message}`,
                severity: 'error'
            });
        } finally {
            dispatch({ type: 'UPDATE_FIELD', field: 'is_syncing', value: false });
        }
    };

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
            <div 
                className="absolute inset-0 bg-zinc-950/60 backdrop-blur-sm animate-in fade-in duration-300" 
                onClick={on_close}
                onKeyDown={(e) => { if (e.key === 'Escape') on_close(); }}
                role="button"
                tabIndex={-1}
                aria-label={i18n.t('common.close_modal', { defaultValue: 'Close Modal' })}
            />

            <div className="relative w-full max-w-2xl bg-[color:var(--color-background)] border border-[color:var(--color-border)] rounded-3xl shadow-2xl flex flex-col overflow-hidden animate-in zoom-in-95 duration-300 max-h-[85vh]">
                <Identity_Header 
                    name={state.name} 
                    icon={state.icon} 
                    provider_id={provider.id}
                    on_name_change={(val) => dispatch({ type: 'UPDATE_FIELD', field: 'name', value: val })}
                    on_close={on_close}
                />

                <div className="flex-1 overflow-y-auto p-6 space-y-8 custom-scrollbar relative">
                    <div className="neural-grid opacity-[0.03]" />

                    <label className="flex items-center gap-2 px-6 text-xs text-zinc-400">
                        <input type="checkbox" checked={remove_key} onChange={event => set_remove_key(event.target.checked)} />
                        {i18n.t('provider.remove_saved_key', { defaultValue: 'Remove saved API key on save' })}
                    </label>
                    <Auth_Section
                        api_key={state.api_key}
                        base_url={state.base_url}
                        external_id={state.external_id}
                        persist_to_engine={state.persist_to_engine}
                        has_saved_key={has_saved_key}
                        test_result={state.test_result}
                        test_message={state.test_message}
                        error_message={error_message}
                        on_change={(field, value) => dispatch({ type: 'UPDATE_FIELD', field: field as keyof Panel_State, value: value as string | boolean })}
                    >
                        {state.base_url.toLowerCase().includes('localhost') && (
                            <Local_Server_Module 
                                path={local_server_path}
                                on_path_change={set_local_server_path}
                                on_start={handle_start_server}
                                on_stop={handle_stop_server}
                            />
                        )}
                    </Auth_Section>

                    <Protocol_Section
                        protocol={state.protocol || 'openai'}
                        custom_headers={state.custom_headers}
                        is_testing={state.is_testing}
                        is_syncing={state.is_syncing}
                        test_result={state.test_result}
                        on_change={(field, value) => dispatch({ type: 'UPDATE_FIELD', field: field as keyof Panel_State, value: value as string | boolean })}
                        on_test_connection={handle_test_connection}
                        on_sync_models={handle_sync_models}
                    />

                    <Audio_Settings
                        audio_model={state.audio_model}
                        provider_id={provider.id}
                        models={models}
                        on_change={(field, value) => dispatch({ type: 'UPDATE_FIELD', field: field as keyof Panel_State, value: value as string | boolean })}
                    />

                    <Model_Forge
                        provider_id={provider.id}
                        provider_models={provider_models}
                        add_model={add_model}
                        edit_model={edit_model}
                        delete_model={delete_model}
                    />
                </div>

                <div className="p-4 border-t border-[color:var(--color-border)] bg-[color:var(--color-surface)] shrink-0">
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
