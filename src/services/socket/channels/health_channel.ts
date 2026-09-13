/**
 * @docs ARCHITECTURE:UI-Services
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / health_channel
 * - **Primary Entrypoints**: `HealthChannel`
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
