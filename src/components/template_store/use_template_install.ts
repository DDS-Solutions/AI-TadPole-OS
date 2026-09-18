/**
 * @docs ARCHITECTURE:Hooks
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Template_Store / use_template_install
 * - **Primary Entrypoints**: `useTemplateInstall`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { useState, useCallback } from 'react';
import { system_api_service } from '../../services/system_api_service';
import { use_workspace_store, type Mission_Cluster } from '../../stores/workspace_store';
import { REPO_URL, APP_REFRESH_AGENTS_EVENT } from './constants';
import type { Template, LocalSwarmAssets, InstallOptions, McpPlaceholderVariable } from './types';
import { extractMcpPlaceholders } from './mcp_placeholders';

export function useTemplateInstall(
    onSuccess?: (templateId: string) => void,
    onMcpSecretsRequired?: (placeholders: McpPlaceholderVariable[]) => void
) {
    const [isInstalling, setIsInstalling] = useState<string | null>(null);
    const [pendingMcpSecrets, setPendingMcpSecrets] = useState<McpPlaceholderVariable[]>([]);

    const install = useCallback(async (
        template: Template,
        localAssets?: LocalSwarmAssets | null,
        options?: InstallOptions | null
    ) => {
        try {
            setIsInstalling(template.id);

            let modelOverride: { provider: string; model_id: string; base_url?: string } | undefined;
            const modelMapping = options?.modelMapping;
            if (modelMapping && modelMapping.strategy !== 'template') {
                if (modelMapping.provider && modelMapping.modelId) {
                    modelOverride = {
                        provider: modelMapping.provider,
                        model_id: modelMapping.modelId,
                        base_url: modelMapping.baseUrl
                    };
                }
            }

            const overwrite = options?.overwrite ?? true;
            const namespace = options?.namespace;

            if (template.path === 'local' && localAssets) {
                // 1. Native direct local import without remote git clone
                await system_api_service.engine.import_template({
                    swarm: localAssets.config || {
                        id: template.id,
                        name: template.name,
                        description: template.description,
                        industry: template.industry
                    },
                    agents: localAssets.agents || [],
                    workflows: localAssets.workflows || [],
                    mcps: localAssets.mcps,
                    overwrite,
                    namespace,
                    model_override: modelOverride
                });
            } else {
                // 2. Remote git-based registry installation
                await system_api_service.engine.install_template(
                    REPO_URL,
                    template.path,
                    modelOverride,
                    overwrite,
                    namespace
                );
            }

            // 3. Register Mission Cluster in the Cluster Manager
            try {
                const config = localAssets?.config;
                let alphaId = '1';
                let collaboratorIds: string[] = [];
                const clusterWorkflows: string[] = [];

                if (localAssets?.agents && localAssets.agents.length > 0) {
                    collaboratorIds = localAssets.agents.map(a => {
                        const content = a.content;
                        const originalId = (typeof content.id === 'string' ? content.id : a.filename.replace(/\.json$/, ''));
                        return namespace ? `${namespace}__${originalId}` : originalId;
                    });
                    if (collaboratorIds.length > 0) {
                        alphaId = collaboratorIds[0];
                    }
                } else if (Array.isArray(config?.roster)) {
                    for (const member of config.roster as Array<Record<string, unknown>>) {
                        if (typeof member.id === 'string') {
                            const mId = namespace ? `${namespace}__${member.id}` : member.id;
                            collaboratorIds.push(mId);
                            if (member.priority === 'critical' || member.supervisor === null) {
                                alphaId = mId;
                            }
                        }
                    }
                }

                if (localAssets?.workflows) {
                    for (const wf of localAssets.workflows) {
                        const baseName = wf.filename.replace(/\.md$/, '');
                        clusterWorkflows.push(namespace ? `${namespace}__${baseName}` : baseName);
                    }
                }

                const department = (template.industry || 'Operations') as Mission_Cluster['department'];

                await use_workspace_store.getState().create_cluster({
                    name: `${template.name} Cluster`,
                    objective: template.description,
                    department,
                    alpha_id: alphaId,
                    collaborators: collaboratorIds,
                    theme: 'cyan'
                });
            } catch (clusterErr) {
                console.warn('[useTemplateInstall] Cluster creation fallback:', clusterErr);
            }

            // 4. Dispatch global refresh event to hot-reload agents in UI
            window.dispatchEvent(new Event(APP_REFRESH_AGENTS_EVENT));

            // 5. Check for unpopulated MCP environment variables
            const placeholders = extractMcpPlaceholders(localAssets?.mcps);
            if (placeholders.length > 0) {
                setPendingMcpSecrets(placeholders);
                if (onMcpSecretsRequired) {
                    onMcpSecretsRequired(placeholders);
                }
            }

            if (onSuccess) {
                onSuccess(template.id);
            }
            return true;
        } catch (err) {
            const msg = err instanceof Error ? err.message : String(err);
            window.alert(`Error installing template: ${msg}`);
            return false;
        } finally {
            setIsInstalling(null);
        }
    }, [onSuccess, onMcpSecretsRequired]);

    return {
        isInstalling,
        pendingMcpSecrets,
        setPendingMcpSecrets,
        install
    };
}

// Metadata: [Template_Store]
