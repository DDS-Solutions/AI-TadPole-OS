/**
 * @docs ARCHITECTURE:Hooks
 * 
 * ### AI Assist Note
 * **Custom Hook**: Encapsulates preview state management and parallel asset loading.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Gracefully handles missing/corrupt swarm configs or playbooks without global crash.
 * - **Telemetry Link**: Search `[Template_Store]` in network traces.
 */

import { useState, useCallback } from 'react';
import type { Template, PlaybookPreview } from './types';
import { fetchSwarmConfig, fetchKnowledgeBase } from './template_store_api';

export function useTemplatePreview() {
    const [previewTemplate, setPreviewTemplate] = useState<Template | null>(null);
    const [previewConfig, setPreviewConfig] = useState<Record<string, unknown> | null>(null);
    const [previewKnowledge, setPreviewKnowledge] = useState<PlaybookPreview[] | null>(null);
    const [isPreviewLoading, setIsPreviewLoading] = useState(false);
    const [previewError, setPreviewError] = useState<string | null>(null);

    const openPreview = useCallback(async (template: Template) => {
        try {
            setPreviewTemplate(template);
            setIsPreviewLoading(true);
            setPreviewConfig(null);
            setPreviewKnowledge(null);
            setPreviewError(null);

            const [config, knowledge] = await Promise.all([
                fetchSwarmConfig(template.path),
                fetchKnowledgeBase(template.path).catch((kErr) => {
                    console.warn('No knowledge.json found or failed to parse:', kErr);
                    return null;
                })
            ]);

            setPreviewConfig(config);
            setPreviewKnowledge(knowledge);
        } catch (err) {
            console.error('Preview error:', err);
            setPreviewError('Could not load configuration preview.');
        } finally {
            setIsPreviewLoading(false);
        }
    }, []);

    const closePreview = useCallback(() => {
        setPreviewTemplate(null);
        setPreviewConfig(null);
        setPreviewKnowledge(null);
        setPreviewError(null);
    }, []);

    return {
        previewTemplate,
        previewConfig,
        previewKnowledge,
        isPreviewLoading,
        previewError,
        openPreview,
        closePreview
    };
}

// Metadata: [Template_Store]


// [Template_Store]
