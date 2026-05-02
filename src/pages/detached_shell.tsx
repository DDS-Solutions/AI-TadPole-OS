/**
 * @docs ARCHITECTURE:Interface
 * @docs OPERATIONS_MANUAL:Navigation
 * 
 * ### AI Assist Note
 * **Dispatcher Shell**: A unified entry point for all native detached windows in the Tadpole OS.
 * Orchestrates standalone rendering of components (Chat, Logs, Trace) in high-fidelity isolated webview windows.
 * Use URL parameters (?type) to route specific components to this shell.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Null route type, missing search parameters, or lazy-load synchronization errors.
 * - **Telemetry Link**: Search for `[DetachedShell]` in UI traces or check `Portal_Window` spawn logs.
 */

import { Suspense, lazy } from 'react';
import { useSearchParams } from 'react-router-dom';
import { useDashboardData } from '../hooks/use_dashboard_data';
import { get_route_by_path } from '../constants/routes';
import Error_Boundary from '../components/Error_Boundary';

// lazy-loaded components to match the main app's architecture
const System_Log = lazy(() => import('../components/dashboard/System_Log').then(module => ({ default: module.System_Log })));
const Neural_Waterfall = lazy(() => import('../components/Neural_Waterfall').then(module => ({ default: module.Neural_Waterfall })));
const Lineage_Stream = lazy(() => import('../components/Lineage_Stream').then(module => ({ default: module.Lineage_Stream })));
const Swarm_Visualizer = lazy(() => import('../components/Swarm_Visualizer').then(module => ({ default: module.Swarm_Visualizer })));
const Agent_Status_Grid = lazy(() => import('../components/dashboard/Agent_Status_Grid').then(module => ({ default: module.Agent_Status_Grid })));
const AgentConfigPanel = lazy(() => import('../components/AgentConfigPanel').then(module => ({ default: module.default })));
const Observability_Sidebar = lazy(() => import('../components/layout/Observability_Sidebar').then(module => ({ default: module.Observability_Sidebar })));
const SovereignChat = lazy(() => import('../components/SovereignChat').then(module => ({ default: module.SovereignChat })));

function Detached_Agent_Status({ tab_id }: { tab_id?: string }) {
    const {
        agents_list,
        assigned_agent_ids,
        available_roles,
        clusters,
        toggle_cluster_active,
        update_agent,
    } = useDashboardData();

    return (
        <div className="h-screen bg-zinc-950 p-6 flex flex-col overflow-hidden">
            <Agent_Status_Grid
                agents={agents_list}
                assigned_agent_ids={assigned_agent_ids}
                available_roles={available_roles}
                clusters={clusters}
                initial_tab_id={tab_id}
                on_skill_trigger={async (agent_id, skill) => {
                    const agent = agents_list.find(a => a.id === agent_id);
                    if (!agent) return;
                    const cluster = clusters.find(c => (c.collaborators || []).includes(agent_id));
                    try {
                        const { tadpole_os_service } = await import('../services/tadpoleos_service');
                        await tadpole_os_service.send_command(
                            agent_id,
                            skill,
                            agent.model,
                            agent.model_config?.provider || 'google',
                            cluster?.id,
                            agent.department,
                            cluster?.budget_usd
                        );
                    } catch (e) {
                        console.error('[DetachedShell] Skill trigger failed:', e);
                    }
                }}
                on_model_change={() => {}}
                on_model_2_change={() => {}}
                on_model_3_change={() => {}}
                on_role_change={() => {}}
                on_configure_click={() => {}}
                handle_agent_update={update_agent}
                on_toggle_cluster={toggle_cluster_active}
            />
        </div>
    );
}

function Detached_Agent_Config({ id }: { id: string | null }) {
    const { agents_list, update_agent } = useDashboardData();
    const agent = agents_list.find(a => a.id === id);

    return (
        <div className="h-screen bg-zinc-950 p-6 flex items-center justify-center">
            <AgentConfigPanel
                agent={agent}
                onClose={() => window.close()}
                onUpdate={update_agent}
                isNew={id === 'new'}
                isDetachedMode={true}
            />
        </div>
    );
}

/**
 * Detached_Shell
 * A unified dispatcher for all native detached windows in the Tadpole OS.
 * Reads the ?type= parameter and renders the appropriate component standalone.
 */
export default function Detached_Shell() {
    const [search_params] = useSearchParams();
    const type = search_params.get('type');
    const id = search_params.get('id');
    const tab_id = search_params.get('tabId');
    const path = search_params.get('path');

    // Mapping of types to components
    const render_content = () => {
        switch (type) {
            case 'chat':
                return <SovereignChat isDetachedView />;
                
            case 'system-log':
                return (
                    <div className="h-screen bg-zinc-950 p-6 flex flex-col">
                        <System_Log is_detached_view />
                    </div>
                );

            case 'trace-stream':
                return (
                    <div className="h-screen bg-zinc-950 p-0 flex flex-col">
                        <Neural_Waterfall is_detached_view />
                    </div>
                );

            case 'lineage-stream':
                return (
                    <div className="h-screen bg-zinc-950 p-0 flex flex-col">
                        <Lineage_Stream is_detached_view />
                    </div>
                );

            case 'swarm-pulse':
                return (
                    <div className="h-screen bg-zinc-950 p-6 flex flex-col overflow-hidden">
                        <Swarm_Visualizer is_detached={true} />
                    </div>
                );

            case 'agent-status':
                return <Detached_Agent_Status tab_id={tab_id || undefined} />;

            case 'agent-config':
                return <Detached_Agent_Config id={id} />;

            case 'tab': {
                const route = get_route_by_path(path || '/dashboard');
                const Component = route.component;
                return (
                    <div className="h-screen bg-zinc-950 p-6 flex flex-row gap-6 overflow-hidden">
                        <div className="flex-1 relative overflow-auto custom-scrollbar">
                            <Error_Boundary name={`Sector: ${route.label}`}>
                                <Suspense fallback={<div className="p-8 text-zinc-500 font-mono text-xs animate-pulse">Initializing Sector...</div>}>
                                    <Component />
                                </Suspense>
                            </Error_Boundary>
                        </div>
                        <Observability_Sidebar is_detached_context />
                    </div>
                );
            }

            default:
                return (
                    <div className="h-screen bg-zinc-950 flex items-center justify-center text-zinc-500 font-mono">
                        INITIALIZING NEURAL LINK... [NULL_TYPE]
                    </div>
                );
        }
    };

    return (
        <div className="min-h-screen bg-zinc-950 text-zinc-100 font-sans antialiased">
            {/* GEO Optimization: Structured Data & Semantic Header */}
            <script type="application/ld+json">
            {JSON.stringify({
              "@context": "https://schema.org",
              "@type": "SoftwareApplication",
              "name": "Tadpole OS Detached Shell",
              "description": "Unified entry point for standalone detached windows. Orchestrates isolated rendering of telemetry, logs, and chat modules.",
              "author": { "@type": "Organization", "name": "Sovereign Engineering" },
              "applicationCategory": "System Shell",
              "operatingSystem": "Tadpole OS"
            })}
            </script>
            <h1 className="sr-only">Tadpole OS Unified Detached Shell & Resource Portal</h1>
            <Suspense fallback={<div className="h-screen bg-zinc-950 flex items-center justify-center animate-pulse font-mono uppercase tracking-widest text-zinc-700 text-xs">Synchronizing Neural Channels...</div>}>
                {render_content()}
            </Suspense>
        </div>
    );
}

// Metadata: [detached_shell]

// Metadata: [detached_shell]
