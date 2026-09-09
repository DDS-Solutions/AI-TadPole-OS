/**
 * @docs ARCHITECTURE:Domain
 *
 * ### AI Context Alignment
 * - **Subsystem**: System Core / domain
 * - **Primary Entrypoints**: `Role`
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
