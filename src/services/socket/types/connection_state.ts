/**
 * @docs ARCHITECTURE:UI-Services
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / connection_state
 * - **Primary Entrypoints**: `Connection_State`, `State_Listener`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

export type Connection_State = 'connecting' | 'authenticating' | 'connected' | 'disconnected' | 'reconnecting' | 'error';

export type State_Listener = (state: Connection_State) => void;
