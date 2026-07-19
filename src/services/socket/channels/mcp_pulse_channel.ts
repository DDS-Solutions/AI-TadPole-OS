/**
 * @docs ARCHITECTURE:UI-Services
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[mcp_pulse_channel]` in observability traces.
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

// Metadata: [mcp_pulse_channel]
