/**
 * @docs ARCHITECTURE:Logic
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend React Hooks / use_web_socket_event
 * - **Primary Entrypoints**: `useWebSocketEvent`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` React hook lifecycle adheres to Rules of Hooks without conditional execution branches.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { useEffect, useRef } from 'react';
import { get_tadpole_os_socket } from '../services/socket';

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
    const has_pending_data = useRef(false);

    useEffect(() => {
        saved_handler.current = handler;
    }, [handler]);

    useEffect(() => {
        get_tadpole_os_socket().connect();

        const wrapped_handler = (data: unknown) => {
            if (throttle_ms > 0) {
                const now = Date.now();
                const elapsed = now - last_update.current;

                if (elapsed < throttle_ms) {
                    pending_data.current = data;
                    has_pending_data.current = true;

                    if (!throttle_timeout.current) {
                        const remaining = throttle_ms - elapsed;
                        throttle_timeout.current = setTimeout(() => {
                            if (has_pending_data.current) {
                                saved_handler.current(pending_data.current as T);
                                pending_data.current = null;
                                has_pending_data.current = false;
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
                has_pending_data.current = false;
                last_update.current = now;
            }
            saved_handler.current(data as T);
        };

        const unsubscribe = get_tadpole_os_socket().subscribe(channel, wrapped_handler);

        return () => {
            unsubscribe();
            if (throttle_timeout.current) {
                clearTimeout(throttle_timeout.current);
                throttle_timeout.current = null;
            }
        };
    }, [channel, throttle_ms]);
}
