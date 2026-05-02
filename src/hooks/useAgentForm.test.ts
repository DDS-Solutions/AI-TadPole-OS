/**
 * @docs ARCHITECTURE:UI-Hooks
 * 
 * ### AI Assist Note
 * **Verification and quality assurance for the Tadpole OS engine.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[useAgentForm_test]` in observability traces.
 */

import { describe, it, expect, vi } from 'vitest';
import { config_reducer, Agent_Config_State } from './useAgentForm';

// Mock role store for RESET_ROLE
vi.mock('../stores/role_store', () => ({
    use_role_store: {
        getState: () => ({
            roles: {
                'Engineer': { skills: ['coding'], workflows: ['build'] }
            }
        })
    }
}));

describe('config_reducer', () => {
    const initialState: Agent_Config_State = {
        main_tab: 'cognition',
        active_tab: 'primary',
        identity: { name: '', role: '', department: '' },
        voice: { voice_id: '', voice_engine: 'elevenlabs' },
        slots: {
            primary: { model: '', provider: '', system_prompt: '', temperature: 0.7, reasoning_depth: 1, act_threshold: 0.5, skills: [], workflows: [] },
            secondary: { model: '', provider: '', system_prompt: '', temperature: 0.7, reasoning_depth: 1, act_threshold: 0.5, skills: [], workflows: [] },
            tertiary: { model: '', provider: '', system_prompt: '', temperature: 0.7, reasoning_depth: 1, act_threshold: 0.5, skills: [], workflows: [] },
        },
        mcp_tools: [],
        governance: { budget_usd: 0, requires_oversight: false },
        ui: { direct_message: '', saving: false, theme_color: '', new_role_name: '', show_promote: false },
        connector_configs: []
    };

    it('handles SET_MAIN_TAB', () => {
        const state = config_reducer(initialState, { type: 'SET_MAIN_TAB', payload: 'memory' });
        expect(state.main_tab).toBe('memory');
    });

    it('handles UPDATE_IDENTITY', () => {
        const state = config_reducer(initialState, { type: 'UPDATE_IDENTITY', field: 'name', value: 'Agent Smith' });
        expect(state.identity.name).toBe('Agent Smith');
    });

    it('handles TOGGLE_SKILL', () => {
        // Add
        let state = config_reducer(initialState, { type: 'TOGGLE_SKILL', slot: 'primary', kind: 'skills', value: 'rust' });
        expect(state.slots.primary.skills).toContain('rust');
        
        // Remove
        state = config_reducer(state, { type: 'TOGGLE_SKILL', slot: 'primary', kind: 'skills', value: 'rust' });
        expect(state.slots.primary.skills).not.toContain('rust');
    });

    it('handles RESET_ROLE', () => {
        const state = config_reducer(initialState, { type: 'RESET_ROLE', role: 'Engineer' });
        expect(state.identity.role).toBe('Engineer');
        expect(state.slots.primary.skills).toEqual(['coding']);
        expect(state.slots.primary.workflows).toEqual(['build']);
    });

    it('handles ADD_CONNECTOR and REMOVE_CONNECTOR', () => {
        const connector = { id: '1', type: 'fs', uri: '/tmp', options: {} } as any;
        let state = config_reducer(initialState, { type: 'ADD_CONNECTOR', payload: connector });
        expect(state.connector_configs).toHaveLength(1);
        
        state = config_reducer(state, { type: 'REMOVE_CONNECTOR', uri: '/tmp' });
        expect(state.connector_configs).toHaveLength(0);
    });
});

// Metadata: [useAgentForm_test]
