/**
 * @docs ARCHITECTURE:UI-Services
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / handoff_channel
 * - **Primary Entrypoints**: `HandoffChannel`
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
