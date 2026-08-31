/**
 * @docs ARCHITECTURE:Types
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Template_Store / types
 * - **Primary Entrypoints**: `PlaybookPreview`, `Template`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

export interface PlaybookPreview {
    title?: string;
    description?: string;
    content?: string;
    topic?: string;
    concept_type?: string;
    resource_uri?: string;
    tags?: string;
}

export interface Template {
    id: string;
    name: string;
    description: string;
    industry: string;
    company_size?: number;
    tags: string[];
    path: string;
    author?: string;
    updatedAt?: string;
    stars?: number;
    installed?: boolean;
}

export interface RawAgentAsset {
    filename: string;
    content: Record<string, unknown>;
}

export interface RawWorkflowAsset {
    filename: string;
    content: string;
}

export type ModelMappingStrategy = 'system' | 'template' | 'ollama' | 'custom';

export interface ModelMappingSelection {
    strategy: ModelMappingStrategy;
    provider?: string;
    modelId?: string;
    baseUrl?: string;
}

export interface LocalSwarmAssets {
    config?: Record<string, unknown>;
    agents?: RawAgentAsset[];
    workflows?: RawWorkflowAsset[];
    mcps?: Record<string, unknown>;
}

export interface ImportedSwarmResult {
    template: Template;
    config: Record<string, unknown>;
    playbooks?: PlaybookPreview[];
    assets?: LocalSwarmAssets;
}

export interface InstallOptions {
    modelMapping?: ModelMappingSelection | null;
    overwrite?: boolean;
    namespace?: string;
}

export interface McpPlaceholderVariable {
    server: string;
    variable: string;
    description?: string;
}

export type { InstalledSwarmSummary, UninstallTemplateResponse } from '../../services/system/engine_api';

// Metadata: [Template_Store]


