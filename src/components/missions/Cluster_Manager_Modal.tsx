/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Mode-Switched Utility Cluster Studio Modal.
 * Enables users to create, edit, delete, and configure team cluster presets with strict structural boundaries.
 * 
 * ### 🔍 Debugging & Observability
 * - **Telemetry Link**: Search for `[Cluster_Manager_Modal]` in browser logs.
 */

import React, { useState, useMemo } from 'react';
import { X, Plus, Trash2, Shield, Users, Layers, Check } from 'lucide-react';
import { use_workspace_store, type Team_Cluster_Preset, type Mission_Cluster } from '../../stores/workspace_store';
import type { Agent } from '../../types';
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

export const Cluster_Manager_Modal: React.FC<ClusterManagerModalProps> = ({
    isOpen,
    onClose,
    agents
}) => {
    const team_presets = use_workspace_store(state => state.team_presets || []);
    const add_team_preset = use_workspace_store(state => state.add_team_preset);
    const update_team_preset = use_workspace_store(state => state.update_team_preset);
    const delete_team_preset = use_workspace_store(state => state.delete_team_preset);

    // Consolidated Form State
    const [form_data, set_form_data] = useState<PresetFormData>(INITIAL_FORM_STATE);
    const [preset_to_delete, set_preset_to_delete] = useState<Team_Cluster_Preset | null>(null);
    const [show_form, set_show_form] = useState(false);

    // O(1) Agent Lookup Map
    const agent_map = useMemo(() => {
        return new Map(agents.map(a => [a.id, a]));
    }, [agents]);

    if (!isOpen) return null;

    const reset_form = () => {
        set_form_data(INITIAL_FORM_STATE);
        set_show_form(false);
    };

    const handle_edit = (preset: Team_Cluster_Preset) => {
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
                <div className="sovereign-card border-zinc-700/60 bg-[color:var(--color-surface)] w-full max-w-4xl max-h-[90vh] flex flex-col shadow-2xl overflow-hidden rounded-xl">
                    {/* Modal Header */}
                    <div className="p-4 px-6 border-b border-[color:var(--color-border)] flex items-center justify-between shrink-0 bg-[color:var(--color-background)]">
                        <div className="flex items-center gap-2">
                            <Layers className="text-cyan-400" size={20} />
                            <div>
                                <h2 className="sovereign-header-text !text-white text-base">Utility Cluster Configuration Studio</h2>
                                <p className="text-xs text-zinc-400">Configure Mode-Switched Multi-Role Agent Teams with strict operational boundaries.</p>
                            </div>
                        </div>
                        <button
                            onClick={onClose}
                            className="p-1.5 rounded-lg hover:bg-zinc-800 text-zinc-400 hover:text-white transition-colors"
                            aria-label="Close studio modal"
                        >
                            <X size={18} />
                        </button>
                    </div>

                    {/* Modal Body */}
                    <div className="p-6 overflow-y-auto space-y-6 flex-1">
                        {/* Header Toolbar */}
                        <div className="flex items-center justify-between">
                            <h3 className="text-xs font-semibold uppercase tracking-wider text-zinc-400 flex items-center gap-1.5">
                                <Users size={14} className="text-cyan-400" /> Defined Team Cluster Presets ({team_presets.length})
                            </h3>
                            {!show_form && (
                                <button
                                    onClick={() => {
                                        reset_form();
                                        set_show_form(true);
                                    }}
                                    className="px-3 py-1.5 rounded-lg bg-cyan-600 hover:bg-cyan-500 text-white font-semibold text-xs transition-all flex items-center gap-1 shadow-sm"
                                >
                                    <Plus size={14} /> Create Preset
                                </button>
                            )}
                        </div>

                        {/* Presets List / Empty State */}
                        {team_presets.length === 0 ? (
                            <Empty_State
                                icon={Layers}
                                title="No Utility Cluster Presets Configured"
                                description="Create reusable multi-role team presets to rapidly initialize mission clusters."
                                action_label="Define First Cluster Preset"
                                on_action={() => set_show_form(true)}
                            />
                        ) : (
                            <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
                                {team_presets.map(preset => {
                                    const is_active_editing = form_data.editing_id === preset.id;
                                    return (
                                        <div
                                            key={preset.id}
                                            className={`p-3 rounded-lg border transition-all flex flex-col justify-between ${
                                                is_active_editing
                                                    ? 'border-cyan-500 bg-cyan-500/10'
                                                    : 'border-[color:var(--color-border)] bg-[color:var(--color-background)] hover:border-zinc-700'
                                            }`}
                                        >
                                            <div>
                                                <div className="flex items-center justify-between mb-1.5">
                                                    <span className="px-1.5 py-0.5 rounded text-[10px] font-mono bg-cyan-500/20 text-cyan-300 border border-cyan-500/30">
                                                        🏷️ {preset.badge_label}
                                                    </span>
                                                    <span className="text-[10px] text-zinc-400 capitalize">{preset.department}</span>
                                                </div>
                                                <h4 className="text-xs font-bold text-white mb-1">{preset.name}</h4>
                                                <p className="text-[11px] text-zinc-400 line-clamp-2 mb-2">{preset.description}</p>
                                                <div className="text-[11px] text-zinc-300 font-mono mb-2">
                                                    Budget: ${preset.default_budget_usd.toFixed(2)} USD
                                                </div>
                                                {/* O(1) Agent Lookups */}
                                                <div className="flex flex-wrap gap-1 mb-2">
                                                    {preset.collaborators.map(agent_id => {
                                                        const agent = agent_map.get(agent_id);
                                                        return (
                                                            <span key={agent_id} className="text-[9px] px-1.5 py-0.5 rounded bg-zinc-800 text-zinc-300 border border-zinc-700">
                                                                {agent?.name || agent_id}
                                                            </span>
                                                        );
                                                    })}
                                                </div>
                                            </div>
                                            <div className="flex items-center justify-end gap-2 pt-2 border-t border-zinc-800/60">
                                                <button
                                                    onClick={() => handle_edit(preset)}
                                                    className="text-[11px] font-medium text-cyan-400 hover:text-cyan-300 transition-colors"
                                                >
                                                    Edit
                                                </button>
                                                <button
                                                    onClick={() => set_preset_to_delete(preset)}
                                                    className="p-1 text-zinc-500 hover:text-red-400 transition-colors"
                                                    title="Delete Preset"
                                                >
                                                    <Trash2 size={12} />
                                                </button>
                                            </div>
                                        </div>
                                    );
                                })}
                            </div>
                        )}

                        {/* Creator / Editor Form Drawer */}
                        {(show_form || form_data.editing_id) && (
                            <div className="sovereign-card border-zinc-800 bg-[color:var(--color-background)] p-4 rounded-xl space-y-4 animate-in slide-in-from-top-2">
                                <div className="flex items-center justify-between border-b border-zinc-800 pb-2">
                                    <h3 className="text-xs font-bold text-white flex items-center gap-1.5">
                                        {form_data.editing_id ? <Shield size={14} className="text-amber-400" /> : <Plus size={14} className="text-cyan-400" />}
                                        {form_data.editing_id ? 'Edit Team Cluster Preset' : 'Create New Utility Cluster Preset'}
                                    </h3>
                                    <button
                                        onClick={reset_form}
                                        className="text-[11px] text-zinc-400 hover:text-white"
                                    >
                                        Cancel
                                    </button>
                                </div>

                                <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                                    <div>
                                        <label className="block text-[11px] font-semibold text-zinc-400 mb-1">Preset Cluster Name</label>
                                        <input
                                            type="text"
                                            placeholder="e.g. Security & Audit Cluster"
                                            value={form_data.name}
                                            onChange={e => set_form_data(prev => ({ ...prev, name: e.target.value }))}
                                            className="w-full bg-[color:var(--color-surface)] border border-[color:var(--color-border)] rounded p-2 text-xs text-white placeholder-zinc-500 focus:outline-none focus:border-cyan-500"
                                        />
                                    </div>
                                    <div>
                                        <label className="block text-[11px] font-semibold text-zinc-400 mb-1">Team Badge Tag (e.g. SEC-AUDIT)</label>
                                        <input
                                            type="text"
                                            placeholder="e.g. SEC-AUDIT"
                                            value={form_data.badge_label}
                                            onChange={e => set_form_data(prev => ({ ...prev, badge_label: e.target.value.toUpperCase() }))}
                                            className="w-full bg-[color:var(--color-surface)] border border-[color:var(--color-border)] rounded p-2 text-xs text-white font-mono placeholder-zinc-500 focus:outline-none focus:border-cyan-500"
                                        />
                                    </div>
                                    <div className="md:col-span-2">
                                        <label className="block text-[11px] font-semibold text-zinc-400 mb-1">Description & Scope</label>
                                        <input
                                            type="text"
                                            placeholder="Brief description of the cluster's operational boundary..."
                                            value={form_data.description}
                                            onChange={e => set_form_data(prev => ({ ...prev, description: e.target.value }))}
                                            className="w-full bg-[color:var(--color-surface)] border border-[color:var(--color-border)] rounded p-2 text-xs text-white placeholder-zinc-500 focus:outline-none focus:border-cyan-500"
                                        />
                                    </div>
                                    <div>
                                        <label className="block text-[11px] font-semibold text-zinc-400 mb-1">Department</label>
                                        <select
                                            value={form_data.department}
                                            onChange={e => set_form_data(prev => ({ ...prev, department: e.target.value as Mission_Cluster['department'] }))}
                                            className="w-full bg-[color:var(--color-surface)] border border-[color:var(--color-border)] rounded p-2 text-xs text-white focus:outline-none focus:border-cyan-500"
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
                                        <label className="block text-[11px] font-semibold text-zinc-400 mb-1">Default Budget (USD)</label>
                                        <input
                                            type="number"
                                            step="100"
                                            min="1"
                                            value={form_data.budget_usd}
                                            onChange={e => set_form_data(prev => ({ ...prev, budget_usd: e.target.value }))}
                                            className="w-full bg-[color:var(--color-surface)] border border-[color:var(--color-border)] rounded p-2 text-xs text-white font-mono focus:outline-none focus:border-cyan-500"
                                        />
                                    </div>
                                </div>

                                {/* Agent Selector Chips */}
                                <div>
                                    <label className="block text-[11px] font-semibold text-zinc-400 mb-1.5">
                                        Select Multi-Role Agents ({form_data.selected_agents.length} Selected)
                                    </label>
                                    <div className="flex flex-wrap gap-1.5 max-h-36 overflow-y-auto p-2 bg-[color:var(--color-surface)] border border-[color:var(--color-border)] rounded-lg">
                                        {agents.map(agent => {
                                            const is_selected = form_data.selected_agents.includes(agent.id);
                                            return (
                                                <button
                                                    type="button"
                                                    key={agent.id}
                                                    onClick={() => toggle_agent(agent.id)}
                                                    className={`px-2 py-1 rounded text-xs transition-all flex items-center gap-1 ${
                                                        is_selected
                                                            ? 'bg-cyan-600 text-white font-medium shadow-sm'
                                                            : 'bg-zinc-800 text-zinc-400 hover:text-zinc-200 border border-zinc-700'
                                                    }`}
                                                >
                                                    {is_selected && <Check size={10} />}
                                                    {agent.name}
                                                </button>
                                            );
                                        })}
                                    </div>
                                </div>

                                {/* Form Submit Button */}
                                <div className="flex justify-end pt-2">
                                    <button
                                        type="button"
                                        onClick={handle_save}
                                        disabled={!is_form_valid}
                                        className="px-4 py-2 rounded-lg bg-cyan-600 hover:bg-cyan-500 disabled:opacity-50 text-white font-semibold text-xs transition-all flex items-center gap-1.5"
                                    >
                                        <Check size={14} /> {form_data.editing_id ? 'Save Changes' : 'Create Cluster Preset'}
                                    </button>
                                </div>
                            </div>
                        )}
                    </div>
                </div>
            </div>
        </>
    );
};
