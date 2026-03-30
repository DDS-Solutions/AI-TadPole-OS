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
    budgetUsd?: number;
    /** Current cost accumulated by this mission (USD) */
    costUsd?: number;
    /** If the mission was executed under degraded conditions (e.g. NullProvider) */
    is_degraded?: boolean;
}
