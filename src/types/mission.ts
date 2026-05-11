/**
 * @docs ARCHITECTURE:Types
 * 
 * ### AI Assist Note
 * **Mission Type Definitions**: Structural blueprint for goals, constraints, and priorities. 
 * Orchestrates the validation of objective strings and budget allocations for agent swarm tasks.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Missing `objective` property in raw JSON, or invalid `priority` enum value after backend expansion.
 * - **Telemetry Link**: Look for `interface Mission` during backend parity audits.
 */

/**
 * @module Mission
 * Represents a structured goal or objective assigned to an agent swarm member.
 * Synchronized with the local workspace to ensure persistent mission alignment.
 */

export interface Mission {
    /** Unique identifier for the mission */
    id: string;
    /** High-level goal or task description */
    objective: string;
    /** Specific constraints or rules for the mission */
    constraints: string[];
    /** Priority level for mission execution */
    priority: 'low' | 'medium' | 'high';
    /** Optional expiration or deadline */
    deadline?: string;
    /** IDs of other missions that must be completed first */
    dependencies?: string[];
    /** Allocated budget for this mission (USD) */
    budget_usd?: number;
    /** Current cost accumulated by this mission (USD) */
    cost_usd?: number;
    /** If the mission was executed under degraded conditions (e.g. NullProvider) */
    is_degraded?: boolean;
}


// Metadata: [mission]

// Metadata: [mission]
