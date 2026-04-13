/**
 * @docs ARCHITECTURE:Logic
 * 
 * ### AI Assist Note
 * **Provider Config Hook**: Manages the ephemeral state for AI Infrastructure provider setup. 
 * Facilitates connectivity testing and credential validation before persistence to the `provider_store`.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Reducer update collisions, validation rejection for base URLs, or UI hang during `test_provider` RPC calls.
 * - **Telemetry Link**: Check `panel_reducer` logs or search `[useProviderForm]` in tracing.
 */

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
    protocol: Provider_Config['protocol'] | string;
    custom_headers: string; // JSON string
    audio_model: string;
    persist_to_engine: boolean;
    is_testing: boolean;
    test_result: 'idle' | 'success' | 'failed';
    test_message: string;
}

export type Action =
    | { type: 'UPDATE_FIELD'; field: keyof Panel_State; value: string | boolean | 'idle' | 'success' | 'failed' | Provider_Config['protocol'] };

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

