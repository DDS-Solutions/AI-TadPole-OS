/**
 * @docs ARCHITECTURE:Logic
 * @docs OPERATIONS_MANUAL:Telemetry
 * 
 * ### AI Assist Note
 * **Central Telemetry Hook**: Provides real-time health, connectivity, and performance metrics from the Tadpole Engine. 
 * Subscribes to health, status, and swarm pulse events via WebSocket.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: WebSocket closure (shows "OFFLINE"), stale pulse data (if pulse interval > health interval), or missing event fields in the `Engine_Health_Event`.
 * - **Telemetry Link**: Subscribes to `tadpole_os_socket` for `SYNC_HEALTH` and `SWARM_PULSE` events.
 */

import { useState, useEffect } from 'react';
import { tadpole_os_socket, type Engine_Health_Event, type Connection_State } from '../services/socket';

/**
 * Interface for Engine Status Telemetry
 */
export interface Engine_Status {
    status: Connection_State;
    connection_state: Connection_State;
    is_online: boolean;
    health: Engine_Health_Event | null;
    cpu: number;
    memory: number;
    latency: number;
    active_agents: number;
    max_depth: number;
    tpm: number;
    recruit_count: number;
}

/**
 * useEngineStatus
 * 
 * Centralized hook for engine telemetry. 
 * Provides real-time health, connectivity, and performance metrics.
 */
export function useEngineStatus(): Engine_Status {
    const [status, set_status] = useState<Connection_State>(tadpole_os_socket.get_connection_state());
    const [is_online, set_is_online] = useState(tadpole_os_socket.get_connection_state() === 'connected');
    const [health, set_health] = useState<Engine_Health_Event | null>(null);

    // Deep Telemetry Metrics (Mirrored from Engine_Dashboard expectations)
    const [metrics, set_metrics] = useState({
        cpu: 0,
        memory: 0,
        latency: 0,
        active_agents: 0,
        max_depth: 0,
        tpm: 0,
        recruit_count: 0
    });

    useEffect(() => {
        const unsubscribe_status = tadpole_os_socket.subscribe_status((new_state) => {
            set_status(new_state as Connection_State);
            set_is_online(new_state === 'connected');
        });

        const unsubscribe_health = tadpole_os_socket.subscribe_health((h: Engine_Health_Event) => {
            set_health(h);
            
            // Map event fields to UI metrics, with fallback logic for legacy/transient states
            set_metrics({
                cpu: h.cpu ?? 0,
                memory: h.memory ?? 0,
                latency: h.latency ?? 0,
                active_agents: h.active_agents ?? h.agent_count ?? 0,
                max_depth: h.max_depth ?? 0,
                tpm: h.tpm ?? 0,
                recruit_count: h.recruit_count ?? 0
            });
        });

        const unsubscribe_pulse = tadpole_os_socket.subscribe_swarm_pulse((pulse) => {
            // Swarm pulse can also update agent counts if health event is delayed
            if (pulse.nodes) {
                set_metrics(prev => ({
                    ...prev,
                    active_agents: pulse.nodes.length
                }));
            }
        });

        return () => {
            unsubscribe_status();
            unsubscribe_health();
            unsubscribe_pulse();
        };
    }, []);

    return { 
        status, 
        connection_state: status, // Alias for legacy component support
        is_online, 
        health,
        ...metrics
    };
}

