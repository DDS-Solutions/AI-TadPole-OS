/**
 * @docs ARCHITECTURE:Hooks
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Template_Store / use_template_preview
 * - **Primary Entrypoints**: `useTemplatePreview`
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
import type { Template, PlaybookPreview, LocalSwarmAssets } from './types';
import { fetchSwarmConfig, fetchKnowledgeBase } from './template_store_api';

export function useTemplatePreview() {
    const [previewTemplate, setPreviewTemplate] = useState<Template | null>(null);
    const [previewConfig, setPreviewConfig] = useState<Record<string, unknown> | null>(null);
    const [previewKnowledge, setPreviewKnowledge] = useState<PlaybookPreview[] | null>(null);
    const [previewAssets, setPreviewAssets] = useState<LocalSwarmAssets | null>(null);
    const [isPreviewLoading, setIsPreviewLoading] = useState(false);
    const [previewError, setPreviewError] = useState<string | null>(null);

    const openPreview = useCallback(async (
        template: Template, 
        customConfig?: Record<string, unknown>, 
        customKnowledge?: PlaybookPreview[],
        customAssets?: LocalSwarmAssets
    ) => {
        try {
            setPreviewTemplate(template);
            setIsPreviewLoading(true);
            setPreviewConfig(customConfig || null);
            setPreviewKnowledge(customKnowledge || null);
            setPreviewAssets(customAssets || null);
            setPreviewError(null);

            if (!customConfig && template.path !== 'local') {
                const [config, knowledge] = await Promise.all([
                    fetchSwarmConfig(template.path),
                    fetchKnowledgeBase(template.path).catch((kErr) => {
                        console.warn('No knowledge.json found or failed to parse:', kErr);
                        return null;
                    })
                ]);

                setPreviewConfig(config);
                setPreviewKnowledge(knowledge);
            }
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
        setPreviewAssets(null);
        setPreviewError(null);
    }, []);

    return {
        previewTemplate,
        previewConfig,
        previewKnowledge,
        previewAssets,
        isPreviewLoading,
        previewError,
        openPreview,
        closePreview
    };
}

// Metadata: [Template_Store]
