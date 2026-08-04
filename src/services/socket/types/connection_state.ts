/**
 * @docs ARCHITECTURE:UI-Services
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[connection_state]` in observability traces.
 */

export type Connection_State = 'connecting' | 'authenticating' | 'connected' | 'disconnected' | 'reconnecting' | 'error';

export type State_Listener = (state: Connection_State) => void;

// Metadata: [connection_state]
