/**
 * @docs ARCHITECTURE:UI-Services
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Template_Store / template_store_api
 * - **Primary Entrypoints**: `fetchTemplateRegistry`, `fetchSwarmConfig`, `fetchKnowledgeBase`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { BASE_RAW_URL, REGISTRY_RAW } from './constants';
import type { Template, PlaybookPreview } from './types';

export async function fetchTemplateRegistry(): Promise<Template[]> {
    const controller = typeof AbortController !== 'undefined' ? new AbortController() : null;
    const timer = controller ? setTimeout(() => controller.abort(), 5000) : null;
    try {
        const res = await fetch(REGISTRY_RAW, { signal: controller?.signal });
        if (!res.ok) throw new Error('Failed to load Swarm Template Registry');
        const data = await res.json();
        return (data.templates || []) as Template[];
    } finally {
        if (timer) clearTimeout(timer);
    }
}

export async function fetchSwarmConfig(path: string): Promise<Record<string, unknown>> {
    const configUrl = `${BASE_RAW_URL}/${path}/swarm.json`;
    const controller = typeof AbortController !== 'undefined' ? new AbortController() : null;
    const timer = controller ? setTimeout(() => controller.abort(), 5000) : null;
    try {
        const res = await fetch(configUrl, { signal: controller?.signal });
        if (!res.ok) throw new Error('Failed to fetch swarm configuration');
        return await res.json();
    } finally {
        if (timer) clearTimeout(timer);
    }
}

export async function fetchKnowledgeBase(path: string): Promise<PlaybookPreview[] | null> {
    const knowledgeUrl = `${BASE_RAW_URL}/${path}/knowledge.json`;
    const controller = typeof AbortController !== 'undefined' ? new AbortController() : null;
    const timer = controller ? setTimeout(() => controller.abort(), 5000) : null;
    try {
        const res = await fetch(knowledgeUrl, { signal: controller?.signal });
        if (!res.ok) return null;
        const data = await res.json();
        if (Array.isArray(data)) {
            return data as PlaybookPreview[];
        }
        return null;
    } finally {
        if (timer) clearTimeout(timer);
    }
}

// Metadata: [Template_Store]


// [Template_Store]
