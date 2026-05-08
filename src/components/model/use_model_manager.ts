/**
 * @docs ARCHITECTURE:Components
 * 
 * ### AI Assist Note
 * **Core functional element for the Tadpole OS engine.**
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path: Runtime logic error or state corruption.**
 * - **Telemetry Link**: Search `[use_model_manager.ts]` in tracing logs.
 */

/**
 * @docs ARCHITECTURE:Logic
 * 
 * ### AI Assist Note
 * **Root/Core**: Manages the use model manager. 
 * Part of the Tadpole-OS core layer.
 */

import { useState, useMemo } from 'react';
import { use_provider_store } from '../../stores/provider_store';
import { use_vault_store } from '../../stores/vault_store';
import { use_model_store } from '../../stores/model_store';
import type { Model_Entry } from '../../stores/provider_store';
import { i18n } from '../../i18n';
import { ValidationUtils } from '../../utils/validation_utils';
import { event_bus } from '../../services/event_bus';

export function useModelManager() {
    const { unlock, lock, reset_vault } = use_vault_store();
    const { providers, add_provider, delete_provider } = use_provider_store();
    const { models, add_model, edit_model, delete_model } = use_model_store();

    const [password_input, set_password_input] = useState('');
    const [error, set_error] = useState<string | null>(null);

    // Navigation & UI States
    const [selected_provider_id, set_selected_provider_id] = useState<string | null>(null);
    const [modality_filter, set_modality_filter] = useState<'all' | Model_Entry['modality']>('all');
    const [editing_id, set_editing_id] = useState<string | null>(null);

    const [provider_form, set_provider_form] = useState({
        is_adding: false,
        name: '',
        icon: '⚡'
    });

    const [node_form, set_node_form] = useState({
        is_adding: false,
        name: '',
        provider: '',
        modality: 'llm' as Model_Entry['modality'],
        rpm: 10,
        tpm: 100000,
        rpd: 1000,
        tpd: 10000000,
        is_custom_modality: false,
        custom_modality: ''
    });

    // Confirmation State
    const [confirm_delete, set_confirm_delete] = useState<{
        type: 'provider' | 'model';
        id: string;
        name: string;
    } | null>(null);
    const [show_reset_confirm, set_show_reset_confirm] = useState(false);

    const is_secure = typeof window !== 'undefined' && (
        window.isSecureContext || 
        window.location.hostname === 'localhost' || 
        window.location.hostname === '127.0.0.1'
    );

    const handle_unlock = async (): Promise<void> => {
        set_error(null);
        const result = await unlock(password_input);
        if (!result.success) {
            set_error(result.error?.toUpperCase() || i18n.t('model_manager.vault.error_invalid'));
        }
        set_password_input('');
    };

    const selected_provider = useMemo(() =>
        providers.find(p => p.id === selected_provider_id),
        [providers, selected_provider_id]
    );

    const filtered_models = useMemo(() => {
        let filtered = [...models];
        if (modality_filter !== 'all') {
            filtered = filtered.filter(m => m.modality === modality_filter);
        }
        return filtered.sort((a, b) => a.name.localeCompare(b.name));
    }, [models, modality_filter]);

    const handle_add_provider = async () => {
        try {
            await add_provider(provider_form.name, provider_form.icon);
            set_provider_form({ is_adding: false, name: '', icon: '⚡' });
        } catch (e: unknown) {
            set_error(e instanceof Error ? e.message : 'Unknown error');
        }
    };

    const handle_add_node = () => {
        if (!ValidationUtils.is_valid_name(node_form.name)) {
            event_bus.emit_log({ source: 'System', text: 'Invalid Model Name: 2-64 characters required.', severity: 'warning' });
            return;
        }
        if (!node_form.provider) return;

        const final_modality = node_form.is_custom_modality ? node_form.custom_modality : node_form.modality;
        
        // Final numeric validation before store dispatch
        const limits = {
            rpm: Math.max(0, node_form.rpm),
            tpm: Math.max(0, node_form.tpm),
            rpd: Math.max(0, node_form.rpd),
            tpd: Math.max(0, node_form.tpd)
        };

        add_model(node_form.name, node_form.provider, final_modality as Model_Entry['modality'], limits);
        set_node_form({ 
            is_adding: false, 
            name: '', 
            provider: '', 
            modality: 'llm', 
            rpm: 10, 
            tpm: 100000, 
            rpd: 1000, 
            tpd: 10000000,
            is_custom_modality: false,
            custom_modality: ''
        });
    };

    const handle_edit_node = (id: string, name: string, prov: string, modality: Model_Entry['modality'], limits: Record<string, number>) => {
        if (!ValidationUtils.is_valid_name(name)) {
            event_bus.emit_log({ source: 'System', text: 'Invalid Model Name: 2-64 characters required.', severity: 'warning' });
            return;
        }
        edit_model(id, name, prov, modality, limits);
        set_editing_id(null);
    };

    const handle_delete_confirm = async () => {
        if (!confirm_delete) return;
        try {
            if (confirm_delete.type === 'provider') {
                await delete_provider(confirm_delete.id);
            } else {
                await delete_model(confirm_delete.id);
            }
        } catch (err: unknown) {
            alert(`${i18n.t('model_manager.dialogs.change_failed')}: ${err instanceof Error ? err.message : String(err)}`);
        } finally {
            set_confirm_delete(null);
        }
    };

    const handle_reset_vault = () => {
        reset_vault();
        set_show_reset_confirm(false);
    };

    return {
        // State
        models,
        filtered_models,
        selected_provider,
        selected_provider_id,
        set_selected_provider_id,
        modality_filter,
        set_modality_filter,
        editing_id,
        set_editing_id,
        provider_form,
        set_provider_form,
        node_form,
        set_node_form,
        confirm_delete,
        set_confirm_delete,
        show_reset_confirm,
        set_show_reset_confirm,
        error,
        password_input,
        set_password_input,
        is_secure,

        // Handlers
        handle_unlock,
        handle_add_provider,
        handle_add_node,
        handle_edit_node,
        handle_delete_confirm,
        handle_reset_vault,
        lock
    };
}


// Metadata: [use_model_manager]

// Metadata: [use_model_manager]
