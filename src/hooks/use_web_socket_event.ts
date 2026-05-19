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
    const throttle_timeout = useRef<ReturnType<typeof setTimeout> | null>(null);
    const pending_data = useRef<unknown>(null);

    useEffect(() => {
        saved_handler.current = handler;
    }, [handler]);

    useEffect(() => {
        tadpole_os_socket.connect();

        const wrapped_handler = (data: unknown) => {
            if (throttle_ms > 0) {
                const now = Date.now();
                const elapsed = now - last_update.current;

                if (elapsed < throttle_ms) {
                    pending_data.current = data;

                    if (!throttle_timeout.current) {
                        const remaining = throttle_ms - elapsed;
                        throttle_timeout.current = setTimeout(() => {
                            if (pending_data.current !== null) {
                                saved_handler.current(pending_data.current as T);
                                pending_data.current = null;
                            }
                            last_update.current = Date.now();
                            throttle_timeout.current = null;
                        }, remaining);
                    }
                    return;
                }

                if (throttle_timeout.current) {
                    clearTimeout(throttle_timeout.current);
                    throttle_timeout.current = null;
                }
                pending_data.current = null;
                last_update.current = now;
            }
            saved_handler.current(data as T);
        };

        const unsubscribe = tadpole_os_socket.subscribe(channel, wrapped_handler);

        return () => {
            unsubscribe();
            if (throttle_timeout.current) {
                clearTimeout(throttle_timeout.current);
                throttle_timeout.current = null;
            }
        };
    }, [channel, throttle_ms]);
}


// Metadata: [use_web_socket_event]

// Metadata: [use_web_socket_event]
