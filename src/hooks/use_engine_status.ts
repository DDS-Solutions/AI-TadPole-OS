/**
 * @docs ARCHITECTURE:Logic
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend React Hooks / use_engine_status
 * - **Primary Entrypoints**: `useEngineStatus`, `Engine_Status`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` React hook lifecycle adheres to Rules of Hooks without conditional execution branches.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { useState, useEffect } from 'react';
import { get_tadpole_os_socket, type Engine_Health_Event, type Connection_State } from '../services/socket';

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
    agent_count: number;
    max_depth: number;
    tpm: number;
    recruit_count: number;
    max_memory: number;
}

/**
 * useEngineStatus
 * 
 * Centralized hook for engine telemetry. 
 * Provides real-time health, connectivity, and performance metrics.
 */
export function useEngineStatus(): Engine_Status {
    const [status, set_status] = useState<Connection_State>(get_tadpole_os_socket().get_connection_state());
    const [health, set_health] = useState<Engine_Health_Event | null>(null);

    // Deep Telemetry Metrics (Mirrored from Engine_Dashboard expectations)
    const [metrics, set_metrics] = useState({
        cpu: 0,
        memory: 0,
        latency: 0,
        active_agents: 0,
        agent_count: 0,
        max_depth: 0,
        tpm: 0,
        recruit_count: 0,
        max_memory: 16
    });

    useEffect(() => {
        const unsubscribe_status = get_tadpole_os_socket().subscribe('status', (new_state) => {
            set_status(new_state as Connection_State);
        });

        const unsubscribe_health = get_tadpole_os_socket().subscribe('health', (h: Engine_Health_Event) => {
            set_health(h);


            // Map event fields to UI metrics, with fallback logic for legacy/transient states
            set_metrics({
                cpu: h.cpu ?? 0,
                memory: h.memory ?? 0,
                max_memory: h.maxMemory as number ?? 16,
                latency: h.latency ?? 0,
                active_agents: h.active_agents ?? 0,
                agent_count: h.agent_count ?? h.active_agents ?? 0,
                max_depth: h.max_depth ?? 0,
                tpm: h.tpm ?? 0,
                recruit_count: h.recruit_count ?? 0
            });
        });

        const unsubscribe_pulse = get_tadpole_os_socket().subscribe_swarm_pulse((pulse) => {
            // Swarm pulse can also update agent counts if health event is delayed
            if (pulse.nodes) {
                set_metrics(prev => ({
                    ...prev,
                    agent_count: pulse.nodes.length,
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
        is_online: status === 'connected',
        health,
        ...metrics
    };
}
