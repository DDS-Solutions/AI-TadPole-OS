/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Standup_Target_Selector]` in observability traces.
 */

import { Users, Target, ChevronDown } from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';
import type { Agent } from '../../types';
import type { Mission_Cluster } from '../../stores/workspace_store';

interface StandupTargetSelectorProps {
    target_type: 'agent' | 'cluster';
    selected_target_id: string;
    agents: Agent[];
    clusters: Mission_Cluster[];
    set_target_type: (type: 'agent' | 'cluster') => void;
    set_selected_target_id: (id: string) => void;
}

export function Standup_Target_Selector({
    target_type,
    selected_target_id,
    agents,
    clusters,
    set_target_type,
    set_selected_target_id
}: StandupTargetSelectorProps) {
    return (
        <div className="w-full max-w-sm flex flex-col gap-3 p-4 bg-[color:var(--color-surface)]/50 border border-[color:var(--color-border)] rounded-2xl backdrop-blur-sm">
            <div className="flex bg-[color:var(--color-background)] p-1 rounded-lg border border-[color:var(--color-border)]">
                <Tooltip content={i18n.t('standups.tooltip_agent')} position="top">
                    <button
                        onClick={() => { set_target_type('agent'); set_selected_target_id(agents[0]?.id || ''); }}
                        className={`flex-1 flex items-center justify-center gap-2 py-1.5 text-[10px] font-bold uppercase tracking-widest transition-all rounded-md ${target_type === 'agent' ? 'bg-zinc-800 text-zinc-100' : 'text-zinc-500 hover:text-zinc-300'}`}
                    >
                        <Users size={12} /> {i18n.t('standups.label_agent_node')}
                    </button>
                </Tooltip>
                <Tooltip content={i18n.t('standups.tooltip_cluster')} position="top">
                    <button
                        onClick={() => { set_target_type('cluster'); set_selected_target_id(clusters[0]?.id || ''); }}
                        className={`flex-1 flex items-center justify-center gap-2 py-1.5 text-[10px] font-bold uppercase tracking-widest transition-all rounded-md ${target_type === 'cluster' ? 'bg-zinc-800 text-zinc-100' : 'text-zinc-500 hover:text-zinc-300'}`}
                    >
                        <Target size={12} /> {i18n.t('standups.label_mission_cluster')}
                    </button>
                </Tooltip>
            </div>

            <div className="relative">
                <label htmlFor="target-select" className="sr-only">{i18n.t('standups.placeholder_select_target')}</label>
                <select
                    id="target-select"
                    value={selected_target_id}
                    onChange={(e) => set_selected_target_id(e.target.value)}
                    className="w-full bg-[color:var(--color-background)] border border-[color:var(--color-border)] rounded-lg px-4 py-2.5 text-xs text-zinc-200 focus:outline-none focus:border-zinc-700 transition-all font-mono appearance-none cursor-pointer"
                >
                    {target_type === 'agent' ? (
                        agents.map(a => <option key={a.id} value={a.id}>{a.name.toUpperCase()}</option>)
                    ) : (
                        clusters.map(c => <option key={c.id} value={c.id}>{c.name.toUpperCase()}</option>)
                    )}
                </select>
                <ChevronDown size={14} className="absolute right-3 top-1/2 -translate-y-1/2 text-zinc-600 pointer-events-none" />
            </div>
        </div>
    );
}

// Metadata: [Standup_Target_Selector]
