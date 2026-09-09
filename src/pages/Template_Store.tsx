/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Pages / Template_Store
 * - **Primary Entrypoints**: `Template_Store`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: `Template_Store.test.tsx`
 */

import { useState, useCallback, useEffect } from 'react';
import { Store, Layers } from 'lucide-react';
import {
    useTemplateRegistry,
    useTemplateFilters,
    useTemplatePreview,
    useTemplateInstall,
    importDownloadedSwarmFile,
    Template_Store_Header,
    Structured_Data,
    Market_Selector,
    Repository_Actions,
    Template_Filters,
    Template_Grid,
    Template_Preview_Modal,
    Installed_Swarms_List,
    Uninstall_Swarm_Modal,
    Mcp_Secrets_Wizard_Modal,
    APP_REFRESH_AGENTS_EVENT
} from '../components/template_store';
import type { InstalledSwarmSummary, McpPlaceholderVariable, InstallOptions } from '../components/template_store/types';
import { system_api_service } from '../services/system_api_service';
import { i18n } from '../i18n';

function Template_Store() {
    const [activeTab, setActiveTab] = useState<'marketplace' | 'installed'>('marketplace');
    const { templates, setTemplates, isLoading, error, refresh } = useTemplateRegistry();
    const filters = useTemplateFilters(templates);
    const {
        previewTemplate,
        previewConfig,
        previewKnowledge,
        previewAssets,
        isPreviewLoading,
        previewError,
        openPreview,
        closePreview
    } = useTemplatePreview();

    // Installed Swarms State
    const [installedSwarms, setInstalledSwarms] = useState<InstalledSwarmSummary[]>([]);
    const [isLoadingInstalled, setIsLoadingInstalled] = useState(false);
    const [installedError, setInstalledError] = useState<string | null>(null);

    // MCP Wizard State
    const [mcpPlaceholders, setMcpPlaceholders] = useState<McpPlaceholderVariable[]>([]);
    const [isMcpWizardOpen, setIsMcpWizardOpen] = useState(false);

    // Uninstall Modal State
    const [swarmToUninstall, setSwarmToUninstall] = useState<InstalledSwarmSummary | null>(null);
    const [isUninstallModalOpen, setIsUninstallModalOpen] = useState(false);
    const [isUninstalling, setIsUninstalling] = useState(false);

    const fetchInstalledSwarms = useCallback(async () => {
        try {
            setIsLoadingInstalled(true);
            setInstalledError(null);
            const res = await system_api_service.engine.get_installed_templates();
            setInstalledSwarms(res?.swarms || []);
        } catch (err) {
            const msg = err instanceof Error ? err.message : String(err);
            console.warn('[Template_Store] Failed to load installed swarms:', msg);
            setInstalledError(msg);
        } finally {
            setIsLoadingInstalled(false);
        }
    }, []);

    useEffect(() => {
        let isMounted = true;
        void (async () => {
            if (isMounted) {
                await fetchInstalledSwarms();
            }
        })();
        return () => {
            isMounted = false;
        };
    }, [fetchInstalledSwarms]);

    const handleMcpSecretsRequired = useCallback((placeholders: McpPlaceholderVariable[]) => {
        if (placeholders.length > 0) {
            setMcpPlaceholders(placeholders);
            setIsMcpWizardOpen(true);
        }
    }, []);

    const handleInstallSuccess = useCallback((templateId: string) => {
        setTemplates((prev) =>
            prev.map((t) => (t.id === templateId ? { ...t, installed: true } : t))
        );
        closePreview();
        fetchInstalledSwarms();
    }, [setTemplates, closePreview, fetchInstalledSwarms]);

    const { isInstalling: installingId, install } = useTemplateInstall(
        handleInstallSuccess,
        handleMcpSecretsRequired
    );

    const handleImportDownloadedSwarm = useCallback(async (file: File) => {
        try {
            const result = await importDownloadedSwarmFile(file);
            if (!result) {
                console.warn(`[Template_Store] Import returned null for file: ${file.name}`);
                return;
            }

            const { template: downloadedTemplate, config, playbooks, assets } = result;
            setTemplates((prev) => [downloadedTemplate, ...prev.filter(t => t.id !== downloadedTemplate.id)]);
            await openPreview(downloadedTemplate, config, playbooks, assets);
        } catch (err) {
            console.error('[Template_Store] Failed to import downloaded swarm file:', err);
        }
    }, [setTemplates, openPreview]);

    const handleOpenUninstallModal = useCallback((swarm: InstalledSwarmSummary) => {
        setSwarmToUninstall(swarm);
        setIsUninstallModalOpen(true);
    }, []);

    const handleCloseUninstallModal = useCallback(() => {
        if (!isUninstalling) {
            setSwarmToUninstall(null);
            setIsUninstallModalOpen(false);
        }
    }, [isUninstalling]);

    const handleConfirmUninstall = useCallback(async (swarmId: string, archive: boolean) => {
        try {
            setIsUninstalling(true);
            await system_api_service.engine.uninstall_template(swarmId, archive);
            
            // Mark uninstalled in template registry if present
            setTemplates((prev) =>
                prev.map((t) => (t.id === swarmId || t.path.includes(swarmId) ? { ...t, installed: false } : t))
            );

            // Refresh installed swarms and dispatch agent refresh
            await fetchInstalledSwarms();
            window.dispatchEvent(new Event(APP_REFRESH_AGENTS_EVENT));

            setIsUninstallModalOpen(false);
            setSwarmToUninstall(null);
        } catch (err) {
            const msg = err instanceof Error ? err.message : String(err);
            window.alert(`Error uninstalling swarm: ${msg}`);
        } finally {
            setIsUninstalling(false);
        }
    }, [setTemplates, fetchInstalledSwarms]);

    const activeTemplate = previewTemplate;

    return (
        <div className="p-6 space-y-6 max-w-7xl mx-auto w-full">
            <Template_Store_Header />
            <Structured_Data />

            {/* Navigation Tabs Bar */}
            <div className="flex items-center gap-3 border-b border-[color:var(--color-border)]/50 pb-4">
                <button
                    type="button"
                    onClick={() => setActiveTab('marketplace')}
                    className={`px-4 py-2 rounded-xl text-xs font-bold flex items-center gap-2 transition-all duration-200 ${
                        activeTab === 'marketplace'
                            ? 'bg-emerald-500/15 text-emerald-400 border border-emerald-500/30 shadow-lg shadow-emerald-950/30'
                            : 'bg-zinc-900/40 text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800/60 border border-zinc-800/60'
                    }`}
                >
                    <Store size={14} className={activeTab === 'marketplace' ? 'text-emerald-400' : 'text-zinc-500'} />
                    <span>{i18n.t('template_store.tab_marketplace') || 'Swarm Marketplace'}</span>
                    <span className={`px-2 py-0.5 rounded text-[10px] font-mono ${
                        activeTab === 'marketplace' ? 'bg-emerald-500/20 text-emerald-300 border border-emerald-500/30' : 'bg-zinc-800/80 text-zinc-500'
                    }`}>
                        {templates.length}
                    </span>
                </button>

                <button
                    type="button"
                    onClick={() => setActiveTab('installed')}
                    className={`px-4 py-2 rounded-xl text-xs font-bold flex items-center gap-2 transition-all duration-200 ${
                        activeTab === 'installed'
                            ? 'bg-emerald-500/15 text-emerald-400 border border-emerald-500/30 shadow-lg shadow-emerald-950/30'
                            : 'bg-zinc-900/40 text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800/60 border border-zinc-800/60'
                    }`}
                >
                    <Layers size={14} className={activeTab === 'installed' ? 'text-emerald-400' : 'text-zinc-500'} />
                    <span>{i18n.t('template_store.tab_installed') || 'Installed Swarms'}</span>
                    <span className={`px-2 py-0.5 rounded text-[10px] font-mono ${
                        activeTab === 'installed' ? 'bg-emerald-500/20 text-emerald-300 border border-emerald-500/30' : 'bg-zinc-800/80 text-zinc-500'
                    }`}>
                        {installedSwarms.length}
                    </span>
                </button>
            </div>

            {/* Tab: Marketplace */}
            {activeTab === 'marketplace' && (
                <div className="space-y-6 animate-in fade-in duration-200">
                    <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4 mb-6">
                        <Market_Selector />
                        <Repository_Actions isLoading={isLoading} onRefresh={refresh} />
                    </div>

                    <Template_Filters {...filters} onImportDownloadedSwarm={handleImportDownloadedSwarm} />

                    <Template_Grid
                        templates={filters.filteredTemplates}
                        isLoading={isLoading}
                        error={error}
                        isInstalling={installingId}
                        onPreview={openPreview}
                    />
                </div>
            )}

            {/* Tab: Installed Swarms */}
            {activeTab === 'installed' && (
                <Installed_Swarms_List
                    swarms={installedSwarms}
                    isLoading={isLoadingInstalled}
                    error={installedError}
                    onUninstallClick={handleOpenUninstallModal}
                    onRefresh={fetchInstalledSwarms}
                    onBrowseMarketplace={() => setActiveTab('marketplace')}
                />
            )}

            {/* Template Preview & Install Modal */}
            {activeTemplate && (
                <Template_Preview_Modal
                    template={activeTemplate}
                    config={previewConfig}
                    knowledge={previewKnowledge}
                    isLoading={isPreviewLoading}
                    error={previewError}
                    isInstalling={installingId === activeTemplate.id}
                    onClose={closePreview}
                    onInstall={(options: InstallOptions) => install(activeTemplate, previewAssets, options)}
                />
            )}

            {/* Post-Install MCP Secrets Wizard Modal */}
            <Mcp_Secrets_Wizard_Modal
                isOpen={isMcpWizardOpen}
                placeholders={mcpPlaceholders}
                onClose={() => setIsMcpWizardOpen(false)}
                onSaveSuccess={() => {
                    console.debug('[Template_Store] MCP credentials saved successfully');
                }}
            />

            {/* Uninstall Swarm Confirmation Modal */}
            <Uninstall_Swarm_Modal
                swarm={swarmToUninstall}
                isOpen={isUninstallModalOpen}
                isUninstalling={isUninstalling}
                onClose={handleCloseUninstallModal}
                onConfirm={handleConfirmUninstall}
            />
        </div>
    );
}

export default Template_Store;
