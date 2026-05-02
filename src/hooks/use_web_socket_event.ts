/**
 * @docs ARCHITECTURE:Logic
 * 
 * ### AI Assist Note
 * **WebSocket Lifecycle Hook**: Subscribes to specific WebSocket event channels with automatic cleanup and optional throttling. 
 * Wraps the typed `tadpole_os_socket` subscription methods for component-level reactivity.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Channel mismatch (subscribing to non-existent topic), stale data if `throttle_ms` is too high, or implicit connection overhead if called in high-frequency loops.
 * - **Telemetry Link**: Search for `[useWebSocketEvent]` or `[Tadpole_OS_Socket]` in browser logs.
 */

import { useEffect, useRef } from 'react';
import { tadpole_os_socket } from '../services/socket';

/**
 * use_web_socket_event
 * Subscribes to a specific WebSocket event channel with automatic cleanup.
 * Wraps the typed tadpole_os_socket subscription methods.
 *
 * @param channel    - Which channel to subscribe to: 'agentUpdates' | 'health' | 'handoff' | 'status'
 * @param handler    - Callback invoked with the parsed event data
 * @param throttle_ms - Optional throttle interval in ms (default: 0 = no throttle)
 */
export function useWebSocketEvent<T = unknown>(
    channel: 'agentUpdates' | 'health' | 'handoff' | 'status',
    handler: (data: T) => void,
    throttle_ms = 0
) {
    const last_update = useRef(0);
    const saved_handler = useRef(handler);

    useEffect(() => {
        saved_handler.current = handler;
    }, [handler]);

    useEffect(() => {
        tadpole_os_socket.connect();

        const wrapped_handler = (data: unknown) => {
            if (throttle_ms > 0) {
                const now = Date.now();
                if (now - last_update.current < throttle_ms) return;
                last_update.current = now;
            }
            saved_handler.current(data as T);
        };

        let unsubscribe: (() => void) | undefined;

        switch (channel) {
            case 'agentUpdates':
                unsubscribe = tadpole_os_socket.subscribe_agent_updates(wrapped_handler as Parameters<typeof tadpole_os_socket.subscribe_agent_updates>[0]);
                break;
            case 'health':
                unsubscribe = tadpole_os_socket.subscribe_health(wrapped_handler as Parameters<typeof tadpole_os_socket.subscribe_health>[0]);
                break;
            case 'handoff':
                unsubscribe = tadpole_os_socket.subscribe_handoff(wrapped_handler as Parameters<typeof tadpole_os_socket.subscribe_handoff>[0]);
                break;
            case 'status':
                unsubscribe = tadpole_os_socket.subscribe_status(wrapped_handler as Parameters<typeof tadpole_os_socket.subscribe_status>[0]);
                break;
        }

        return () => {
            unsubscribe?.();
        };
    }, [channel, throttle_ms]);
}


// Metadata: [use_web_socket_event]

// Metadata: [use_web_socket_event]
