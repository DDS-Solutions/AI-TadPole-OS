import { useState, useMemo } from 'react';
import { useProviderStore } from '../../stores/providerStore';
import { useVaultStore } from '../../stores/vaultStore';
import { useModelStore } from '../../stores/modelStore';
import type { ModelEntry } from '../../stores/providerStore';
import { i18n } from '../../i18n';
import { ValidationUtils } from '../../utils/validationUtils';
import { EventBus } from '../../services/eventBus';

export function useModelManager() {
    const { unlock, lock, resetVault } = useVaultStore();
    const { providers, addProvider, deleteProvider } = useProviderStore();
    const { models, addModel, editModel, deleteModel } = useModelStore();

    const [passwordInput, setPasswordInput] = useState('');
    const [error, setError] = useState<string | null>(null);

    // Navigation & UI States
    const [selectedProviderId, setSelectedProviderId] = useState<string | null>(null);
    const [modalityFilter, setModalityFilter] = useState<'all' | ModelEntry['modality']>('all');
    const [editingId, setEditingId] = useState<string | null>(null);

    // Add states
    const [isAddingProvider, setIsAddingProvider] = useState(false);
    const [newProvider, setNewProvider] = useState({ name: '', icon: '⚡' });
    const [isAddingNode, setIsAddingNode] = useState(false);
    const [newNode, setNewNode] = useState({ 
        name: '', 
        provider: '', 
        modality: 'llm' as ModelEntry['modality'], 
        rpm: 10, 
        tpm: 100000, 
        rpd: 1000, 
        tpd: 10000000 
    });
    const [isCustomModality, setIsCustomModality] = useState(false);
    const [customModality, setCustomModality] = useState('');

    // Confirmation State
    const [confirmDelete, setConfirmDelete] = useState<{
        type: 'provider' | 'model';
        id: string;
        name: string;
    } | null>(null);
    const [showResetConfirm, setShowResetConfirm] = useState(false);

    const isSecure = typeof window !== 'undefined' && (
        window.isSecureContext || 
        window.location.hostname === 'localhost' || 
        window.location.hostname === '127.0.0.1'
    );

    const handleUnlock = async (): Promise<void> => {
        setError(null);
        const result = await unlock(passwordInput);
        if (!result.success) {
            setError(result.error?.toUpperCase() || i18n.t('model_manager.vault.error_invalid'));
        }
        setPasswordInput('');
    };

    const selectedProvider = useMemo(() =>
        providers.find(p => p.id === selectedProviderId),
        [providers, selectedProviderId]
    );

    const filteredModels = useMemo(() => {
        let filtered = [...models];
        if (modalityFilter !== 'all') {
            filtered = filtered.filter(m => m.modality === modalityFilter);
        }
        return filtered.sort((a, b) => a.name.localeCompare(b.name));
    }, [models, modalityFilter]);

    const handleAddProvider = async () => {
        try {
            await addProvider(newProvider.name, newProvider.icon);
            setIsAddingProvider(false);
            setNewProvider({ name: '', icon: '⚡' });
        } catch (e: unknown) {
            setError(e instanceof Error ? e.message : 'Unknown error');
        }
    };

    const handleAddNode = () => {
        if (!ValidationUtils.isValidName(newNode.name)) {
            EventBus.emit({ source: 'System', text: 'Invalid Model Name: 2-64 characters required.', severity: 'warning' });
            return;
        }
        if (!newNode.provider) return;

        const finalModality = isCustomModality ? customModality : newNode.modality;
        
        // Final numeric validation before store dispatch
        const limits = {
            rpm: Math.max(0, newNode.rpm),
            tpm: Math.max(0, newNode.tpm),
            rpd: Math.max(0, newNode.rpd),
            tpd: Math.max(0, newNode.tpd)
        };

        addModel(newNode.name, newNode.provider, finalModality, limits);
        setIsAddingNode(false);
        setNewNode({ name: '', provider: '', modality: 'llm', rpm: 10, tpm: 100000, rpd: 1000, tpd: 10000000 });
        setIsCustomModality(false);
        setCustomModality('');
    };

    const handleEditNode = (id: string, name: string, prov: string, modality: ModelEntry['modality'], limits: Record<string, number>) => {
        if (!ValidationUtils.isValidName(name)) {
            EventBus.emit({ source: 'System', text: 'Invalid Model Name: 2-64 characters required.', severity: 'warning' });
            return;
        }
        editModel(id, name, prov, modality, limits);
        setEditingId(null);
    };

    const handleDeleteConfirm = async () => {
        if (!confirmDelete) return;
        try {
            if (confirmDelete.type === 'provider') {
                await deleteProvider(confirmDelete.id);
            } else {
                await deleteModel(confirmDelete.id);
            }
        } catch (err: unknown) {
            alert(`${i18n.t('model_manager.dialogs.change_failed')}: ${err instanceof Error ? err.message : String(err)}`);
        } finally {
            setConfirmDelete(null);
        }
    };

    const handleResetVault = () => {
        resetVault();
        setShowResetConfirm(false);
    };

    return {
        // State
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
        passwordInput,
        setPasswordInput,
        isSecure,

        // Handlers
        handleUnlock,
        handleAddProvider,
        handleAddNode,
        handleEditNode,
        handleDeleteConfirm,
        handleResetVault,
        lock
    };
}
