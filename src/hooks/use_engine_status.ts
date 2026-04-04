import { useState, useEffect } from 'react';
import { tadpole_os_socket, type Engine_Health_Event, type Connection_State } from '../services/socket';

/**
 * use_engine_status
 * 
 * Centralized hook for engine telemetry. 
 * Provides real-time health, connectivity, and performance metrics.
 */
export function use_engine_status() {
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
                cpu: (h.cpu as number) || 0,
                memory: (h.memory as number) || 0,
                latency: (h.latency as number) || 0,
                active_agents: h.agent_count || (h as any).active_agents || 0,
                max_depth: (h as any).max_depth || 0,
                tpm: (h as any).tpm || 0,
                recruit_count: (h as any).recruit_count || 0
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
