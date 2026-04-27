/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Verification Suite**: Slot Orchestration Logic. 
 * Validates the serialization of model slots and the aggregation of unique capabilities (skills/workflows) across the agent's cognition stack.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Duplicate skill entries not being filtered or missing model IDs in DTO serialization.
 * - **Telemetry Link**: Run `npm run test` or check `[slot_utils.test]` in Vitest logs.
 */

import { describe, it, expect } from 'vitest';
import { build_slot_config, get_aggregate_skills, get_aggregate_workflows } from './slot_utils';
import type { Agent_Model_Slot_State } from '../types';

describe('slot_utils', () => {
    const mock_slot: Agent_Model_Slot_State = {
        model: 'gemini-pro',
        provider: 'google',
        temperature: 0.7,
        system_prompt: 'Test prompt',
        reasoning_depth: 10,
        act_threshold: 0.5,
        skills: ['skill-1', 'skill-2'],
        workflows: ['wf-1']
    };

    describe('build_slot_config', () => {
        it('correctly maps a slot state to a backend DTO', () => {
            const config = build_slot_config(mock_slot);
            expect(config.model_id).toBe('gemini-pro');
            expect(config.temperature).toBe(0.7);
            expect(config.skills).toEqual(['skill-1', 'skill-2']);
        });
    });

    describe('get_aggregate_skills', () => {
        it('aggregates unique skills across multiple slots', () => {
            const slots: Record<string, Agent_Model_Slot_State> = {
                primary: { ...mock_slot, skills: ['s1', 's2'] },
                secondary: { ...mock_slot, skills: ['s2', 's3'] }
            };
            const result = get_aggregate_skills(slots);
            expect(result).toEqual(['s1', 's2', 's3']);
            expect(result.length).toBe(3);
        });

        it('handles slots with no skills gracefully', () => {
            const slots: Record<string, Agent_Model_Slot_State> = {
                primary: { ...mock_slot, skills: [] }
            };
            const result = get_aggregate_skills(slots);
            expect(result).toEqual([]);
        });
    });

    describe('get_aggregate_workflows', () => {
        it('aggregates unique workflows across multiple slots', () => {
            const slots: Record<string, Agent_Model_Slot_State> = {
                primary: { ...mock_slot, workflows: ['w1'] },
                secondary: { ...mock_slot, workflows: ['w1', 'w2'] }
            };
            const result = get_aggregate_workflows(slots);
            expect(result).toEqual(['w1', 'w2']);
        });
    });
});

// Metadata: [slot_utils_test]

// Metadata: [slot_utils_test]
