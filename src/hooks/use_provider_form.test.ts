/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Logic Verification**: Validates state transition integrity for the provider configuration reducer.
 * Ensures strict synchronization across all form fields and reactive state updates.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Invalid field update dispatch, or unexpected state property pollution.
 * - **Telemetry Link**: Search for `panel_reducer` in tracing logs.
 */

import { describe, it, expect } from 'vitest';
import { panel_reducer, type Panel_State } from './use_provider_form';

describe('panel_reducer', () => {
    const initial_state: Panel_State = {
        name: 'Initial Name',
        icon: '⚡',
        api_key: '',
        base_url: 'https://api.openai.com/v1',
        external_id: '',
        protocol: 'openai',
        custom_headers: '{}',
        audio_model: '',
        persist_to_engine: false,
        is_testing: false,
        test_result: 'idle',
        test_message: ''
    };

    it('updates name field correctly', () => {
        const action = { type: 'UPDATE_FIELD' as const, field: 'name' as const, value: 'New Name' };
        const next_state = panel_reducer(initial_state, action);
        expect(next_state.name).toBe('New Name');
    });

    it('updates api_key field correctly', () => {
        const action = { type: 'UPDATE_FIELD' as const, field: 'api_key' as const, value: 'sk-12345' };
        const next_state = panel_reducer(initial_state, action);
        expect(next_state.api_key).toBe('sk-12345');
    });

    it('updates protocol field correctly', () => {
        const action = { type: 'UPDATE_FIELD' as const, field: 'protocol' as const, value: 'anthropic' };
        const next_state = panel_reducer(initial_state, action);
        expect(next_state.protocol).toBe('anthropic');
    });

    it('updates is_testing field correctly', () => {
        const action = { type: 'UPDATE_FIELD' as const, field: 'is_testing' as const, value: true };
        const next_state = panel_reducer(initial_state, action);
        expect(next_state.is_testing).toBe(true);
    });

    it('updates test_result field correctly', () => {
        const action = { type: 'UPDATE_FIELD' as const, field: 'test_result' as const, value: 'success' };
        const next_state = panel_reducer(initial_state, action);
        expect(next_state.test_result).toBe('success');
    });

    it('preserves other state fields when one is updated', () => {
        const action = { type: 'UPDATE_FIELD' as const, field: 'name' as const, value: 'Updated Name' };
        const next_state = panel_reducer(initial_state, action);
        expect(next_state.name).toBe('Updated Name');
        expect(next_state.icon).toBe('⚡');
        expect(next_state.base_url).toBe('https://api.openai.com/v1');
    });
});

// Metadata: [use_provider_form_test]

// Metadata: [use_provider_form_test]
