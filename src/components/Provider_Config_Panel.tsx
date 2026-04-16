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
import { Save } from 'lucide-react';
import { Tooltip } from './ui';
import { use_provider_store } from '../stores/provider_store';
import { use_vault_store } from '../stores/vault_store';
import { use_model_store } from '../stores/model_store';
import type { Provider_Config } from '../stores/provider_store';
import { event_bus } from '../services/event_bus';
import { tadpole_os_service } from '../services/tadpoleos_service';
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
    const { edit_provider, set_provider_config } = use_provider_store();
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
        protocol: provider.protocol || 'openai',
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
            edit_provider(provider.id, state.name, state.icon);

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
                state.protocol as Provider_Config['protocol'], // Cast to literal type
                parsed_headers,
                state.audio_model,
                state.persist_to_engine,
                state.supports_steering_vectors
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
                persist_to_engine: state.persist_to_engine,
                supports_steering_vectors: state.supports_steering_vectors
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
        } catch (error) {
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

    const handle_sync_models = async (): Promise<void> => {
        dispatch({ type: 'UPDATE_FIELD', field: 'is_syncing', value: true });
        
        try {
            const result = await tadpole_os_service.sync_provider_models(provider.id);
            
            event_bus.emit_log({
                source: 'System',
                text: result.message,
                severity: 'success'
            });

            // Trigger a re-sync of the model registry locally
            await use_model_store.getState().sync_models();
        } catch (error) {
            const message = error instanceof Error ? error.message : 'Discovery sequence failed.';
            event_bus.emit_log({
                source: 'System',
                text: `[IMR] Discovery Error: ${message}`,
                severity: 'error'
            });
        } finally {
            dispatch({ type: 'UPDATE_FIELD', field: 'is_syncing', value: false });
        }
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
                <Identity_Header 
                    name={state.name} 
                    icon={state.icon} 
                    provider_id={provider.id}
                    on_name_change={(val) => dispatch({ type: 'UPDATE_FIELD', field: 'name', value: val })}
                    on_close={on_close}
                />

                <div className="flex-1 overflow-y-auto p-6 space-y-8 custom-scrollbar relative">
                    <div className="neural-grid opacity-[0.03]" />

                    <Auth_Section
                        api_key={state.api_key}
                        base_url={state.base_url}
                        external_id={state.external_id}
                        persist_to_engine={state.persist_to_engine}
                        has_saved_key={has_saved_key}
                        test_result={state.test_result}
                        test_message={state.test_message}
                        error_message={error_message}
                        on_change={(field, value) => dispatch({ type: 'UPDATE_FIELD', field: field as keyof Panel_State, value: value as string | boolean | 'idle' | 'success' | 'failed' })}

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
                        on_change={(field, value) => dispatch({ type: 'UPDATE_FIELD', field: field as keyof Panel_State, value: value as string | boolean | 'idle'| 'success' | 'failed' })}

                        on_test_connection={handle_test_connection}
                        on_sync_models={handle_sync_models}
                    />

                    <Audio_Settings
                        audio_model={state.audio_model}
                        provider_id={provider.id}
                        models={models}
                        on_change={(field, value) => dispatch({ type: 'UPDATE_FIELD', field: field as keyof Panel_State, value: value as string | boolean | 'idle' | 'success' | 'failed' })}

                    />

                    <Model_Forge
                        provider_id={provider.id}
                        provider_models={provider_models}
                        add_model={add_model}
                        edit_model={edit_model}
                        delete_model={delete_model}
                    />
                </div>

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
