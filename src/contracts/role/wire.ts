/**
 * @docs ARCHITECTURE:Contracts
 *
 * ### AI Context Alignment
 * - **Subsystem**: System Core / wire
 * - **Primary Entrypoints**: `Role_Blueprint_Dto`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
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
