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
    const res = await fetch(REGISTRY_RAW);
    if (!res.ok) throw new Error('Failed to load Swarm Template Registry');
    const data = await res.json();
    return (data.templates || []) as Template[];
}

export async function fetchSwarmConfig(path: string): Promise<Record<string, unknown>> {
    const configUrl = `${BASE_RAW_URL}/${path}/swarm.json`;
    const res = await fetch(configUrl);
    if (!res.ok) throw new Error('Failed to fetch swarm configuration');
    return await res.json();
}

export async function fetchKnowledgeBase(path: string): Promise<PlaybookPreview[] | null> {
    const knowledgeUrl = `${BASE_RAW_URL}/${path}/knowledge.json`;
    const res = await fetch(knowledgeUrl);
    if (!res.ok) return null;
    const data = await res.json();
    if (Array.isArray(data)) {
        return data as PlaybookPreview[];
    }
    return null;
}

// Metadata: [Template_Store]


// [Template_Store]
