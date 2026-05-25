/**
 * @docs ARCHITECTURE:UI-Hooks
 * 
 * ### AI Assist Note
 * **useLayoutServices**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[useLayoutServices]` in observability traces.
 */

import { useEffect } from 'react';
import { tadpole_os_socket } from '../../services/socket';
import { use_skill_store } from '../../stores/skill_store';
import { agent_service } from '../../services/agent_service';
import { use_notification_store } from '../../stores/notification_store';
import { event_bus } from '../../services/event_bus';
import { provider_service } from '../../services/provider_service';
import { use_agent_store } from '../../stores/agent_store';

/**
 * useLayoutServices
 * Orchestrates global background services: socket pulses, agent refreshes, and the unified notification hub.
 */
export function useLayoutServices() {
    // ── Socket Pulse ──────────────────────────────────────
    useEffect(() => {
        tadpole_os_socket.connect();
        const unsubscribe_pulse = tadpole_os_socket.subscribe_pulse((pulse) => {
            use_skill_store.getState().handle_pulse(pulse.tool, pulse.status, pulse.latency);
        });
        return () => {
            unsubscribe_pulse();
        };
    }, []);

    // ── Neural Infrastructure Sync ────────────────────────
    useEffect(() => {
        const unsubscribe = provider_service.init();
        void provider_service.sync_with_backend();
        return unsubscribe;
    }, []);

    // ── Agent Swarm Sync ─────────────────────────────────
    useEffect(() => {
        const unsubscribe = agent_service.init();
        void agent_service.load_agents_into_store();

        const handle_refresh_agents = () => {
            void use_agent_store.getState().fetch_agents();
        };
        window.addEventListener('app:refresh-agents', handle_refresh_agents);

        return () => {
            unsubscribe();
            window.removeEventListener('app:refresh-agents', handle_refresh_agents);
        };
    }, []);

    // ── Unified Notification Hub ─────────────────────────
    useEffect(() => {
        const { add_notification } = use_notification_store.getState();

        const unsubscribe = event_bus.subscribe_logs((entry) => {
            // Only pipe errors or specifically tagged security/governance events to the Hub
            const is_high_priority = 
                entry.severity === 'error' || 
                entry.severity === 'warning' ||
                entry.text.toLowerCase().includes('budget') ||
                entry.text.toLowerCase().includes('security') ||
                entry.text.toLowerCase().includes('injection');

            if (is_high_priority) {
                const is_persistent = 
                    entry.text.toLowerCase().includes('budget') || 
                    entry.text.toLowerCase().includes('injection') ||
                    entry.text.toLowerCase().includes('sanitizer');

                const title = entry.source === 'Agent' 
                    ? (entry.agent_name || entry.agent_id ? `Agent Alert: ${entry.agent_name || entry.agent_id}` : 'Agent Alert')
                    : 'System Alert';

                add_notification({
                    severity: entry.severity,
                    title,
                    message: entry.text,
                    type_id: entry.metadata?.type_id as string,
                    persistent: is_persistent,
                });
            }
        });

        return unsubscribe;
    }, []);
}

// Metadata: [useLayoutServices]
