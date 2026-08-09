/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Mode-Switched Utility Cluster Studio Modal.
 * Enables users to create, edit, delete, and configure team cluster presets with strict structural boundaries.
 * 
 * ### 🔍 Debugging & Observability
 * - **Telemetry Link**: Search for `[Cluster_Manager_Modal]` in browser logs.
 * 
 * ### Hover Inspector Architecture
 * The Agent Capability Inspector card is rendered via React Portal to `document.body`,
 * completely outside the modal's DOM tree. This eliminates:
 * - Overflow clipping from the modal container
 * - Layout reflows that trigger mouseLeave/mouseEnter strobe loops
 * - Z-index stacking conflicts with the modal backdrop
 * 
 * The inspector uses `pointer-events-none` because it is a read-only hover tooltip.
 * The mouseLeave handler is debounced (150ms) to absorb any residual re-render jitter.
 */

import React, { useState, useMemo, useRef, useCallback } from 'react';
import { createPortal } from 'react-dom';
import { X, Plus, Trash2, Users, Layers, Check, Cpu, Edit2, Sliders, Search } from 'lucide-react';
import { use_workspace_store, type Team_Cluster_Preset, type Mission_Cluster } from '../../stores/workspace_store';
import type { Agent } from '../../types';
import { parse_active_model_slot } from '../../utils/model_utils';
import { Confirm_Dialog, Empty_State } from '../ui';

interface ClusterManagerModalProps {
    isOpen: boolean;
    onClose: () => void;
    agents: Agent[];
}

interface PresetFormData {
    editing_id: string | null;
    name: string;
    description: string;
    department: Mission_Cluster['department'];
    theme: Mission_Cluster['theme'];
    budget_usd: string;
    badge_label: string;
    selected_agents: string[];
}

const INITIAL_FORM_STATE: PresetFormData = {
    editing_id: null,
    name: '',
    description: '',
    department: 'Engineering',
    theme: 'blue',
    budget_usd: '1500',
    badge_label: '',
    selected_agents: []
};

/**
 * Internal Component: Agent Capability Inspector Card
 * Rendered via Portal to document.body with fixed viewport positioning.
 * Uses pointer-events-none to prevent mouse event interference with the modal.
 */
const Agent_Capability_Inspector: React.FC<{ agent: Agent }> = ({ agent }) => {
    const active_slot = parse_active_model_slot(agent.active_model_slot);
    const slot_config = active_slot === 2 ? agent.model_config2 : active_slot === 3 ? agent.model_config3 : agent.model_config;
    const combined_skills = Array.from(new Set([
        ...(agent.skills || []),
        ...(slot_config?.skills || [])
    ]));
    const combined_workflows = Array.from(new Set([
        ...(agent.workflows || []),
        ...(slot_config?.workflows || [])
    ]));
    const slot1 = agent.model_config?.modelId || agent.model || 'Gemini 3 Flash';
    const slot2 = agent.model_config2?.modelId || agent.model_2 || 'Not Configured';
    const slot3 = agent.model_config3?.modelId || agent.model_3 || 'Not Configured';

    const slots = [
        { slot: 1, label: 'Slot 1 (Primary)', model: slot1 },
        { slot: 2, label: 'Slot 2 (Secondary)', model: slot2 },
        { slot: 3, label: 'Slot 3 (Tertiary)', model: slot3 }
    ];

    /*
     * Positioning strategy:
     * - The Studio Modal is max-w-4xl (896px) centered in the viewport.
     * - Half-width = 448px. Right edge of modal = 50% + 448px.
     * - Inspector anchored at: left = 50% + 448px + 16px gap = calc(50% + 464px)
     * - Top-aligned with studio card: top: 5vh (mirrors the modal's centered 90vh layout).
     * - On narrower viewports (< xl), falls back to right: 16px from viewport edge.
     */

    return createPortal(
        <div className="fixed inset-0 z-[9999] overflow-hidden pointer-events-none">
            <div
                className="absolute w-80 sovereign-card bg-zinc-950/95 border border-cyan-500/50 backdrop-blur-md p-4 rounded-xl shadow-2xl space-y-3 animate-in fade-in zoom-in-95 duration-150 top-[5vh] right-4 2xl:right-auto 2xl:left-[calc(50%+464px)]"
            >
                <div className="flex items-center justify-between border-b border-zinc-800 pb-2">
                    <div>
                        <h4 className="text-xs font-bold text-white flex items-center gap-1.5">
                            👤 {agent.name}
                        </h4>
                        <p className="text-[10px] text-zinc-400 font-mono">
                            {agent.role} • {agent.department}
                        </p>
                    </div>
                    <span className="text-[10px] font-mono px-2 py-0.5 rounded bg-cyan-950 text-cyan-300 border border-cyan-500/30">
                        ID: {agent.id}
                    </span>
                </div>

                <div className="space-y-1">
                    <div className="text-[10px] font-semibold uppercase tracking-wider text-zinc-400 flex items-center gap-1">
                        <Cpu size={10} className="text-cyan-400" /> Slot Configuration:
                    </div>
                    <div className="space-y-1 font-mono text-[10px]">
                        {slots.map(s => {
                            const is_active = s.slot === active_slot;
                            return (
                                <div
                                    key={s.slot}
                                    className={`px-2 py-1 rounded flex items-center justify-between transition-colors ${is_active
                                            ? 'bg-cyan-600/30 text-white font-bold border border-cyan-400/50 shadow-sm'
                                            : 'bg-zinc-900/80 text-zinc-400 border border-zinc-800'
                                        }`}
                                >
                                    <span>{s.label}: {s.model}</span>
                                    {is_active && (
                                        <span className="text-[9px] px-1 rounded bg-cyan-500 text-black font-extrabold uppercase tracking-tight">
                                            ★ ACTIVE
                                        </span>
                                    )}
                                </div>
                            );
                        })}
                    </div>
                </div>

                <div className="space-y-1">
                    <div className="text-[10px] font-semibold uppercase tracking-wider text-zinc-400 flex items-center justify-between">
                        <span>🛠️ Skills ({combined_skills.length}):</span>
                    </div>
                    {combined_skills.length === 0 ? (
                        <p className="text-[10px] text-zinc-500 italic">No skills assigned</p>
                    ) : (
                        <div className="flex flex-wrap gap-1">
                            {combined_skills.map(skill => (
                                <span key={skill} className="text-[9px] font-mono px-1.5 py-0.5 rounded bg-cyan-950/80 text-cyan-300 border border-cyan-500/30">
                                    {skill}
                                </span>
                            ))}
                        </div>
                    )}
                </div>

                <div className="space-y-1">
                    <div className="text-[10px] font-semibold uppercase tracking-wider text-zinc-400 flex items-center justify-between">
                        <span>⚡ Workflows ({combined_workflows.length}):</span>
                    </div>
                    {combined_workflows.length === 0 ? (
                        <p className="text-[10px] text-zinc-500 italic">No workflows assigned</p>
                    ) : (
                        <div className="flex flex-wrap gap-1">
                            {combined_workflows.map(wf => (
                                <span key={wf} className="text-[9px] font-mono px-1.5 py-0.5 rounded bg-amber-950/80 text-amber-300 border border-amber-500/30">
                                    {wf}
                                </span>
                            ))}
                        </div>
                    )}
                </div>
            </div>
        </div>,
        document.body
    );
};

export const Cluster_Manager_Modal: React.FC<ClusterManagerModalProps> = ({
    isOpen,
    onClose,
    agents
}) => {
    const team_presets = use_workspace_store(state => state.team_presets || []);
    const add_team_preset = use_workspace_store(state => state.add_team_preset);
    const update_team_preset = use_workspace_store(state => state.update_team_preset);
    const delete_team_preset = use_workspace_store(state => state.delete_team_preset);

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
            <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/70 backdrop-blur-sm animate-in fade-in duration-200">
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
                            <div ref={form_ref} className="p-4 rounded-xl bg-[color:var(--color-background)] border border-cyan-500/40 space-y-4 animate-in fade-in slide-in-from-top-2 duration-200 shadow-xl">
                                <div className="flex items-center justify-between border-b border-zinc-800 pb-2">
                                    <h3 className="text-xs font-bold text-white flex items-center gap-1.5">
                                        <Sliders size={14} className="text-cyan-400" />
                                        {form_data.editing_id ? 'Edit Team Cluster Preset' : 'Create New Utility Cluster Preset'}
                                    </h3>
                                    <button
                                        type="button"
                                        onClick={reset_form}
                                        className="text-[11px] text-zinc-400 hover:text-white"
                                    >
                                        Cancel
                                    </button>
                                </div>

                                <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                                    <div>
                                        <label className="block text-[11px] font-semibold text-zinc-400 mb-1">Preset Name</label>
                                        <input
                                            type="text"
                                            value={form_data.name}
                                            onChange={(e) => update_field('name', e.target.value)}
                                            placeholder="e.g., Code Audit & Security Cluster"
                                            className="w-full bg-[color:var(--color-surface)] border border-[color:var(--color-border)] rounded p-2 text-xs text-white font-mono focus:outline-none focus:border-cyan-500"
                                        />
                                    </div>
                                    <div>
                                        <label className="block text-[11px] font-semibold text-zinc-400 mb-1">Badge Tag Label</label>
                                        <input
                                            type="text"
                                            value={form_data.badge_label}
                                            onChange={(e) => update_field('badge_label', e.target.value)}
                                            placeholder="e.g., AUDIT-PRO"
                                            className="w-full bg-[color:var(--color-surface)] border border-[color:var(--color-border)] rounded p-2 text-xs text-white font-mono focus:outline-none focus:border-cyan-500 uppercase"
                                        />
                                    </div>
                                </div>

                                <div>
                                    <label className="block text-[11px] font-semibold text-zinc-400 mb-1">Description</label>
                                    <input
                                        type="text"
                                        value={form_data.description}
                                        onChange={(e) => update_field('description', e.target.value)}
                                        placeholder="Brief operational objective for this multi-role cluster team."
                                        className="w-full bg-[color:var(--color-surface)] border border-[color:var(--color-border)] rounded p-2 text-xs text-white font-mono focus:outline-none focus:border-cyan-500"
                                    />
                                </div>

                                <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                                    <div>
                                        <label className="block text-[11px] font-semibold text-zinc-400 mb-1">Department</label>
                                        <select
                                            value={form_data.department}
                                            onChange={(e) => update_field('department', e.target.value as Mission_Cluster['department'])}
                                            className="w-full bg-[color:var(--color-surface)] border border-[color:var(--color-border)] rounded p-2 text-xs text-white font-mono focus:outline-none focus:border-cyan-500"
                                        >
                                            <option value="Engineering">Engineering</option>
                                            <option value="Quality Assurance">Quality Assurance</option>
                                            <option value="Security">Security</option>
                                            <option value="Product">Product</option>
                                            <option value="Research">Research</option>
                                            <option value="Operations">Operations</option>
                                            <option value="Executive">Executive</option>
                                        </select>
                                    </div>
                                    <div>
                                        <label className="block text-[11px] font-semibold text-zinc-400 mb-1">Monthly Budget Boundary ($ USD)</label>
                                        <input
                                            type="number"
                                            min="1"
                                            step="5"
                                            value={form_data.budget_usd}
                                            onChange={(e) => update_field('budget_usd', e.target.value)}
                                            className="w-full bg-[color:var(--color-surface)] border border-[color:var(--color-border)] rounded p-2 text-xs text-white font-mono focus:outline-none focus:border-cyan-500"
                                        />
                                    </div>
                                </div>

                                {/* Agent Selector Chips */}
                                <div>
                                    <div className="flex items-center justify-between mb-1.5">
                                        <label className="block text-[11px] font-semibold text-zinc-400">
                                            Select Multi-Role Agents ({form_data.selected_agents.length} Selected)
                                        </label>
                                        <div className="relative w-44">
                                            <Search size={11} className="absolute left-2 top-1/2 -translate-y-1/2 text-zinc-500 pointer-events-none" />
                                            <input
                                                type="text"
                                                value={agent_search_query}
                                                onChange={(e) => set_agent_search_query(e.target.value)}
                                                placeholder="Filter agents..."
                                                className="w-full bg-[color:var(--color-surface)] border border-[color:var(--color-border)] rounded pl-6 pr-2 py-0.5 text-[10px] text-white font-mono focus:outline-none focus:border-cyan-500"
                                            />
                                        </div>
                                    </div>
                                    <div className="flex flex-wrap gap-1.5 max-h-36 overflow-y-auto p-2 bg-[color:var(--color-surface)] border border-[color:var(--color-border)] rounded-lg">
                                        {filtered_selectable_agents.map(agent => {
                                            const is_selected = form_data.selected_agents.includes(agent.id);
                                            return (
                                                <button
                                                    type="button"
                                                    key={agent.id}
                                                    aria-pressed={is_selected}
                                                    onClick={() => toggle_agent(agent.id)}
                                                    onMouseEnter={() => handle_agent_hover(agent)}
                                                    onMouseLeave={handle_agent_leave}
                                                    onFocus={() => handle_agent_hover(agent)}
                                                    onBlur={handle_agent_leave}
                                                    className={`px-2 py-1 rounded text-xs transition-all flex items-center gap-1 ${is_selected
                                                            ? 'bg-cyan-600 text-white font-medium shadow-sm'
                                                            : 'bg-zinc-800 text-zinc-400 hover:text-zinc-200 border border-zinc-700'
                                                        }`}
                                                >
                                                    {is_selected && <Check size={10} />}
                                                    {agent.name}
                                                </button>
                                            );
                                        })}
                                        {filtered_selectable_agents.length === 0 && (
                                            <span className="text-xs text-zinc-500 py-2 px-1 font-mono italic">No agents match "{agent_search_query}"</span>
                                        )}
                                    </div>
                                </div>

                                <div className="flex items-center justify-end pt-3 border-t border-zinc-800/80">
                                    <button
                                        type="button"
                                        onClick={handle_save}
                                        disabled={!is_form_valid}
                                        className="px-4 py-2 rounded-lg bg-cyan-600 hover:bg-cyan-500 disabled:opacity-50 text-white font-semibold text-xs transition-all flex items-center gap-1.5 shadow-sm"
                                    >
                                        <Check size={14} /> {form_data.editing_id ? 'Save Changes' : 'Create Cluster Preset'}
                                    </button>
                                </div>
                            </div>
                        )}
                    </div>
                </div>
            </div>

            {/* Agent Capability Hover Inspector — Rendered via Portal to document.body */}
            {hovered_agent && <Agent_Capability_Inspector agent={hovered_agent} />}
        </>
    );
};
