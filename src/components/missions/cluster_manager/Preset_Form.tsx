/**
 * @docs ARCHITECTURE:Interface:Missions
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Missions / Cluster Manager / Preset_Form
 * - **Primary Entrypoints**: `Preset_Form`
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import React from 'react';
import { Check, Sliders, Search } from 'lucide-react';
import type { Agent } from '../../../types';
import type { Mission_Cluster } from '../../../stores/workspace_store';
import type { PresetFormData } from './types';

interface PresetFormProps {
    form_ref: React.RefObject<HTMLDivElement | null>;
    form_data: PresetFormData;
    update_field: <K extends keyof PresetFormData>(field: K, value: PresetFormData[K]) => void;
    reset_form: () => void;
    handle_save: () => void;
    is_form_valid: boolean;
    agent_search_query: string;
    set_agent_search_query: (q: string) => void;
    filtered_selectable_agents: Agent[];
    toggle_agent: (agent_id: string) => void;
    handle_agent_hover: (agent: Agent) => void;
    handle_agent_leave: () => void;
}

export const Preset_Form: React.FC<PresetFormProps> = ({
    form_ref,
    form_data,
    update_field,
    reset_form,
    handle_save,
    is_form_valid,
    agent_search_query,
    set_agent_search_query,
    filtered_selectable_agents,
    toggle_agent,
    handle_agent_hover,
    handle_agent_leave,
}) => {
    return (
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
    );
};
