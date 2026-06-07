/**
 * @docs ARCHITECTURE:Domain
 * 
 * ### AI Assist Note
 * **Role Definition**: Normalized contract for agent personas. 
 * Defines the skills, workflows, and tool access allowed for a specific duty.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Validation errors when importing roles from legacy JSON manifests or ID collisions in the role store.
 * - **Telemetry Link**: Not tracked (Static Domain Model).
 */

import type { Department } from '../agent/shared';
export interface Role {
    id: string;
    name: string;
    department: Department;
    description: string;
    skills: string[];
    workflows: string[];
    mcp_tools: string[];
    requires_oversight: boolean;
    model_id?: string;
    created_at?: string;
}

// Metadata: [domain]

// Metadata: [domain]
