/**
 * @docs ARCHITECTURE:UI-Services
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[health_channel]` in observability traces.
 */

import { BaseChannel } from './channel';
import type { Incoming_Socket_Message, Engine_Health_Event } from '../types/events';

export class HealthChannel extends BaseChannel<Engine_Health_Event> {
    readonly name = 'health';

    matches(message: Incoming_Socket_Message): boolean {
        return message.type === 'engine:health';
    }

    handle(message: Incoming_Socket_Message): void {
        this.emit(message as Engine_Health_Event);
    }
}

// Metadata: [health_channel]
