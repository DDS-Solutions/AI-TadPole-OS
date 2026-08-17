/**
 * @docs ARCHITECTURE:Types
 * 
 * ### AI Assist Note
 * **Types and Interfaces**: Shared types for Swarm Template Store models.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: N/A
 * - **Telemetry Link**: Search `[Template_Store]` in telemetry traces.
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

// Metadata: [Template_Store]



// [Template_Store]
