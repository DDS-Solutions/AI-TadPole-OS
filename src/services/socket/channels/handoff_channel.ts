/**
 * @docs ARCHITECTURE:UI-Services
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[handoff_channel]` in observability traces.
 */

import { BaseChannel } from './channel';
import type { Incoming_Socket_Message, Handoff_Event } from '../types/events';

export class HandoffChannel extends BaseChannel<Handoff_Event> {
    readonly name = 'handoff';

    matches(message: Incoming_Socket_Message): boolean {
        return message.type === 'agent:handoff';
    }

    handle(message: Incoming_Socket_Message): void {
        this.emit(message as Handoff_Event);
    }
}

// Metadata: [handoff_channel]
