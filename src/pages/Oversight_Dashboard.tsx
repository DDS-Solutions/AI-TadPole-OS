/**
 * @docs ARCHITECTURE:UI-Pages
 * 
 * ### AI Assist Note
 * **Oversight_Dashboard Page**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Oversight_Dashboard]` in observability traces.
 */

import { useNavigate } from 'react-router-dom';
import { useOversightDashboard } from '../hooks/useOversightDashboard';
import { use_workspace_store } from '../stores/workspace_store';
import { Simulation_Banner } from '../components/oversight/Simulation_Banner';
import { Oversight_Stats } from '../components/oversight/Oversight_Stats';
import { Pending_Queue } from '../components/oversight/Pending_Queue';
import { Action_Ledger } from '../components/oversight/Action_Ledger';
import { Swarm_Intelligence_View } from '../components/oversight/Swarm_Intelligence_View';
import { Command_Table } from '../components/Command_Table';

/**
 * Oversight_Dashboard Page
 * 
 * ### 🛡️ Governance & Oversight
 * Root view for swarm governance, compliance, and safety controls.
 * Features:
 * - **Action Approval**: Human-in-the-loop verification for sensitive agent actions.
 * - **Immutable Ledger**: Persistent record of all governance decisions.
 * - **Swarm Intelligence**: Visualization of alpha-node optimization traces.
 * - **Emergency Controls**: Global kill-switch and engine shutdown protocols.
 * 
 * Refactored to Container-Presentational pattern.
 */
export default function Oversight_Dashboard() {
    const navigate = useNavigate();
    const { clusters, active_proposals } = use_workspace_store();
    const {
        stats,
        filter,
        is_simulated,
        selected_cluster_id,
        is_online,
        set_filter,
        set_selected_cluster_id,
        set_is_simulated,
        handle_decide,
        handle_kill_switch,
        handle_kill_engine,
        resolve_agent_name,
        filtered_ledger,
        filtered_pending
    } = useOversightDashboard();

    return (
        <div className="p-6 space-y-6 max-w-7xl mx-auto">
            {/* GEO Optimization: Structured Data & Semantic Header */}
            <script type="application/ld+json">
                {JSON.stringify({
                    "@context": "https://schema.org",
                    "@type": "SoftwareApplication",
                    "name": "Tadpole OS Oversight Dashboard",
                    "description": "High-level governance and oversight dashboard for swarm-wide compliance and safety protocols.",
                    "provider": { "@type": "Organization", "name": "Sovereign Engineering" },
                    "applicationCategory": "Governance System"
                })}
            </script>
            <h1 className="sr-only">Tadpole OS Oversight & Governance Command</h1>

            <Simulation_Banner 
                is_simulated={is_simulated} 
                set_is_simulated={set_is_simulated} 
            />

            <Oversight_Stats 
                stats={stats}
                is_online={is_online}
                handle_kill_switch={handle_kill_switch}
                handle_kill_engine={handle_kill_engine}
                on_navigate_security={() => navigate('/security')}
            />

            <Pending_Queue 
                pending={filtered_pending}
                resolve_agent_name={resolve_agent_name}
                handle_decide={handle_decide}
            />

            <Action_Ledger 
                ledger={filtered_ledger}
                filter={filter}
                set_filter={set_filter}
                selected_cluster_id={selected_cluster_id}
                set_selected_cluster_id={set_selected_cluster_id}
                clusters={clusters || []}
                resolve_agent_name={resolve_agent_name}
            />

            <Swarm_Intelligence_View 
                active_proposals={active_proposals || {}}
                clusters={clusters || []}
                resolve_agent_name={resolve_agent_name}
            />

            {/* Neural Footprint Monitoring (Standard Command Table) */}
            <Command_Table />
        </div>
    );
}

// Metadata: [Oversight_Dashboard]

// Metadata: [Oversight_Dashboard]
