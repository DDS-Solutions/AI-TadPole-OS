/**
 * @docs ARCHITECTURE:UI-Services
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / index
 * - **Primary Entrypoints**: `get_tadpole_os_socket`, `Tadpole_OS_Socket_Client`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { SocketManager } from './socket_manager';

// Re-export state types
export type { Connection_State, State_Listener } from './types/connection_state';

// Re-export event types
export type {
    Agent_Update_Event,
    Engine_Health_Event,
    Handoff_Event,
    Mcp_Pulse_Event,
    Socket_Log_Event,
    Socket_Agent_Message_Event,
    Socket_Trace_Span_Event,
    Socket_Trace_Span_Update_Event,
    Socket_Scheduled_Job_Complete_Event,
    Socket_Auth_Ok_Event,
    Socket_Auth_Error_Event,
    Incoming_Socket_Message
} from './types/events';

// Re-export utilities & classes
export { is_allowed_origin } from './security/origin_guard';
export { SocketManager } from './socket_manager';

let instance: Tadpole_OS_Socket_Client | null = null;

/**
 * Tadpole_OS_Socket_Client
 * Backwards compatible subclass for the new SocketManager facade.
 */
export class Tadpole_OS_Socket_Client extends SocketManager {
    static reset(): void {
        if (instance) {
            instance.destroy();
            instance = null;
        }
    }
}

export const get_tadpole_os_socket = (): Tadpole_OS_Socket_Client => {
    if (!instance) {
        instance = new Tadpole_OS_Socket_Client();
    }
    return instance;
};
