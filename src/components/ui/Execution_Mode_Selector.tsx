/**
 * @docs ARCHITECTURE:UI-Components
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Ui / Execution_Mode_Selector
 * - **Primary Entrypoints**: `Execution_Mode_Selector`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 * - `[Structural]` Exposes accessible segmented mode controls (Chat, Plan, Swarm) with optional Swarm handoff action.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import React from 'react';
import { MessageSquare, Search, Zap, ArrowUpRight } from 'lucide-react';
import clsx from 'clsx';
import { Tooltip } from './Tooltip';
import type { Execution_Mode } from '../../contracts/agent';

export interface Execution_Mode_Selector_Props {
    mode: Execution_Mode;
    on_change: (mode: Execution_Mode) => void;
    compact?: boolean;
    disabled?: boolean;
    can_launch_swarm?: boolean;
    on_launch_swarm?: () => void;
}

const MODE_META: Array<{
    id: Execution_Mode;
    label: string;
    tooltip: string;
    icon: React.ComponentType<{ size?: number; className?: string }>;
    active_class: string;
}> = [
    {
        id: 'chat',
        label: 'Chat',
        tooltip: 'Chat Mode: Lightweight brainstorm — zero tools, no Mission created.',
        icon: MessageSquare,
        active_class: 'bg-emerald-500/20 text-emerald-400 border-emerald-500/40',
    },
    {
        id: 'plan',
        label: 'Plan',
        tooltip: 'Plan Mode: Read-only codebase research — zero writes, no Mission created.',
        icon: Search,
        active_class: 'bg-cyan-500/20 text-cyan-400 border-cyan-500/40',
    },
    {
        id: 'swarm',
        label: 'Swarm',
        tooltip: 'Swarm Mode: Full tracked Mission with tools, delegation, and Oversight Gate.',
        icon: Zap,
        active_class: 'bg-amber-500/20 text-amber-400 border-amber-500/40',
    },
];

export const Execution_Mode_Selector: React.FC<Execution_Mode_Selector_Props> = ({
    mode,
    on_change,
    compact = false,
    disabled = false,
    can_launch_swarm = false,
    on_launch_swarm,
}) => {
    return (
        <div className={clsx('flex items-center gap-2 flex-wrap', compact ? 'justify-between' : 'justify-center')}>
            <div
                role="group"
                aria-label="Execution mode"
                className="inline-flex items-center gap-1 rounded-lg bg-zinc-950/70 border border-[color:var(--color-border)]/70 p-0.5"
            >
                {MODE_META.map((item) => {
                    const Icon = item.icon;
                    const is_active = mode === item.id;
                    return (
                        <Tooltip key={item.id} content={item.tooltip} position="top">
                            <button
                                type="button"
                                disabled={disabled}
                                aria-pressed={is_active}
                                aria-label={`${item.label} mode`}
                                onClick={() => on_change(item.id)}
                                className={clsx(
                                    'inline-flex items-center gap-1.5 rounded-md border font-mono font-bold uppercase tracking-wider transition-all cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed',
                                    compact ? 'px-2 py-1 text-[10px]' : 'px-3 py-1.5 text-[11px]',
                                    is_active
                                        ? item.active_class
                                        : 'border-transparent text-zinc-500 hover:text-zinc-200 hover:bg-zinc-900/80'
                                )}
                            >
                                <Icon size={compact ? 11 : 13} />
                                <span>{item.label}</span>
                            </button>
                        </Tooltip>
                    );
                })}
            </div>

            {mode !== 'swarm' && can_launch_swarm && on_launch_swarm && (
                <Tooltip
                    content="Switch to Swarm Mode and launch a tracked Mission carrying this brainstorm thread."
                    position="top"
                >
                    <button
                        type="button"
                        disabled={disabled}
                        aria-label="Launch Swarm Mission from brainstorm"
                        onClick={on_launch_swarm}
                        className={clsx(
                            'inline-flex items-center gap-1 rounded-md border border-amber-500/40 bg-amber-500/15 text-amber-300 hover:bg-amber-500/25 hover:text-amber-200 font-mono font-bold uppercase tracking-wider transition-all cursor-pointer disabled:opacity-50',
                            compact ? 'px-2 py-1 text-[10px]' : 'px-3 py-1.5 text-[11px]'
                        )}
                    >
                        <span>Launch Swarm</span>
                        <ArrowUpRight size={compact ? 11 : 13} />
                    </button>
                </Tooltip>
            )}
        </div>
    );
};
