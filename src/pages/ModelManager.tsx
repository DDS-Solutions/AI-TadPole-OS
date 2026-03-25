import { useEffect } from 'react';
import { useVaultStore } from '../stores/vaultStore';
import { useProviderStore } from '../stores/providerStore';
import { useHeaderStore } from '../stores/headerStore';
import { ModelHeaderActions } from '../components/model/ModelHeaderActions';
import ProviderConfigPanel from '../components/ProviderConfigPanel';
import { ConfirmDialog } from '../components/ui/ConfirmDialog';
import { i18n } from '../i18n';
import { useModelManager } from '../components/model/useModelManager';
import {
    VaultLockScreen,
    ProviderGrid,
    ModelInventoryTable,
    AddProviderDialog,
    AddNodeDialog
} from '../components/model';

export default function ModelManager(): React.ReactElement {
    const { isLocked, lock } = useVaultStore();
    const { providers } = useProviderStore();
    const setHeaderActions = useHeaderStore(s => s.setActions);
    const clearHeaderActions = useHeaderStore(s => s.clearActions);

    const {
        models,
        filteredModels,
        selectedProvider,
        selectedProviderId,
        setSelectedProviderId,
        modalityFilter,
        setModalityFilter,
        editingId,
        setEditingId,
        isAddingProvider,
        setIsAddingProvider,
        newProvider,
        setNewProvider,
        isAddingNode,
        setIsAddingNode,
        newNode,
        setNewNode,
        isCustomModality,
        setIsCustomModality,
        customModality,
        setCustomModality,
        confirmDelete,
        setConfirmDelete,
        showResetConfirm,
        setShowResetConfirm,
        error,
        handleUnlock,
        handleAddProvider,
        handleAddNode,
        handleEditNode,
        handleDeleteConfirm,
        handleResetVault,
        passwordInput,
        setPasswordInput,
        isSecure
    } = useModelManager();

    useEffect(() => {
        if (!isLocked) {
            setHeaderActions(
                <ModelHeaderActions
                    onLock={lock}
                    providersCount={providers.length}
                />
            );
        } else {
            clearHeaderActions();
        }
        return () => clearHeaderActions();
    }, [isLocked, providers.length, lock, setHeaderActions, clearHeaderActions]);

    if (isLocked) {
        return (
            <VaultLockScreen
                passwordInput={passwordInput}
                onPasswordChange={setPasswordInput}
                onUnlock={handleUnlock}
                error={error}
                isSecure={isSecure}
                showResetConfirm={showResetConfirm}
                onSetShowResetConfirm={setShowResetConfirm}
                onResetVault={handleResetVault}
            />
        );
    }

    return (
        <div className="h-full flex flex-col bg-zinc-950 relative overflow-hidden">
            <div className="neural-grid opacity-5 pointer-events-none" />

            <div className="flex-1 overflow-auto custom-scrollbar relative">
                <div className="max-w-7xl mx-auto p-8 space-y-12">
                    <ProviderGrid
                        providers={providers}
                        models={models}
                        selectedProviderId={selectedProviderId}
                        onSelectProvider={setSelectedProviderId}
                        onDeleteProvider={(id, name) => setConfirmDelete({ type: 'provider', id, name })}
                        onAddProvider={() => setIsAddingProvider(true)}
                        isAddingProvider={isAddingProvider}
                    />

                    {isAddingProvider && (
                        <AddProviderDialog
                            name={newProvider.name}
                            icon={newProvider.icon}
                            onNameChange={(name) => setNewProvider({ ...newProvider, name })}
                            onIconChange={(icon) => setNewProvider({ ...newProvider, icon })}
                            onConfirm={handleAddProvider}
                            onCancel={() => setIsAddingProvider(false)}
                            error={error}
                        />
                    )}

                    <ModelInventoryTable
                        models={filteredModels}
                        modalityFilter={modalityFilter}
                        onSetModalityFilter={setModalityFilter}
                        onAddNode={() => {
                            setIsAddingNode(!isAddingNode);
                            if (providers.length > 0) setNewNode(prev => ({ ...prev, provider: providers[0].id }));
                        }}
                        editingId={editingId}
                        onEditNode={setEditingId}
                        onSaveNode={handleEditNode}
                        onDeleteNode={(id, name) => setConfirmDelete({ type: 'model', id, name })}
                        providers={providers}
                    >
                        {isAddingNode && (
                            <AddNodeDialog
                                newNode={newNode}
                                onUpdateNewNode={(updates) => setNewNode({ ...newNode, ...updates })}
                                onConfirm={handleAddNode}
                                onCancel={() => setIsAddingNode(false)}
                                isCustomModality={isCustomModality}
                                onSetIsCustomModality={setIsCustomModality}
                                customModality={customModality}
                                onSetCustomModality={setCustomModality}
                                providers={providers}
                            />
                        )}
                    </ModelInventoryTable>
                </div>
            </div>

            {selectedProvider && (
                <ProviderConfigPanel
                    provider={selectedProvider}
                    onClose={() => setSelectedProviderId(null)}
                />
            )}

            <ConfirmDialog
                isOpen={!!confirmDelete}
                title={confirmDelete?.type === 'provider' ? i18n.t('model_manager.dialogs.terminate_provider_title') : i18n.t('model_manager.dialogs.decommission_node_title')}
                message={confirmDelete?.type === 'provider'
                    ? i18n.t('model_manager.dialogs.terminate_provider_desc', { name: confirmDelete?.name || '' })
                    : i18n.t('model_manager.dialogs.decommission_node_desc', { name: confirmDelete?.name || '' })
                }
                confirmLabel={confirmDelete?.type === 'provider' ? i18n.t('model_manager.dialogs.terminate_provider_btn') : i18n.t('model_manager.dialogs.decommission_node_btn')}
                onConfirm={handleDeleteConfirm}
                onCancel={() => setConfirmDelete(null)}
                variant="danger"
            />
        </div>
    );
}
