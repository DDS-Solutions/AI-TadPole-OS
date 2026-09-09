/**
 * @docs ARCHITECTURE:TemplateStore:Importer
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Template_Store / template_importer
 * - **Primary Entrypoints**: `importDownloadedSwarmFile`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: `[TemplateImporter]`, `[Template_Importer]`
 * - **Witness Tests**: none declared
 */

import JSZip from 'jszip';
import type { Template, PlaybookPreview, RawAgentAsset, RawWorkflowAsset, ImportedSwarmResult } from './types';
import { use_notification_store } from '../../stores/notification_store';

export type { ImportedSwarmResult };

/**
 * importDownloadedSwarmFile
 * Reads and unpacks a local .json, .zip, or .tadpole swarm archive file.
 */
export async function importDownloadedSwarmFile(file: File): Promise<ImportedSwarmResult | null> {
    try {
        let parsedConfig: Record<string, unknown> | null = null;
        let playbooks: PlaybookPreview[] | null = null;
        const rawAgents: RawAgentAsset[] = [];
        const rawWorkflows: RawWorkflowAsset[] = [];
        let rawMcps: Record<string, unknown> | undefined = undefined;

        // Attempt direct JSON parse first
        try {
            let text = '';
            if (typeof file.text === 'function') {
                try {
                    text = await file.text();
                } catch (err) {
                    console.debug('[TemplateImporter] file.text() failed, falling back to FileReader:', err);
                    text = '';
                }
            }
            if (!text) {
                text = await new Promise<string>((resolve, reject) => {
                    const reader = new FileReader();
                    reader.onload = () => resolve((reader.result as string) || '');
                    reader.onerror = () => reject(new Error('FileReader error'));
                    reader.readAsText(file);
                });
            }
            if (text && text.trim().startsWith('{')) {
                parsedConfig = JSON.parse(text) as Record<string, unknown>;

                // Handle single-file JSON bundle that embeds agents/playbooks
                if (Array.isArray(parsedConfig.agents)) {
                    for (let idx = 0; idx < parsedConfig.agents.length; idx++) {
                        const agentObj = parsedConfig.agents[idx] as Record<string, unknown>;
                        const filename = (typeof agentObj.id === 'string' ? agentObj.id : `agent-${idx + 1}`) + '.json';
                        rawAgents.push({ filename, content: agentObj });
                    }
                }
                if (Array.isArray(parsedConfig.playbooks)) {
                    for (const pb of parsedConfig.playbooks as Array<Record<string, unknown>>) {
                        const title = typeof pb.title === 'string' ? pb.title : 'Playbook';
                        const content = typeof pb.content === 'string' ? pb.content : '';
                        const filename = title.toLowerCase().replace(/[^a-z0-9]+/g, '_') + '.md';
                        rawWorkflows.push({ filename, content });
                    }
                }
                if (parsedConfig.mcp_servers && typeof parsedConfig.mcp_servers === 'object') {
                    rawMcps = { mcp_servers: parsedConfig.mcp_servers };
                }
            }
        } catch (err) {
            console.debug('[TemplateImporter] Direct JSON parse failed, trying archive:', err);
            // Not direct JSON text
        }

        // Attempt ZIP archive extraction if direct JSON parse was not successful or if it's a zip
        if (!parsedConfig || rawAgents.length === 0) {
            try {
                const zip = await JSZip.loadAsync(file);

                const configEntry = zip.file(/(?:^|\/)(?:swarm|blueprint|config|template|manifest)\.json$/i)[0]
                    || zip.file(/\.json$/i)[0];

                if (configEntry) {
                    const configText = await configEntry.async('string');
                    parsedConfig = JSON.parse(configText) as Record<string, unknown>;
                }

                // Extract all agent JSONs from agents/ or root
                const agentFiles = zip.file(/(?:^|\/)agents\/[^/]+\.json$/i);
                for (const af of agentFiles) {
                    const contentStr = await af.async('string');
                    try {
                        const agentContent = JSON.parse(contentStr) as Record<string, unknown>;
                        const filename = af.name.split('/').pop() || af.name;
                        rawAgents.push({ filename, content: agentContent });
                    } catch {
                        // ignore malformed agent file
                    }
                }

                // Extract all workflow playbooks (.md files)
                const workflowFiles = zip.file(/(?:^|\/)workflows\/[^/]+\.md$/i);
                const generalMdFiles = workflowFiles.length > 0 ? workflowFiles : zip.file(/\.md$/i);

                const loadedPlaybooks: PlaybookPreview[] = [];
                for (const wf of generalMdFiles) {
                    const content = await wf.async('string');
                    const name = wf.name.split('/').pop() || wf.name;
                    rawWorkflows.push({ filename: name, content });
                    loadedPlaybooks.push({
                        title: name.replace(/\.md$/, ''),
                        content
                    });
                }
                if (loadedPlaybooks.length > 0) {
                    playbooks = loadedPlaybooks;
                }

                // Extract mcps.json if present
                const mcpEntry = zip.file(/(?:^|\/)mcps\.json$/i)[0];
                if (mcpEntry) {
                    const mcpText = await mcpEntry.async('string');
                    try {
                        rawMcps = JSON.parse(mcpText) as Record<string, unknown>;
                    } catch {
                        // ignore malformed mcps.json
                    }
                }
            } catch (zipErr) {
                console.warn('[Template_Importer] ZIP extraction fallback failed:', zipErr);
            }
        }

        if (!parsedConfig || typeof parsedConfig !== 'object') {
            use_notification_store.getState().add_notification({
                severity: 'error',
                title: 'Invalid Swarm File',
                message: 'Please select a valid JSON blueprint or Swarm Archive (.zip, .tadpole) file.',
                persistent: false
            });
            return null;
        }

        const localId = typeof parsedConfig.id === 'string' ? parsedConfig.id
            : typeof parsedConfig.swarm_id === 'string' ? parsedConfig.swarm_id
            : `downloaded-${Date.now()}`;

        const localName = typeof parsedConfig.name === 'string' ? parsedConfig.name
            : typeof parsedConfig.title === 'string' ? parsedConfig.title
            : file.name.replace(/\.[^/.]+$/, '');

        const localDesc = typeof parsedConfig.description === 'string' ? parsedConfig.description
            : typeof parsedConfig.goal === 'string' ? parsedConfig.goal
            : 'Downloaded Swarm Archive imported from local machine.';

        const localIndustry = typeof parsedConfig.industry === 'string' ? parsedConfig.industry
            : typeof parsedConfig.category === 'string' ? parsedConfig.category
            : 'Downloaded Swarm';

        const localSize = typeof parsedConfig.company_size === 'number' ? parsedConfig.company_size
            : typeof parsedConfig.default_seats === 'number' ? parsedConfig.default_seats
            : 50;

        const localTags = Array.isArray(parsedConfig.tags)
            ? (parsedConfig.tags.filter((t): t is string => typeof t === 'string'))
            : ['Downloaded', 'Local Archive'];

        const downloadedTemplate: Template = {
            id: localId,
            name: localName,
            description: localDesc,
            industry: localIndustry,
            company_size: localSize,
            tags: localTags,
            path: 'local',
            author: typeof parsedConfig.author === 'string' ? parsedConfig.author : 'Local File',
            updatedAt: new Date().toISOString().split('T')[0],
            stars: 999,
            installed: false
        };

        return {
            template: downloadedTemplate,
            config: parsedConfig,
            playbooks: playbooks || undefined,
            assets: {
                config: parsedConfig,
                agents: rawAgents,
                workflows: rawWorkflows,
                mcps: rawMcps
            }
        };
    } catch (err) {
        console.error('[Template_Importer] Failed to import swarm file:', err);
        use_notification_store.getState().add_notification({
            severity: 'error',
            title: 'Import Failed',
            message: 'Failed to read or unpack the selected swarm archive.',
            persistent: false
        });
        return null;
    }
}
