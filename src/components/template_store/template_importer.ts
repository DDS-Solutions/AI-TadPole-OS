/**
 * @docs ARCHITECTURE:TemplateStore:Importer
 * 
 * ### AI Assist Note
 * **Swarm Archive Importer**: Deterministic parser for local JSON and ZIP (.zip, .tadpole) swarm archives.
 * Validates blueprint payloads, extracts embedded playbooks, and normalizes template metadata.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Malformed JSON schema, corrupt ZIP file, or missing manifest inside archive.
 * - **Telemetry Link**: Search for `[Template_Importer]` in UI logs.
 */

import JSZip from 'jszip';
import type { Template, PlaybookPreview } from './types';
import { use_notification_store } from '../../stores/notification_store';

export interface ImportedSwarmResult {
    template: Template;
    config: Record<string, unknown>;
    playbooks?: PlaybookPreview[];
}

/**
 * importDownloadedSwarmFile
 * Reads and unpacks a local .json, .zip, or .tadpole swarm archive file.
 */
export async function importDownloadedSwarmFile(file: File): Promise<ImportedSwarmResult | null> {
    try {
        let parsedConfig: Record<string, unknown> | null = null;
        let playbooks: PlaybookPreview[] | null = null;

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
            }
        } catch (err) {
            console.debug('[TemplateImporter] Direct JSON parse failed, trying archive:', err);
            // Not direct JSON text
        }

        // Attempt ZIP archive extraction if direct JSON parse was not successful
        if (!parsedConfig) {
            try {
                const zip = await JSZip.loadAsync(file);

                const configEntry = zip.file(/(?:^|\/)(?:swarm|blueprint|config|template|manifest)\.json$/i)[0]
                    || zip.file(/\.json$/i)[0];

                if (configEntry) {
                    const configText = await configEntry.async('string');
                    parsedConfig = JSON.parse(configText) as Record<string, unknown>;
                }

                const playbookFiles = zip.file(/\.(md|json)$/i).filter(f => f.name !== configEntry?.name);
                if (playbookFiles.length > 0) {
                    const loadedPlaybooks: PlaybookPreview[] = [];
                    for (const pf of playbookFiles) {
                        const content = await pf.async('string');
                        const name = pf.name.split('/').pop() || pf.name;
                        loadedPlaybooks.push({
                            title: name,
                            content
                        });
                    }
                    playbooks = loadedPlaybooks;
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
            playbooks: playbooks || undefined
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
