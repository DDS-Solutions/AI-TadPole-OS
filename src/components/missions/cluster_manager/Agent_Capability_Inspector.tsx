/**
 * @docs ARCHITECTURE:Interface:Missions
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Missions / Cluster Manager / Agent_Capability_Inspector
 * - **Primary Entrypoints**: `Agent_Capability_Inspector`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Rendered via Portal to document.body with fixed viewport positioning.
 * - `[Structural]` Uses pointer-events-none to prevent mouse event interference with the modal.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import React from 'react';
import { createPortal } from 'react-dom';
import { Cpu } from 'lucide-react';
import type { Agent } from '../../../types';
import { parse_active_model_slot } from '../../../utils/model_utils';

export const Agent_Capability_Inspector: React.FC<{ agent: Agent }> = ({ agent }) => {
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
                                        <span className="text-[9px] px-1 rounded bg-cyan-500 text-zinc-950 font-extrabold uppercase tracking-tight">
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
