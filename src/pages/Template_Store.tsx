/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **Root View**: Blueprint registry for agent personas and swarm structures. 
 * Orchestrates discovery and deployment of predefined configuration templates.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Template parsing failure (schema mismatch), or missing mandatory field in deployment payload.
 * - **Telemetry Link**: Search for `[Template_Store]` or BLUEPRINT_FETCH in service logs.
 */

import { useCallback } from 'react';
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
    Template_Preview_Modal
} from '../components/template_store';
import type { Template } from '../components/template_store/types';

function Template_Store() {
    const { templates, setTemplates, isLoading, error, refresh } = useTemplateRegistry();
    const filters = useTemplateFilters(templates);
    const preview = useTemplatePreview();

    const handleInstallSuccess = useCallback((templateId: string) => {
        setTemplates((prev) =>
            prev.map((t) => (t.id === templateId ? { ...t, installed: true } : t))
        );
        preview.closePreview();
    }, [setTemplates, preview]);

    const { isInstalling: installingId, install } = useTemplateInstall(handleInstallSuccess);

    const handleImportDownloadedSwarm = useCallback(async (file: File) => {
        const result = await importDownloadedSwarmFile(file);
        if (!result) return;

        const { template: downloadedTemplate, config, playbooks } = result;
        setTemplates((prev) => [downloadedTemplate, ...prev.filter(t => t.id !== downloadedTemplate.id)]);
        void preview.openPreview(downloadedTemplate, config, playbooks);
    }, [setTemplates, preview]);

    return (
        <div className="p-6 space-y-6 max-w-7xl mx-auto w-full">
            <Template_Store_Header />
            <Structured_Data />

            <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4 mb-6">
                <Market_Selector />
                <Repository_Actions isLoading={isLoading} onRefresh={refresh} />
            </div>

            <Template_Filters {...filters} onImportDownloadedSwarm={handleImportDownloadedSwarm} />

            <Template_Grid
                templates={filters.filteredTemplates}
                isLoading={isLoading}
                error={error}
                isInstalling={Boolean(installingId)}
                onPreview={preview.openPreview}
            />

            {preview.previewTemplate && (
                <Template_Preview_Modal
                    template={preview.previewTemplate}
                    config={preview.previewConfig}
                    knowledge={preview.previewKnowledge}
                    isLoading={preview.isPreviewLoading}
                    error={preview.previewError}
                    isInstalling={installingId === preview.previewTemplate.id}
                    onClose={preview.closePreview}
                    onInstall={() => install(
                        preview.previewTemplate!, 
                        preview.previewTemplate?.path === 'local' ? (preview.previewConfig || undefined) : undefined
                    )}
                />
            )}
        </div>
    );
}

export default Template_Store;


// Metadata: [Template_Store]
