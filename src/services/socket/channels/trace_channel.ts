/**
 * @docs ARCHITECTURE:UI-Services
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[trace_channel]` in observability traces.
 */

import { BaseChannel } from './channel';
import type { Incoming_Socket_Message, Socket_Trace_Span_Event, Socket_Trace_Span_Update_Event } from '../types/events';
import { event_bus } from '../../event_bus';

export class TraceChannel extends BaseChannel<Incoming_Socket_Message> {
    readonly name = 'trace';

    matches(message: Incoming_Socket_Message): boolean {
        return message.type === 'trace:span' || message.type === 'trace:span_update';
    }

    handle(message: Incoming_Socket_Message): void {
        this.emit(message);
        
        if (message.type === 'trace:span') {
            const data = message as Socket_Trace_Span_Event;
            event_bus.emit_trace(data.span);
        } else if (message.type === 'trace:span_update') {
            const data = message as Socket_Trace_Span_Update_Event;
            event_bus.emit_trace({
                id: (data.span_id || data.spanId) as string,
                ...data.update
            });
        }
    }
}

// Metadata: [trace_channel]
