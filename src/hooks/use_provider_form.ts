import { type Provider_Config } from '../stores/provider_store';

/**
 * Panel_State
 * Defines the state for the provider configuration panel.
 * Refactored for strict snake_case compliance and backend parity.
 */
export interface Panel_State {
    name: string;
    icon: string;
    api_key: string;
    base_url: string;
    external_id: string;
    protocol: Provider_Config['protocol'];
    custom_headers: string; // JSON string
    audio_model: string;
    persist_to_engine: boolean;
    is_testing: boolean;
    test_result: 'idle' | 'success' | 'failed';
    test_message: string;
}

export type Action =
    | { type: 'UPDATE_FIELD'; field: keyof Panel_State; value: Panel_State[keyof Panel_State] };

/**
 * panel_reducer
 * Reducer for managing the provider configuration form state.
 */
export function panel_reducer(state: Panel_State, action: Action): Panel_State {
    switch (action.type) {
        case 'UPDATE_FIELD':
            return { ...state, [action.field]: action.value };
        default:
            return state;
    }
}
