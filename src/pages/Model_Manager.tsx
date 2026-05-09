/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **Root View**: Intelligence Forge command center. 
 * Orchestrates provider registration, model inventory management, and API vault synchronization.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: API key decryption failure, or inventory fetch timeout from the `model_store`.
 * - **Telemetry Link**: Search for `[Model_Manager]` or `INVENTORY_SYNC` in UI logs.
 */

import { use_vault_store } from '../stores/vault_store';
import { use_provider_store } from '../stores/provider_store';
import Provider_Config_Panel from '../components/Provider_Config_Panel';
import { Confirm_Dialog } from '../components/ui/Confirm_Dialog';
import { i18n } from '../i18n';
import { useModelManager } from '../components/model/use_model_manager';
import {
    Vault_Lock_Screen,
    Provider_Grid,
    Model_Inventory_Table,
    Add_Provider_Dialog,
    Add_Node_Dialog
} from '../components/model';

export default function Model_Manager(): React.ReactElement {
    const { is_locked } = use_vault_store();
    const { providers } = use_provider_store();

    const {
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
        handle_unlock,
        handle_add_provider,
        handle_add_node,
        handle_edit_node,
        handle_delete_confirm,
        handle_reset_vault,
        password_input,
        set_password_input,
        is_secure
    } = useModelManager();



    if (is_locked) {
        return (
            <Vault_Lock_Screen
                password_input={password_input}
                on_password_change={set_password_input}
                on_unlock={handle_unlock}
                error={error}
                is_secure={is_secure}
                show_reset_confirm={show_reset_confirm}
                on_set_show_reset_confirm={set_show_reset_confirm}
                on_reset_vault={handle_reset_vault}
            />
        );
    }

    return (
        <div className="h-full flex flex-col bg-zinc-950 relative overflow-hidden">
            {/* GEO Optimization: Structured Data & Semantic Header */}
            <script type="application/ld+json">
            {JSON.stringify({
              "@context": "https://schema.org",
              "@type": "SoftwareApplication",
              "name": "Tadpole OS Model Manager",
              "description": "Enterprise-grade intelligence forge for managing AI providers, model inventories, and secure vault synchronization.",
              "author": { "@type": "Organization", "name": "Sovereign Engineering" },
              "applicationCategory": "Development Environment",
              "operatingSystem": "Tadpole OS"
            })}
            </script>
            <h1 className="sr-only">Tadpole OS Intelligence Forge: Model & Provider Management</h1>
            <div className="neural-grid opacity-5 pointer-events-none" />

            <div className="flex-1 overflow-auto custom-scrollbar relative">
                <div className="max-w-7xl mx-auto p-8 space-y-12">
                    <Provider_Grid
                        providers={providers}
                        models={models}
                        selected_provider_id={selected_provider_id}
                        on_select_provider={set_selected_provider_id}
                        on_delete_provider={(id, name) => set_confirm_delete({ type: 'provider', id, name })}
                        on_add_provider={() => set_provider_form({ ...provider_form, is_adding: true })}
                        is_adding_provider={provider_form.is_adding}
                    />

                    {provider_form.is_adding && (
                        <Add_Provider_Dialog
                            name={provider_form.name}
                            icon={provider_form.icon}
                            on_name_change={(name) => set_provider_form({ ...provider_form, name })}
                            on_icon_change={(icon) => set_provider_form({ ...provider_form, icon })}
                            on_confirm={handle_add_provider}
                            on_cancel={() => set_provider_form({ ...provider_form, is_adding: false })}
                            error={error}
                        />
                    )}

                    <Model_Inventory_Table
                        models={filtered_models}
                        modality_filter={modality_filter}
                        on_set_modality_filter={set_modality_filter}
                        on_add_node={() => {
                            set_node_form((prev) => ({ ...prev, is_adding: !prev.is_adding, provider: providers.length > 0 ? providers[0].id : '' }));
                        }}
                        editing_id={editing_id}
                        on_edit_node={set_editing_id}
                        on_save_node={handle_edit_node}
                        on_delete_node={(id, name) => set_confirm_delete({ type: 'model', id, name })}
                        providers={providers}
                    >
                        {node_form.is_adding && (
                            <Add_Node_Dialog
                                new_node={node_form}
                                on_update_new_node={(updates) => set_node_form({ ...node_form, ...updates })}
                                on_confirm={handle_add_node}
                                on_cancel={() => set_node_form({ ...node_form, is_adding: false })}
                                is_custom_modality={node_form.is_custom_modality}
                                on_set_is_custom_modality={(val) => set_node_form({ ...node_form, is_custom_modality: val })}
                                custom_modality={node_form.custom_modality}
                                on_set_custom_modality={(val) => set_node_form({ ...node_form, custom_modality: val })}
                                providers={providers}
                            />
                        )}
                    </Model_Inventory_Table>
                </div>
            </div>

            {selected_provider && (
                <Provider_Config_Panel
                    provider={selected_provider}
                    on_close={() => set_selected_provider_id(null)}
                />
            )}

            <Confirm_Dialog
                is_open={!!confirm_delete}
                title={confirm_delete?.type === 'provider' ? i18n.t('model_manager.dialogs.terminate_provider_title') : i18n.t('model_manager.dialogs.decommission_node_title')}
                message={confirm_delete?.type === 'provider'
                    ? i18n.t('model_manager.dialogs.terminate_provider_desc', { name: confirm_delete?.name || '' })
                    : i18n.t('model_manager.dialogs.decommission_node_desc', { name: confirm_delete?.name || '' })
                }
                confirm_label={confirm_delete?.type === 'provider' ? i18n.t('model_manager.dialogs.terminate_provider_btn') : i18n.t('model_manager.dialogs.decommission_node_btn')}
                on_confirm={handle_delete_confirm}
                on_cancel={() => set_confirm_delete(null)}
                variant="danger"
            />
        </div>
    );
}


// Metadata: [Model_Manager]
