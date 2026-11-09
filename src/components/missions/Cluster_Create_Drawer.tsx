/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Missions / Cluster_Create_Drawer
 * - **Primary Entrypoints**: `Cluster_Create_Drawer`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import React, { useState } from 'react';
import { Target, Shield, Layers, Check } from 'lucide-react';
import { Tooltip } from '../ui';
import type { Mission_Cluster, Team_Cluster_Preset } from '../../stores/workspace_store';
import type { Agent } from '../../types';
import { i18n } from '../../i18n';

export interface ClusterCreateDrawerProps {
    isOpen: boolean;
    onClose: () => void;
    onCreate: (cluster: Partial<Mission_Cluster>) => void;
    departments: string[];
    team_presets: Team_Cluster_Preset[];
    agents: Agent[];
}

const INITIAL_NEW_CLUSTER_STATE = {
    name: '',
    department: 'Engineering' as Mission_Cluster['department'],
    theme: 'blue' as Mission_Cluster['theme'],
    path: '/workspaces/new-mission',
    collaborators: [] as string[],
    privacy_mode: false,
    is_team: false,
    team_badge: ''
};

type NewClusterDraft = typeof INITIAL_NEW_CLUSTER_STATE;

interface ClusterCoreFieldsProps {
    new_cluster: NewClusterDraft;
    new_mission_budget: string;
    departments: string[];
    on_cluster_change: (next: NewClusterDraft) => void;
    on_budget_change: (val: string) => void;
}

const ClusterCoreFields: React.FC<ClusterCoreFieldsProps> = ({
    new_cluster,
    new_mission_budget,
    departments,
    on_cluster_change,
    on_budget_change,
}) => (
    <>
        <input
            className="w-full bg-[color:var(--color-background)] border border-[color:var(--color-border)] rounded p-2 text-xs text-zinc-200"
            placeholder={i18n.t('missions.placeholder_name')}
            aria-label={i18n.t('missions.placeholder_name')}
            value={new_cluster.name}
            onChange={e => on_cluster_change({ ...new_cluster, name: e.target.value })}
        />
        <div className="space-y-1">
            <div className="flex items-center justify-between">
                <label className="text-[10px] uppercase text-zinc-500 font-bold tracking-wider">{i18n.t('missions.label_budget')}</label>
                <Tooltip content={i18n.t('missions.tooltip_budget')} position="top">
                    <Target size={10} className="text-zinc-600 cursor-help" />
                </Tooltip>
            </div>
            <div className="relative">
                <span className="absolute left-2 top-1/2 -translate-y-1/2 text-zinc-500 font-mono text-[10px]">{i18n.t('common_units.currency')}</span>
                <input
                    type="number"
                    step="0.01"
                    className="w-full bg-[color:var(--color-background)] border border-[color:var(--color-border)] rounded p-2 pl-4 text-xs text-zinc-200 font-mono"
                    placeholder={i18n.t('common_units.placeholder_budget')}
                    aria-label={i18n.t('missions.label_budget')}
                    value={new_mission_budget}
                    onChange={e => on_budget_change(e.target.value)}
                />
            </div>
        </div>
        <div className="flex gap-2 items-end">
            <div className="flex-1 space-y-1">
                <label className="text-[10px] uppercase text-zinc-500 font-bold tracking-wider">{i18n.t('missions.label_dept')}</label>
                <select
                    className="w-full bg-[color:var(--color-background)] border border-[color:var(--color-border)] rounded p-2 text-xs text-zinc-300"
                    aria-label={i18n.t('missions.label_dept')}
                    value={new_cluster.department}
                    onChange={e => on_cluster_change({ ...new_cluster, department: e.target.value as Mission_Cluster['department'] })}
                >
                    {departments.map((dept) => (
                        <option key={dept} value={dept} className="bg-[color:var(--color-background)] text-zinc-300">{dept}</option>
                    ))}
                </select>
            </div>
        </div>
        <div className="flex items-center justify-between p-2 rounded bg-zinc-900/60 border border-zinc-800">
            <div className="flex items-center gap-1.5 text-xs text-zinc-300 font-bold">
                <Shield size={12} className={new_cluster.privacy_mode ? 'text-green-400 animate-pulse' : 'text-zinc-500'} />
                <span>Local-Only Air-Gap Mode</span>
            </div>
            <input
                type="checkbox"
                className="w-4 h-4 rounded border-zinc-700 bg-zinc-800 text-green-500 cursor-pointer"
                checked={!!new_cluster.privacy_mode}
                onChange={e => on_cluster_change({ ...new_cluster, privacy_mode: e.target.checked })}
            />
        </div>
    </>
);

interface TeamPresetPickerProps {
    show_preset_picker: boolean;
    team_presets: Team_Cluster_Preset[];
    on_toggle_picker: () => void;
    on_select_preset: (preset: Team_Cluster_Preset) => void;
}

const TeamPresetPicker: React.FC<TeamPresetPickerProps> = ({
    show_preset_picker,
    team_presets,
    on_toggle_picker,
    on_select_preset,
}) => (
    <div className="pt-1">
        <button
            type="button"
            onClick={on_toggle_picker}
            className="w-full py-1.5 px-2 rounded border border-cyan-500/40 bg-cyan-500/10 hover:bg-cyan-500/20 text-cyan-300 text-xs font-bold transition-all flex items-center justify-center gap-1.5"
        >
            <Layers size={12} /> {show_preset_picker ? 'Hide Team Cluster Presets' : '+ ADD TEAM CLUSTER PRESET'}
        </button>

        {show_preset_picker && (
            <div className="mt-2 p-2 bg-zinc-900/80 border border-cyan-500/30 rounded-lg space-y-2 max-h-40 overflow-y-auto">
                <span className="text-[10px] uppercase font-bold text-cyan-400 tracking-wider block mb-1">Select Predefined Utility Cluster</span>
                {team_presets.map(preset => (
                    <button
                        type="button"
                        key={preset.id}
                        onClick={() => on_select_preset(preset)}
                        className="w-full text-left p-2 rounded bg-zinc-800/80 hover:bg-cyan-900/30 border border-zinc-700 hover:border-cyan-500/50 transition-all flex items-center justify-between group"
                    >
                        <div>
                            <div className="flex items-center gap-1.5">
                                <span className="text-[10px] font-mono text-cyan-300 font-bold">🏷️ {preset.badge_label}</span>
                                <span className="text-xs font-bold text-white group-hover:text-cyan-200">{preset.name}</span>
                            </div>
                            <p className="text-[10px] text-zinc-400 line-clamp-1">{preset.description}</p>
                        </div>
                        <span className="text-[10px] font-mono text-green-400 font-bold shrink-0 ml-2">${preset.default_budget_usd}</span>
                    </button>
                ))}
            </div>
        )}
    </div>
);

interface CollaboratorChipProps {
    agent: Agent;
    is_checked: boolean;
    on_toggle: (id: string) => void;
}

const CollaboratorChip: React.FC<CollaboratorChipProps> = ({ agent, is_checked, on_toggle }) => {
    const active_cls = is_checked
        ? 'bg-cyan-600/80 text-white font-medium border border-cyan-500'
        : 'bg-zinc-800 text-zinc-400 border border-zinc-700 hover:text-zinc-200';
    return (
        <button
            type="button"
            onClick={() => on_toggle(agent.id)}
            className={`px-1.5 py-0.5 rounded text-[10px] transition-all flex items-center gap-1 ${active_cls}`}
        >
            {is_checked && <Check size={8} />}
            <span>{agent.name}</span>
        </button>
    );
};

interface CollaboratorsSelectorProps {
    agents: Agent[];
    selected_ids: string[];
    on_toggle: (agent_id: string) => void;
}

const CollaboratorsSelector: React.FC<CollaboratorsSelectorProps> = ({ agents, selected_ids, on_toggle }) => (
    <div className="space-y-1">
        <label className="text-[10px] uppercase text-zinc-400 font-bold tracking-wider block">Multi-Role Collaborators ({selected_ids.length} Selected)</label>
        <div className="flex flex-wrap gap-1 max-h-28 overflow-y-auto p-1.5 bg-[color:var(--color-background)] border border-[color:var(--color-border)] rounded">
            {agents.map(agent => (
                <CollaboratorChip
                    key={agent.id}
                    agent={agent}
                    is_checked={selected_ids.includes(agent.id)}
                    on_toggle={on_toggle}
                />
            ))}
        </div>
    </div>
);

function build_preset_cluster_draft(preset: Team_Cluster_Preset): NewClusterDraft {
    return {
        name: preset.name,
        department: preset.department,
        theme: preset.theme,
        path: `/workspaces/${preset.id}-${crypto.randomUUID()}`,
        collaborators: preset.collaborators || [],
        privacy_mode: false,
        is_team: true,
        team_badge: preset.badge_label,
    };
}

function toggle_collaborator_id(collaborators: string[], agent_id: string): string[] {
    return collaborators.includes(agent_id)
        ? collaborators.filter(id => id !== agent_id)
        : [...collaborators, agent_id];
}

export const Cluster_Create_Drawer: React.FC<ClusterCreateDrawerProps> = ({ isOpen, onClose, onCreate, departments, team_presets, agents }) => {
    const [new_cluster, set_new_cluster] = useState(INITIAL_NEW_CLUSTER_STATE);
    const [new_mission_budget, set_new_mission_budget] = useState('1.00');
    const [show_preset_picker, set_show_preset_picker] = useState(false);

    if (!isOpen) return null;

    const reset_and_close = () => {
        set_new_cluster(INITIAL_NEW_CLUSTER_STATE);
        set_new_mission_budget('1.00');
        set_show_preset_picker(false);
        onClose();
    };

    const handle_select_preset = (preset: Team_Cluster_Preset) => {
        set_new_cluster(build_preset_cluster_draft(preset));
        set_new_mission_budget(preset.default_budget_usd.toString());
        set_show_preset_picker(false);
    };

    const toggle_collaborator = (agent_id: string) => {
        set_new_cluster(prev => ({ ...prev, collaborators: toggle_collaborator_id(prev.collaborators, agent_id) }));
    };

    const handle_submit = () => {
        if (!new_cluster.name) return;
        onCreate({ ...new_cluster, budget_usd: parseFloat(new_mission_budget) || 1.00 });
        reset_and_close();
    };

    return (
        <div className="shrink-0 sovereign-card animate-in slide-in-from-top-2 border-green-500/30 bg-green-500/5 shadow-xl relative z-20">
            <h4 className="sovereign-header-text !text-green-400 mb-3">{i18n.t('missions.header_create_cluster')}</h4>
            <div className="space-y-3">
                <ClusterCoreFields
                    new_cluster={new_cluster}
                    new_mission_budget={new_mission_budget}
                    departments={departments}
                    on_cluster_change={set_new_cluster}
                    on_budget_change={set_new_mission_budget}
                />
                <TeamPresetPicker
                    show_preset_picker={show_preset_picker}
                    team_presets={team_presets}
                    on_toggle_picker={() => set_show_preset_picker(prev => !prev)}
                    on_select_preset={handle_select_preset}
                />
                <CollaboratorsSelector agents={agents} selected_ids={new_cluster.collaborators} on_toggle={toggle_collaborator} />
                <div className="flex justify-end gap-2 pt-1">
                    <button type="button" onClick={reset_and_close} className="h-[34px] px-3 bg-zinc-800 hover:bg-zinc-700 text-zinc-300 rounded text-xs font-bold uppercase transition-colors">Cancel</button>
                    <button type="button" onClick={handle_submit} disabled={!new_cluster.name} className="h-[34px] px-4 bg-green-600 hover:bg-green-500 text-white rounded text-xs font-bold uppercase disabled:opacity-50 transition-colors shadow-md shadow-green-950/40">{i18n.t('missions.btn_create')}</button>
                </div>
            </div>
        </div>
    );
};

