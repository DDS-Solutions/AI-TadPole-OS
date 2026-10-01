/**
 * @docs ARCHITECTURE:Interface:Missions
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Missions / Cluster_Manager_Modal
 * - **Primary Entrypoints**: `Cluster_Manager_Modal`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import React, { useState, useMemo, useRef, useCallback } from 'react';
import { X, Plus, Trash2, Users, Layers, Edit2 } from 'lucide-react';
import { use_workspace_store, type Team_Cluster_Preset } from '../../stores/workspace_store';
import { useShallow } from 'zustand/react/shallow';
import type { Agent } from '../../types';
import { Confirm_Dialog, Empty_State } from '../ui';
import { Agent_Capability_Inspector } from './cluster_manager/Agent_Capability_Inspector';
import { Preset_Form } from './cluster_manager/Preset_Form';
import {
    type ClusterManagerModalProps,
    type PresetFormData,
    INITIAL_FORM_STATE
} from './cluster_manager/types';

export { type ClusterManagerModalProps };

export const Cluster_Manager_Modal: React.FC<ClusterManagerModalProps> = ({
    isOpen,
    onClose,
    agents
}) => {
    const { team_presets, add_team_preset, update_team_preset, delete_team_preset } = use_workspace_store(
        useShallow(state => ({
            team_presets: state.team_presets || [],
            add_team_preset: state.add_team_preset,
            update_team_preset: state.update_team_preset,
            delete_team_preset: state.delete_team_preset,
        }))
    );

    const [form_data, set_form_data] = useState<PresetFormData>(INITIAL_FORM_STATE);
    const [preset_to_delete, set_preset_to_delete] = useState<Team_Cluster_Preset | null>(null);
    const [show_form, set_show_form] = useState(false);
    const form_ref = useRef<HTMLDivElement | null>(null);

    const [hovered_agent, set_hovered_agent] = useState<Agent | null>(null);
    const leave_timeout_ref = useRef<ReturnType<typeof setTimeout> | null>(null);

    const [agent_search_query, set_agent_search_query] = useState('');

    const reset_form = useCallback(() => {
        set_form_data(INITIAL_FORM_STATE);
        set_show_form(false);
        set_hovered_agent(null);
        if (leave_timeout_ref.current) {
            clearTimeout(leave_timeout_ref.current);
            leave_timeout_ref.current = null;
        }
    }, []);

    const handle_edit = useCallback((preset: Team_Cluster_Preset) => {
        set_form_data({
            editing_id: preset.id,
            name: preset.name,
            description: preset.description || '',
            department: preset.department,
            theme: preset.theme,
            budget_usd: preset.default_budget_usd.toString(),
            badge_label: preset.badge_label,
            selected_agents: preset.collaborators || []
        });
        set_show_form(true);
        setTimeout(() => {
            form_ref.current?.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
        }, 50);
    }, []);

    const filtered_selectable_agents = useMemo(() => {
        if (!agent_search_query.trim()) return agents;
        const q = agent_search_query.toLowerCase();
        return agents.filter(a => 
            a.name.toLowerCase().includes(q) || 
            a.role.toLowerCase().includes(q) ||
            a.department.toLowerCase().includes(q)
        );
    }, [agents, agent_search_query]);

    const agent_map = useMemo(() => {
        return new Map(agents.map(a => [a.id, a]));
    }, [agents]);

    const sorted_presets = useMemo(() => {
        return [...team_presets].sort((a, b) => a.name.localeCompare(b.name));
    }, [team_presets]);

    const preset_cards_grid = useMemo(() => {
        return (
            <div className="grid grid-cols-1 md:grid-cols-2 gap-3 max-h-64 overflow-y-auto custom-scrollbar p-1">
                {team_presets.map(preset => {
                    const is_editing = form_data.editing_id === preset.id;
                    return (
                        <div
                            key={preset.id}
                            className={`p-3.5 rounded-lg border transition-all flex flex-col justify-between space-y-2.5 ${is_editing
                                    ? 'border-cyan-500/80 bg-cyan-950/20 shadow-md ring-1 ring-cyan-500/30'
                                    : 'border-[color:var(--color-border)] bg-[color:var(--color-background)] hover:border-zinc-700'
                                }`}
                        >
                            <div className="flex items-start justify-between">
                                <div>
                                    <div className="flex items-center gap-2">
                                        <h4 className="text-xs font-bold text-white">{preset.name}</h4>
                                        <span className="text-[10px] font-mono px-1.5 py-0.5 rounded bg-cyan-950 text-cyan-300 border border-cyan-500/30">
                                            {preset.badge_label}
                                        </span>
                                    </div>
                                    <p className="text-[11px] text-zinc-400 line-clamp-1 mt-0.5">{preset.description}</p>
                                </div>
                                <div className="flex items-center gap-1 shrink-0">
                                    <button
                                        type="button"
                                        onClick={(e) => {
                                            e.stopPropagation();
                                            handle_edit(preset);
                                        }}
                                        className="p-1 text-zinc-400 hover:text-cyan-300 transition-colors"
                                        title="Edit Preset"
                                    >
                                        <Edit2 size={13} />
                                    </button>
                                    <button
                                        type="button"
                                        onClick={(e) => {
                                            e.stopPropagation();
                                            set_preset_to_delete(preset);
                                        }}
                                        className="p-1 text-zinc-400 hover:text-red-400 transition-colors"
                                        title="Delete Preset"
                                    >
                                        <Trash2 size={13} />
                                    </button>
                                </div>
                            </div>

                            <div className="space-y-1.5 pt-2 border-t border-zinc-800/60">
                                <div className="flex items-center justify-between text-[11px] font-mono">
                                    <span className="text-zinc-500">Selected Agents:</span>
                                    <span className="text-cyan-300 font-bold">{preset.collaborators.length} Agents</span>
                                </div>

                                <div className="flex flex-wrap gap-1 max-h-12 overflow-hidden">
                                    {preset.collaborators.map((agent_id: string) => {
                                        const agent = agent_map.get(agent_id);
                                        return (
                                            <span
                                                key={agent_id}
                                                className="text-[10px] px-1.5 py-0.5 rounded bg-zinc-800/80 text-zinc-300 border border-zinc-700/60 font-mono"
                                            >
                                                👤 {agent?.name || agent_id}
                                            </span>
                                        );
                                    })}
                                </div>

                                <div className="flex items-center justify-between text-[10px] text-zinc-400 pt-1 border-t border-zinc-800/40 font-mono">
                                    <span>Dept: <strong className="text-white">{preset.department}</strong></span>
                                    <span>Budget: <strong className="text-green-400">${preset.default_budget_usd}/mo</strong></span>
                                </div>
                            </div>
                        </div>
                    );
                })}
            </div>
        );
    }, [team_presets, form_data.editing_id, agent_map, handle_edit]);

    if (!isOpen) return null;

    const update_field = <K extends keyof PresetFormData>(field: K, value: PresetFormData[K]) => {
        set_form_data(prev => ({ ...prev, [field]: value }));
    };

    const handle_agent_hover = (agent: Agent) => {
        if (leave_timeout_ref.current) {
            clearTimeout(leave_timeout_ref.current);
            leave_timeout_ref.current = null;
        }
        set_hovered_agent(agent);
    };

    const handle_agent_leave = () => {
        leave_timeout_ref.current = setTimeout(() => {
            set_hovered_agent(null);
            leave_timeout_ref.current = null;
        }, 150);
    };

    const handle_save = () => {
        const { editing_id, name, description, department, theme, budget_usd, badge_label, selected_agents } = form_data;
        if (!name.trim() || !badge_label.trim()) return;

        const parsed_budget = parseFloat(budget_usd);
        const valid_budget = !isNaN(parsed_budget) && parsed_budget > 0 ? parsed_budget : 1000;

        const preset_data: Team_Cluster_Preset = {
            id: editing_id || `preset-${crypto.randomUUID()}`,
            name: name.trim(),
            description: description.trim(),
            department,
            theme,
            default_budget_usd: valid_budget,
            badge_label: badge_label.trim().toUpperCase(),
            collaborators: selected_agents
        };

        if (editing_id) {
            update_team_preset(editing_id, preset_data);
        } else {
            add_team_preset(preset_data);
        }

        reset_form();
    };

    const toggle_agent = (agent_id: string) => {
        set_form_data(prev => ({
            ...prev,
            selected_agents: prev.selected_agents.includes(agent_id)
                ? prev.selected_agents.filter(id => id !== agent_id)
                : [...prev.selected_agents, agent_id]
        }));
    };

    const is_form_valid = Boolean(
        form_data.name.trim() &&
        form_data.badge_label.trim() &&
        !isNaN(parseFloat(form_data.budget_usd)) &&
        parseFloat(form_data.budget_usd) > 0
    );

    return (
        <>
            {/* Destructive Action Confirmation Gate */}
            {preset_to_delete && (
                <Confirm_Dialog
                    is_open={Boolean(preset_to_delete)}
                    title="Delete Team Cluster Preset"
                    message={`Are you sure you want to delete "${preset_to_delete.name}" (${preset_to_delete.badge_label})? This action cannot be undone.`}
                    confirm_label="Delete Preset"
                    variant="danger"
                    on_confirm={() => {
                        delete_team_preset(preset_to_delete.id);
                        set_preset_to_delete(null);
                        if (form_data.editing_id === preset_to_delete.id) {
                            reset_form();
                        }
                    }}
                    on_cancel={() => set_preset_to_delete(null)}
                />
            )}
            <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-zinc-950/70 backdrop-blur-sm animate-in fade-in duration-200">
                {/* Main Studio Modal Card — Always centered, no layout shift */}
                <div className="sovereign-card border-zinc-700/60 bg-[color:var(--color-surface)] w-full max-w-4xl max-h-[90vh] flex flex-col shadow-2xl rounded-xl">
                    <div className="p-4 px-6 border-b border-[color:var(--color-border)] flex items-center justify-between shrink-0 bg-[color:var(--color-background)] rounded-t-xl">
                        <div className="flex items-center gap-2">
                            <Layers className="text-cyan-400" size={20} />
                            <div>
                                <h2 className="sovereign-header-text !text-white text-base">Utility Cluster Configuration Studio</h2>
                                <p className="text-xs text-zinc-400">Configure Mode-Switched Multi-Role Agent Teams with strict operational boundaries.</p>
                            </div>
                        </div>
                        <div className="flex items-center gap-2">
                            <button
                                onClick={onClose}
                                className="p-1.5 rounded-lg hover:bg-zinc-800 text-zinc-400 hover:text-white transition-colors"
                                aria-label="Close studio modal"
                            >
                                <X size={18} />
                            </button>
                        </div>
                    </div>

                    <div className="p-6 overflow-y-auto space-y-6 flex-1">
                        {sorted_presets.length > 0 && (
                            <div className="flex items-center gap-2 overflow-x-auto custom-scrollbar pb-3 border-b border-zinc-800/60">
                                <span className="text-[11px] font-semibold text-zinc-400 uppercase tracking-wider shrink-0 flex items-center gap-1">
                                    ⚡ Quick Presets:
                                </span>
                                {sorted_presets.map(preset => (
                                    <button
                                        type="button"
                                        key={`chip-${preset.id}`}
                                        onClick={() => handle_edit(preset)}
                                        className={`px-2.5 py-1 rounded-full text-xs font-mono transition-all flex items-center gap-1 shrink-0 ${form_data.editing_id === preset.id
                                                ? 'bg-cyan-600 text-white font-bold shadow-md ring-1 ring-cyan-400'
                                                : 'bg-zinc-800 text-cyan-300 hover:bg-zinc-700 border border-zinc-700'
                                            }`}
                                    >
                                        🏷️ {preset.name}
                                    </button>
                                ))}
                            </div>
                        )}

                        <div>
                            <div className="flex items-center justify-between mb-3">
                                <h3 className="text-xs font-bold text-zinc-300 uppercase tracking-wider flex items-center gap-1.5">
                                    <Users size={14} className="text-cyan-400" /> Utility Cluster Presets ({team_presets.length})
                                </h3>
                                <button
                                    type="button"
                                    onClick={() => {
                                        reset_form();
                                        set_show_form(true);
                                    }}
                                    className="text-xs text-cyan-400 hover:text-cyan-300 font-semibold flex items-center gap-1 transition-colors"
                                >
                                    <Plus size={12} /> Add New Preset
                                </button>
                            </div>

                            {team_presets.length === 0 ? (
                                <Empty_State
                                    icon={<Layers className="w-8 h-8 text-cyan-400" />}
                                    title="No Utility Cluster Presets Configured"
                                    description="Create custom multi-role agent clusters to rapidly execute complex workflows with preset boundaries."
                                    action={{
                                        label: "Create Preset",
                                        onClick: () => {
                                            reset_form();
                                            set_show_form(true);
                                        }
                                    }}
                                />
                            ) : (
                                preset_cards_grid
                            )}
                        </div>

                        {show_form && (
                            <Preset_Form
                                form_ref={form_ref}
                                form_data={form_data}
                                update_field={update_field}
                                reset_form={reset_form}
                                handle_save={handle_save}
                                is_form_valid={is_form_valid}
                                agent_search_query={agent_search_query}
                                set_agent_search_query={set_agent_search_query}
                                filtered_selectable_agents={filtered_selectable_agents}
                                toggle_agent={toggle_agent}
                                handle_agent_hover={handle_agent_hover}
                                handle_agent_leave={handle_agent_leave}
                            />
                        )}
                    </div>
                </div>
            </div>

            {/* Agent Capability Hover Inspector — Rendered via Portal to document.body */}
            {hovered_agent && <Agent_Capability_Inspector agent={hovered_agent} />}
        </>
    );
};
