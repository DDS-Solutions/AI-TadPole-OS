/**
 * @docs ARCHITECTURE:Core
 *
 * ### AI Context Alignment
 * - **Subsystem**: Domain Model / Agent Hierarchy
 * - **Primary Entrypoints**: `resolve_alpha`, `resolve_nexus`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic leadership resolution across all UI consumers (Org_Chart, SovereignChat).
 * - `[Structural]` Never returns undefined if at least one agent is available in the provided list.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

export interface HierarchyAgentLike {
    id: string;
    name: string;
    role?: string;
}

/**
 * Resolves the primary root Alpha (CEO / Lead Agent).
 * Precedence:
 * 1. Role is 'CEO' (case-insensitive) or name includes 'Agent of Nine'
 * 2. Agent with ID '1'
 * 3. First agent in list
 */
export function resolve_alpha<T extends HierarchyAgentLike>(agents: readonly T[]): T | undefined {
    if (!agents || agents.length === 0) return undefined;

    return (
        agents.find(a => a.role?.toLowerCase() === 'ceo' || a.name?.toLowerCase().includes('nine')) ??
        agents.find(a => a.id === '1') ??
        agents[0]
    );
}

/**
 * Resolves the secondary Nexus (COO / Sub-Lead Agent).
 * Precedence:
 * 1. Role is 'COO' (case-insensitive) or name is 'Tadpole Alpha' / 'Tadpole' (distinct from Alpha)
 * 2. Agent with ID '2' (distinct from Alpha)
 * 3. First available agent distinct from Alpha
 */
export function resolve_nexus<T extends HierarchyAgentLike>(agents: readonly T[], alpha?: T | string): T | undefined {
    if (!agents || agents.length === 0) return undefined;

    const alpha_id = typeof alpha === 'string' ? alpha : alpha?.id;

    return (
        agents.find(a => 
            (a.role?.toLowerCase() === 'coo' || a.name === 'Tadpole Alpha' || a.name === 'Tadpole') && 
            a.id !== alpha_id
        ) ??
        agents.find(a => a.id === '2' && a.id !== alpha_id) ??
        agents.find(a => a.id !== alpha_id)
    );
}
