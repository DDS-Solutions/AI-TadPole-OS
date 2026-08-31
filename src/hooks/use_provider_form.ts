/**
 * @docs ARCHITECTURE:Logic
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend React Hooks / use_provider_form
 * - **Primary Entrypoints**: `panel_reducer`, `Panel_State`, `Action`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` React hook lifecycle adheres to Rules of Hooks without conditional execution branches.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
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
    supports_steering_vectors: boolean;
    is_testing: boolean;
    is_syncing: boolean;
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
