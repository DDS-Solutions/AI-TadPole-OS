/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Tri-slot neural model selector for multi-inference agents. 
 * Facilitates hot-swapping active model slots and updating model configurations via the `model_store`.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: `active_model_slot` mismatch between UI LED and backend state, dropdown mount failure on window edge, or "Unknown Model" fallbacks during provider outages.
 * - **Telemetry Link**: Search for `[Node_Model_Slots]` or `Model_Badge` in UI logs.
 */

import React from 'react';
import { i18n } from '../../i18n';
import { Tooltip } from '../ui';
import { Model_Badge } from '../Model_Badge';
import { use_dropdown_store } from '../../stores/dropdown_store';
import { use_model_store } from '../../stores/model_store';
import type { Agent } from '../../types';

interface Node_Model_Slots_Props {
    agent: Agent;
    on_model_change?: (agent_id: string, new_model: string) => void;
    on_model_2_change?: (agent_id: string, new_model: string) => void;
    on_model_3_change?: (agent_id: string, new_model: string) => void;
    on_update?: (agent_id: string, updates: Partial<Agent>) => void;
}

export const Node_Model_Slots: React.FC<Node_Model_Slots_Props> = ({
    agent,
    on_model_change,
    on_model_2_change,
    on_model_3_change,
    on_update
}) => {
    const toggle_dropdown = use_dropdown_store(s => s.toggle_dropdown);
    const close_dropdown = use_dropdown_store(s => s.close_dropdown);
    const is_model_1_open = use_dropdown_store(s => s.is_open(agent.id, 'model'));
    const is_model_2_open = use_dropdown_store(s => s.is_open(agent.id, 'model_2'));
    const is_model_3_open = use_dropdown_store(s => s.is_open(agent.id, 'model_3'));

    const available_models = use_model_store(s => s.models);
    const sorted_model_names = React.useMemo(() =>
        Array.from(new Set((available_models || []).map(m => m.name))).sort(),
        [available_models]);

    const render_slot = (slot_idx: 1 | 2 | 3, model: string | undefined, is_open: boolean, on_model_update?: (id: string, m: string) => void) => {
        const is_active_slot = agent.active_model_slot === slot_idx || (slot_idx === 1 && !agent.active_model_slot && agent.status !== 'idle' && agent.status !== 'offline');

        const led_color =
            slot_idx === 1 ? 'bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.5)]' :
                slot_idx === 2 ? 'bg-blue-500 shadow-[0_0_8px_rgba(59,130,246,0.5)]' :
                    'bg-amber-500 shadow-[0_0_8px_rgba(245,158,11,0.5)]';

        return (
            <div className="relative" key={`slot-${slot_idx}`} role="presentation" onClick={(e) => e.stopPropagation()} onKeyDown={(e) => e.stopPropagation()}>
                <Tooltip content={i18n.t(`agent_card.tooltip_activate_${slot_idx === 1 ? 'primary' : slot_idx === 2 ? 'secondary' : 'tertiary'}`)} position="top">
                    <button
                        onClick={(e) => {
                            e.stopPropagation();
                            if (agent.active_model_slot !== slot_idx) {
                                on_update?.(agent.id, { active_model_slot: slot_idx });
                            }
                        }}
                        className="absolute -top-7 left-1/2 -translate-x-1/2 w-6 h-6 flex items-center justify-center cursor-pointer z-30 group/led"
                    >
                        <div className={`
                            w-1.5 h-1.5 rounded-full transition-all duration-300
                            ${is_active_slot ? `${led_color} scale-110` : 'bg-zinc-800 group-hover/led:bg-zinc-700'}
                        `} />
                    </button>
                </Tooltip>
                <Model_Badge
                    model={model || (slot_idx === 1 ? i18n.t('agent_card.label_unknown_model') : i18n.t('agent_card.label_add_model'))}
                    is_active={is_active_slot}
                    on_click={() => toggle_dropdown(agent.id, slot_idx === 1 ? 'model' : slot_idx === 2 ? 'model_2' : 'model_3')}
                />
                {is_open && (
                    <div className="absolute top-full left-0 mt-1 w-48 bg-zinc-900 border border-zinc-700 rounded-lg shadow-xl z-50 py-1 max-h-60 overflow-y-auto custom-scrollbar">
                        {sorted_model_names.map((m) => (
                            <button key={m} onClick={() => {
                                on_model_update?.(agent.id, m);
                                close_dropdown();
                            }}
                                className={`w-full text-left px-3 py-1.5 text-xs hover:bg-zinc-800 transition-colors ${model === m ? 'text-emerald-400 font-bold bg-emerald-900/10' : 'text-zinc-300'}`}>
                                {m}
                            </button>
                        ))}
                    </div>
                )}
            </div>
        );
    };

    return (
        <div className={`flex flex-col gap-1.5 border-t border-zinc-800 pt-2 relative ${is_model_1_open || is_model_2_open || is_model_3_open ? 'z-50' : 'z-20'}`}>
            <div className="flex items-center gap-1.5 overflow-visible">
                {render_slot(1, agent.model, is_model_1_open, on_model_change)}
                {render_slot(2, agent.model_2, is_model_2_open, on_model_2_change)}
                {render_slot(3, agent.model_3, is_model_3_open, on_model_3_change)}
            </div>
        </div>
    );
};

