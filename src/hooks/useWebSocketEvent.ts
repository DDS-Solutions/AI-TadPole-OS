import { useEffect, useRef } from 'react';
import { TadpoleOSSocket } from '../services/socket';

/**
 * Subscribes to a specific WebSocket event channel with automatic cleanup.
 * Wraps the typed TadpoleOSSocket subscription methods.
 *
 * @param channel    - Which channel to subscribe to: 'agentUpdates' | 'health' | 'handoff' | 'status'
 * @param handler    - Callback invoked with the parsed event data
 * @param throttleMs - Optional throttle interval in ms (default: 0 = no throttle)
 */
export function useWebSocketEvent<T = unknown>(
    channel: 'agentUpdates' | 'health' | 'handoff' | 'status',
    handler: (data: T) => void,
    throttleMs = 0
) {
    const lastUpdate = useRef(0);
    const savedHandler = useRef(handler);

    useEffect(() => {
        savedHandler.current = handler;
    }, [handler]);

    useEffect(() => {
        TadpoleOSSocket.connect();

        const wrappedHandler = (data: unknown) => {
            if (throttleMs > 0) {
                const now = Date.now();
                if (now - lastUpdate.current < throttleMs) return;
                lastUpdate.current = now;
            }
            savedHandler.current(data as T);
        };

        let unsubscribe: (() => void) | undefined;

        switch (channel) {
            case 'agentUpdates':
                unsubscribe = TadpoleOSSocket.subscribeAgentUpdates(wrappedHandler as Parameters<typeof TadpoleOSSocket.subscribeAgentUpdates>[0]);
                break;
            case 'health':
                unsubscribe = TadpoleOSSocket.subscribeHealth(wrappedHandler as Parameters<typeof TadpoleOSSocket.subscribeHealth>[0]);
                break;
            case 'handoff':
                unsubscribe = TadpoleOSSocket.subscribeHandoff(wrappedHandler as Parameters<typeof TadpoleOSSocket.subscribeHandoff>[0]);
                break;
            case 'status':
                unsubscribe = TadpoleOSSocket.subscribeStatus(wrappedHandler as Parameters<typeof TadpoleOSSocket.subscribeStatus>[0]);
                break;
        }

        return () => {
            unsubscribe?.();
        };
    }, [channel, throttleMs]);
}


