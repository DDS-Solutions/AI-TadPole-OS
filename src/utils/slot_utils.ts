/**
 * @docs ARCHITECTURE:Infrastructure
 * 
 * ### AI Assist Note
 * **Slot Orchestration**: Logic for mapping cognition slot states to backend DTOs.
 * Ensures that model parameters (temperature, depth, ACT) are correctly serialized.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Improper mapping of optional skills or workflows leading to 400 Bad Request from backend.
 * - **Telemetry Link**: Search `[SlotUtils]` in UI traces.
 */

console.debug("[SlotUtils] Module loaded");

import type { Agent_Model_Slot_State } from '../types';
export const build_slot_config = (slot: Agent_Model_Slot_State) => {
    return {
        model_id: slot.model,
        provider: slot.provider,
        temperature: slot.temperature,
        system_prompt: slot.system_prompt,
        reasoning_depth: slot.reasoning_depth,
        act_threshold: slot.act_threshold,
        skills: slot.skills,
        workflows: slot.workflows
    };
};

/**
 * get_aggregate_skills
 * Extracts a unique set of skills across all provided model slots.
 */
export const get_aggregate_skills = (slots: Record<string, Agent_Model_Slot_State>) => {
    const all_skills = Object.values(slots).flatMap(s => s.skills || []);
    return [...new Set(all_skills)];
};

/**
 * get_aggregate_workflows
 * Extracts a unique set of workflows across all provided model slots.
 */
export const get_aggregate_workflows = (slots: Record<string, Agent_Model_Slot_State>) => {
    const all_workflows = Object.values(slots).flatMap(s => s.workflows || []);
    return [...new Set(all_workflows)];
};

// Metadata: [slot_utils]

// Metadata: [slot_utils]
