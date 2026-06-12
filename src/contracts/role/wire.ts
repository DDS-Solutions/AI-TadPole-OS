/**
 * @docs ARCHITECTURE:Contracts
 * 
 * ### AI Assist Note
 * **Wire Layer**: Backend-authoritative DTO for Role Blueprints.
 * Standardizes the "over-the-wire" shape, handling JSON-stringified arrays 
 * from the database layer.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Improper parsing of JSON-stringified `skills` or `workflows` during frontend hydration.
 * - **Telemetry Link**: Not tracked (Static Wire DTO).
 */

import type { Department } from '../agent/shared';
export interface Role_Blueprint_Dto {
    id: string;
    name: string;
    department: Department;
    description: string;
    skills: string; // JSON string
    workflows: string; // JSON string
    mcp_tools: string; // JSON string
    requiresOversight: boolean;
    modelId?: string;
    createdAt?: string;
}

// Metadata: [wire]

// Metadata: [wire]
