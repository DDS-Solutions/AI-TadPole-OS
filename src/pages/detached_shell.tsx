/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Pages / detached_shell
 * - **Primary Entrypoints**: `Detached_Shell`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: `[DetachedShell]`
 * - **Witness Tests**: none declared
 */

import { Suspense, lazy, useState, useMemo } from 'react';
import { useSearchParams } from 'react-router-dom';
import { useDashboardData } from '../hooks/use_dashboard_data';
import { useWindowAutoScale } from '../hooks/useWindowAutoScale';
import { get_route_by_path } from '../constants/routes';
import Error_Boundary from '../components/Error_Boundary';
import { i18n } from '../i18n';

// lazy-loaded components to match the main app's architecture
const System_Log = lazy(() => import('../components/dashboard/System_Log').then(module => ({ default: module.System_Log })));
const Neural_Waterfall = lazy(() => import('../components/Neural_Waterfall').then(module => ({ default: module.Neural_Waterfall })));
const Lineage_Stream = lazy(() => import('../components/Lineage_Stream').then(module => ({ default: module.Lineage_Stream })));
const Swarm_Visualizer = lazy(() => import('../components/Swarm_Visualizer').then(module => ({ default: module.Swarm_Visualizer })));
const Agent_Status_Grid = lazy(() => import('../components/dashboard/Agent_Status_Grid').then(module => ({ default: module.Agent_Status_Grid })));
const AgentConfigPanel = lazy(() => import('../components/AgentConfigPanel').then(module => ({ default: module.default })));
const SovereignChat = lazy(() => import('../components/SovereignChat').then(module => ({ default: module.SovereignChat })));

import { resolve_provider } from '../utils/model_utils';

function Detached_Agent_Status({ tab_id }: { tab_id?: string }) {
    const {
        agents_list,
        assigned_agent_ids,
        available_roles,
        clusters,
        toggle_cluster_active,
        update_agent,
        delete_agent
    } = useDashboardData();

    const [config_agent_id, set_config_agent_id] = useState<string | null>(null);

    const config_agent = useMemo(() => {
        if (!config_agent_id) return undefined;
        return agents_list.find(a => a.id === config_agent_id);
    }, [config_agent_id, agents_list]);

    const handle_model_slot_change = (
        agent_id: string,
        new_model: string,
        model_key: 'model' | 'model_2' | 'model_3',
        config_key: 'model_config' | 'model_config2' | 'model_config3'
    ) => {
        const agent = agents_list.find(a => a.id === agent_id);
        const provider = resolve_provider(new_model);
        const current_config = agent?.[config_key];
        const updated_config = current_config
            ? { ...current_config, modelId: new_model, provider }
            : { modelId: new_model, provider };
        update_agent(agent_id, { [model_key]: new_model, [config_key]: updated_config });
    };

    return (
        <div className="h-screen bg-[color:var(--color-background)] p-6 flex flex-col overflow-hidden">
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
                on_model_change={(id, model) => handle_model_slot_change(id, model, 'model', 'model_config')}
                on_model_2_change={(id, model) => handle_model_slot_change(id, model, 'model_2', 'model_config2')}
                on_model_3_change={(id, model) => handle_model_slot_change(id, model, 'model_3', 'model_config3')}
                on_role_change={() => {}}
                on_configure_click={(id) => set_config_agent_id(id)}
                handle_agent_update={update_agent}
                on_toggle_cluster={toggle_cluster_active}
            />

            {config_agent_id && (
                <AgentConfigPanel
                    agent={config_agent}
                    onClose={() => set_config_agent_id(null)}
                    onUpdate={update_agent}
                    onDelete={async (id) => {
                        await delete_agent(id);
                        set_config_agent_id(null);
                    }}
                    isNew={config_agent_id === 'new'}
                />
            )}
        </div>
    );
}


function Detached_Agent_Config({ id }: { id: string | null }) {
    const { agents_list, update_agent, delete_agent } = useDashboardData();
    const agent = agents_list.find(a => a.id === id);

    const handle_close = async () => {
        const is_tauri = !!(window as unknown as { __TAURI_INTERNALS__: unknown }).__TAURI_INTERNALS__;
        if (is_tauri) {
            try {
                const { getCurrentWindow } = await import('@tauri-apps/api/window');
                await getCurrentWindow().close();
            } catch (err) {
                console.debug('[DetachedShell] Tauri window close failed, falling back to window.close():', err);
                window.close();
            }
        } else {
            window.close();
        }
    };

    return (
        <div className="h-screen bg-[color:var(--color-background)] p-6 flex items-center justify-center">
            <AgentConfigPanel
                agent={agent}
                onClose={handle_close}
                onUpdate={update_agent}
                onDelete={async (agent_id) => {
                    await delete_agent(agent_id);
                    await handle_close();
                }}
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

    // Dynamic auto-scaling based on window size for native detached webviews
    useWindowAutoScale(1200, 800, 0.55, 1.0);

    // Mapping of types to components
    const render_content = () => {
        switch (type) {
            case 'chat':
                return <SovereignChat isDetachedView />;
                
            case 'system-log':
                return (
                    <div className="h-screen bg-[color:var(--color-background)] p-6 flex flex-col">
                        <System_Log is_detached_view />
                    </div>
                );

            case 'trace-stream':
                return (
                    <div className="h-screen bg-[color:var(--color-background)] p-0 flex flex-col">
                        <Neural_Waterfall is_detached_view />
                    </div>
                );

            case 'lineage-stream':
                return (
                    <div className="h-screen bg-[color:var(--color-background)] p-0 flex flex-col">
                        <Lineage_Stream is_detached_view />
                    </div>
                );

            case 'swarm-pulse':
                return (
                    <div className="h-screen bg-[color:var(--color-background)] p-6 flex flex-col overflow-hidden">
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
                    <div className="h-screen w-full bg-[color:var(--color-background)] p-4 sm:p-6 flex flex-col overflow-hidden">
                        <div className="flex-1 relative overflow-auto custom-scrollbar w-full h-full">
                            <Error_Boundary name={`Sector: ${route.label}`}>
                                <Suspense fallback={<div className="p-8 text-zinc-500 font-mono text-xs animate-pulse">{i18n.t('common.initializing_sector')}</div>}>
                                    <Component />
                                </Suspense>
                            </Error_Boundary>
                        </div>
                    </div>
                );
            }

            default:
                return (
                    <div className="h-screen bg-[color:var(--color-background)] flex items-center justify-center text-zinc-500 font-mono">
                        {i18n.t('common.initializing_neural_link')}
                    </div>
                );
        }
    };

    return (
        <div className="min-h-screen bg-[color:var(--color-background)] text-zinc-100 font-sans antialiased">
            <h1 className="sr-only">{i18n.t('common.detached_shell_title')}</h1>
            <Suspense fallback={<div className="h-screen bg-[color:var(--color-background)] flex items-center justify-center animate-pulse font-mono uppercase tracking-widest text-zinc-700 text-xs">{i18n.t('common.synchronizing_channels')}</div>}>
                {render_content()}
            </Suspense>
        </div>
    );
}
