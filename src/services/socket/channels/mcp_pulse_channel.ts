/**
 * @docs ARCHITECTURE:UI-Services
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / mcp_pulse_channel
 * - **Primary Entrypoints**: `McpPulseChannel`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { BaseChannel } from './channel';
import type { Incoming_Socket_Message, Mcp_Pulse_Event } from '../types/events';

export class McpPulseChannel extends BaseChannel<Mcp_Pulse_Event> {
    readonly name = 'pulse';

    matches(message: Incoming_Socket_Message): boolean {
        return message.type === 'engine:mcp_pulse';
    }

    handle(message: Incoming_Socket_Message): void {
        this.emit(message as Mcp_Pulse_Event);
    }
}
