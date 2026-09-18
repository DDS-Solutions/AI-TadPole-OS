/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Chat / Question_Choice_Pill
 * - **Primary Entrypoints**: `Question_Choice_Pill`, `Question_Choice_Pill_Props`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Interactive choice selection propagates through sovereign store oversight resolution.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import React, { useState } from 'react';
import { HelpCircle, Sparkles, CheckCircle2, ArrowRight, CornerDownLeft } from 'lucide-react';
import clsx from 'clsx';
import { use_sovereign_store } from '../../stores/sovereign_store';

export interface Question_Choice_Pill_Props {
    question: string;
    options: string[];
    context?: string;
    question_id?: string;
    selected_option?: string;
    status?: 'pending' | 'answered';
}

export const Question_Choice_Pill: React.FC<Question_Choice_Pill_Props> = ({
    question,
    options,
    context,
    question_id,
    selected_option,
    status = 'pending',
}) => {
    const [is_submitting, set_is_submitting] = useState(false);
    const [custom_input, set_custom_input] = useState('');
    const [show_custom_input, set_show_custom_input] = useState(false);
    const resolve_user_question = use_sovereign_store((s) => s.resolve_user_question);

    const is_answered = status === 'answered' || Boolean(selected_option);

    const handle_select_option = async (choice: string) => {
        if (is_answered || is_submitting) return;
        set_is_submitting(true);
        try {
            await resolve_user_question(question_id || '', choice);
        } finally {
            set_is_submitting(false);
        }
    };

    const handle_submit_custom = async (e: React.FormEvent) => {
        e.preventDefault();
        const trimmed = custom_input.trim();
        if (!trimmed || is_answered || is_submitting) return;
        set_is_submitting(true);
        try {
            await resolve_user_question(question_id || '', trimmed);
        } finally {
            set_is_submitting(false);
        }
    };

    const is_recommended = (opt: string) => {
        const lower = opt.toLowerCase();
        return lower.includes('(recommended)') || lower.startsWith('recommended:');
    };

    const clean_option_text = (opt: string) => {
        return opt.replace(/\s*\(recommended\)\s*/i, '').trim();
    };

    return (
        <div className="flex flex-col gap-2.5 my-1 p-3.5 rounded-xl bg-zinc-900/90 border border-zinc-700/60 shadow-xl backdrop-blur-md">
            {/* Header */}
            <div className="flex items-center gap-2">
                <div className="p-1 rounded bg-emerald-500/10 text-emerald-400 border border-emerald-500/30">
                    <HelpCircle size={13} />
                </div>
                <span className="text-[10px] font-mono font-semibold uppercase tracking-wider text-emerald-400/90">
                    Human-In-The-Loop Decision
                </span>
                {is_answered && (
                    <span className="ml-auto inline-flex items-center gap-1 text-[10px] font-mono text-emerald-400 bg-emerald-500/10 px-2 py-0.5 rounded-full border border-emerald-500/20">
                        <CheckCircle2 size={10} />
                        Resolved
                    </span>
                )}
            </div>

            {/* Question Text */}
            <div className="text-[13px] font-medium text-zinc-100 leading-snug">
                {question}
            </div>

            {/* Optional Context Callout */}
            {Boolean(context) && (
                <div className="p-2.5 rounded-lg bg-zinc-950/60 border border-zinc-800 text-[11px] text-zinc-400 leading-relaxed font-mono">
                    <span className="text-zinc-500 font-bold uppercase tracking-wider text-[9px] block mb-0.5">
                        Context:
                    </span>
                    {context}
                </div>
            )}

            {/* Answered View */}
            {is_answered ? (
                <div className="mt-1 p-2.5 rounded-lg bg-emerald-950/20 border border-emerald-500/30 flex items-center gap-2 text-xs text-emerald-300">
                    <CheckCircle2 size={14} className="text-emerald-400 shrink-0" />
                    <span className="font-medium">Selected:</span>
                    <span className="font-mono text-zinc-100">{selected_option || 'Answer submitted'}</span>
                </div>
            ) : (
                /* Choice Pills & Custom Write-in */
                <div className="flex flex-col gap-2 mt-1">
                    <div className="flex flex-wrap gap-2">
                        {options.map((opt, idx) => {
                            const recommended = is_recommended(opt);
                            const label = clean_option_text(opt);

                            return (
                                <button
                                    key={`choice-${idx}`}
                                    disabled={is_submitting}
                                    onClick={() => handle_select_option(opt)}
                                    className={clsx(
                                        "relative group flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium transition-all duration-200 cursor-pointer text-left",
                                        recommended
                                            ? "bg-emerald-950/40 text-emerald-200 border border-emerald-500/50 hover:bg-emerald-900/50 hover:border-emerald-400 shadow-[0_0_12px_rgba(16,185,129,0.2)] active:scale-95"
                                            : "bg-zinc-800/70 text-zinc-200 border border-zinc-700/60 hover:bg-zinc-700/80 hover:border-zinc-500 hover:text-white active:scale-95",
                                        is_submitting && "opacity-50 cursor-not-allowed"
                                    )}
                                >
                                    {recommended && (
                                        <Sparkles size={11} className="text-emerald-400 shrink-0 animate-pulse" />
                                    )}
                                    <span>{label}</span>
                                    {recommended && (
                                        <span className="text-[8px] font-mono font-bold tracking-widest text-emerald-300 bg-emerald-500/20 px-1 py-0.2 rounded uppercase">
                                            Rec
                                        </span>
                                    )}
                                    <ArrowRight size={10} className="opacity-0 group-hover:opacity-100 transition-opacity ml-0.5 text-zinc-400" />
                                </button>
                            );
                        })}

                        {!show_custom_input && (
                            <button
                                disabled={is_submitting}
                                onClick={() => set_show_custom_input(true)}
                                className="px-2.5 py-1.5 rounded-lg text-[11px] font-mono text-zinc-400 border border-dashed border-zinc-700/70 hover:border-zinc-500 hover:text-zinc-200 hover:bg-zinc-800/40 transition-colors"
                            >
                                + Custom answer
                            </button>
                        )}
                    </div>

                    {show_custom_input && (
                        <form onSubmit={handle_submit_custom} className="flex items-center gap-2 mt-1">
                            <input
                                type="text"
                                value={custom_input}
                                onChange={(e) => set_custom_input(e.target.value)}
                                placeholder="Type custom instruction or answer..."
                                disabled={is_submitting}
                                autoFocus
                                className="flex-1 bg-zinc-950/80 border border-zinc-700/70 focus:border-emerald-500/70 rounded-lg px-3 py-1.5 text-xs text-zinc-100 placeholder-zinc-500 focus:outline-none focus:ring-1 focus:ring-emerald-500/40 font-sans"
                            />
                            <button
                                type="submit"
                                disabled={!custom_input.trim() || is_submitting}
                                className="px-3 py-1.5 rounded-lg bg-emerald-600 hover:bg-emerald-500 disabled:opacity-40 disabled:cursor-not-allowed text-zinc-950 font-semibold text-xs flex items-center gap-1 transition-all"
                            >
                                <span>Submit</span>
                                <CornerDownLeft size={12} />
                            </button>
                            <button
                                type="button"
                                onClick={() => {
                                    set_show_custom_input(false);
                                    set_custom_input('');
                                }}
                                className="px-2 py-1.5 text-xs text-zinc-500 hover:text-zinc-300"
                            >
                                Cancel
                            </button>
                        </form>
                    )}
                </div>
            )}
        </div>
    );
};

Question_Choice_Pill.displayName = 'Question_Choice_Pill';
