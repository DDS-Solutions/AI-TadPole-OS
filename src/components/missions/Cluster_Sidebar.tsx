/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Missions / Cluster_Sidebar
 * - **Primary Entrypoints**: `Cluster_Sidebar`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import React, { useState, useMemo } from 'react';
import { Plus, Sliders } from 'lucide-react';
import { Tooltip } from '../ui';
import { use_workspace_store, type Mission_Cluster } from '../../stores/workspace_store';
import { use_department_store } from '../../stores/department_store';
import type { Agent } from '../../types';
import { i18n } from '../../i18n';
import { Cluster_Manager_Modal } from './Cluster_Manager_Modal';
import { Cluster_Create_Drawer } from './Cluster_Create_Drawer';
import { Cluster_Card } from './Cluster_Card';

interface ClusterSidebarProps {
    clusters: Mission_Cluster[];
    selected_cluster_id: string | null;
    agents: Agent[];
    on_select_cluster: (id: string) => void;
    on_create_cluster: (cluster: Partial<Mission_Cluster>) => void;
    on_delete_cluster: (id: string) => void;
    on_toggle_active: (id: string) => void;
    on_update_department: (id: string, dept: Mission_Cluster['department']) => void;
    on_update_budget: (id: string, budget: number) => void;
}

export const Cluster_Sidebar: React.FC<ClusterSidebarProps> = ({
    clusters,
    selected_cluster_id,
    agents,
    on_select_cluster,
    on_create_cluster,
    on_delete_cluster,
    on_toggle_active,
    on_update_department,
    on_update_budget
}) => {
    const toggle_cluster_privacy = use_workspace_store(state => state.toggle_cluster_privacy);
    const team_presets = use_workspace_store(state => state.team_presets || []);
    const { departments } = use_department_store();

    const [show_manager_modal, set_show_manager_modal] = useState(false);
    const [show_create_drawer, set_show_create_drawer] = useState(false);

    // O(1) Fast Agent Lookup Map
    const agent_map = useMemo(() => {
        return new Map(agents.map(a => [a.id, a]));
    }, [agents]);

    return (
        <div className="md:col-span-1 flex flex-col gap-3 h-full min-h-0 overflow-hidden pr-2 pl-1">
            <Cluster_Manager_Modal
                isOpen={show_manager_modal}
                onClose={() => set_show_manager_modal(false)}
                agents={agents}
            />

            <div className="flex items-center justify-between px-2 shrink-0 gap-1">
                <h3 className="sovereign-header-text truncate">{i18n.t('missions.header_active_clusters')}</h3>
                <div className="flex items-center gap-1 shrink-0">
                    <Tooltip content="Manage Mode-Switched Utility Clusters & Team Presets" position="left">
                        <button
                            onClick={() => set_show_manager_modal(true)}
                            className="p-1 px-2 rounded-lg border border-[color:var(--color-border)] bg-[color:var(--color-surface)] text-xs font-bold text-cyan-400 hover:text-cyan-300 hover:border-cyan-500/50 transition-all flex items-center gap-1"
                        >
                            <Sliders size={10} /> CLUSTERS
                        </button>
                    </Tooltip>
                    <Tooltip content={i18n.t('missions.tooltip_create_cluster')} position="left">
                        <button
                            onClick={() => set_show_create_drawer(prev => !prev)}
                            className="p-1 px-2 rounded-lg border border-[color:var(--color-border)] bg-[color:var(--color-surface)] text-xs font-bold text-zinc-400 hover:text-white hover:border-zinc-700 transition-all flex items-center gap-1"
                        >
                            <Plus size={10} /> {i18n.t('missions.btn_new_mission')}
                        </button>
                    </Tooltip>
                </div>
            </div>

            <Cluster_Create_Drawer
                isOpen={show_create_drawer}
                onClose={() => set_show_create_drawer(false)}
                onCreate={on_create_cluster}
                departments={departments}
                team_presets={team_presets}
                agents={agents}
            />

            {/* Dedicated scrollable container for Cluster Cards */}
            <div className="flex-1 overflow-y-auto custom-scrollbar space-y-3 min-h-0 pr-1 pb-4">
                {clusters.map(cluster => (
                    <Cluster_Card
                        key={cluster.id}
                        cluster={cluster}
                        is_selected={selected_cluster_id === cluster.id}
                        agent_map={agent_map}
                        departments={departments}
                        on_select_cluster={on_select_cluster}
                        on_delete_cluster={on_delete_cluster}
                        on_toggle_active={on_toggle_active}
                        on_update_department={on_update_department}
                        on_update_budget={on_update_budget}
                        toggle_cluster_privacy={toggle_cluster_privacy}
                    />
                ))}
            </div>
        </div>
    );
};
