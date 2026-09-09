/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Pages / Agent_Manager
 * - **Primary Entrypoints**: `Agent_Manager`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { useAgentManager } from '../hooks/useAgentManager';
import AgentConfigPanel from '../components/AgentConfigPanel';

// Modular Components
import { Agent_Card_Memo } from '../components/agents/Agent_Card';
import { Agent_Manager_Header } from '../components/agents/Agent_Manager_Header';

export default function Agent_Manager() {
    const {
        agents,
        user_agents_count,
        ai_agents_count,
        selected_agent,
        is_creating,
        search_query,
        filter_role,
        active_tab,
        filter_roles,
        
        set_selected_agent,
        set_is_creating,
        set_search_query,
        set_filter_role,
        set_active_tab,
        
        handle_update_agent,
        handle_add_new_click,
        handle_delete_agent
    } = useAgentManager();

    return (
        <div className="h-full flex flex-col bg-[color:var(--color-background)]">
            <script type="application/ld+json">
                {JSON.stringify({
                    "@context": "https://schema.org",
                    "@type": "SoftwareApplication",
                    "name": "Tadpole OS Agent Manager",
                    "description": "Unified orchestration layer for managing autonomous agent swarms.",
                    "author": { "@type": "Organization", "name": "Sovereign Engineering" },
                    "applicationCategory": "AI Management System",
                    "operatingSystem": "Tadpole OS"
                })}
            </script>

            <Agent_Manager_Header 
                active_tab={active_tab}
                user_count={user_agents_count}
                ai_count={ai_agents_count}
                filter_role={filter_role}
                filter_roles={filter_roles}
                search_query={search_query}
                on_add_click={handle_add_new_click}
                on_tab_change={set_active_tab}
                on_filter_change={set_filter_role}
                on_search_change={set_search_query}
            />

            <div className="flex-1 overflow-y-auto custom-scrollbar p-6">
                <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
                    {agents.map(agent => (
                        <Agent_Card_Memo key={agent.id} agent={agent} on_select={set_selected_agent} />
                    ))}
                </div>
            </div>

            {selected_agent && (
                <AgentConfigPanel
                    agent={selected_agent}
                    onClose={() => {
                        set_selected_agent(null);
                        set_is_creating(false);
                    }}
                    onUpdate={handle_update_agent}
                    onDelete={handle_delete_agent}
                    isNew={is_creating}
                />
            )}
        </div>
    );
}
