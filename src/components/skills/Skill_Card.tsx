/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Skills / Skill_Card
 * - **Primary Entrypoints**: `Skill_Card`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import React from 'react';
import { Edit2, Users, Trash2, Terminal, Shield, ShieldCheck, ShieldAlert } from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';
import type { Skill_Definition } from '../../stores/skill_store';

interface Skill_Card_Props {
    skill: Skill_Definition;
    on_edit: (skill: Skill_Definition) => void;
    on_assign: (name: string) => void;
    on_delete: (name: string) => void;
    on_view_report?: (skill: Skill_Definition) => void;
}

export const Skill_Card: React.FC<Skill_Card_Props> = ({ skill, on_edit, on_assign, on_delete, on_view_report }) => {
    const score = skill.security_score;
    let badgeColor = 'text-zinc-500 bg-zinc-800/40 border-zinc-700/50';
    let badgeText = i18n.t('skills.security_unscanned', { defaultValue: 'Unscanned' });
    let ShieldIcon = Shield;

    if (score !== undefined && score !== null) {
        if (score <= 20) {
            badgeColor = 'text-[color:var(--color-cyber-green)] bg-[color:var(--color-cyber-green)]/10 border-[color:var(--color-cyber-green)]/20';
            badgeText = `${i18n.t('skills.security_safe', { defaultValue: 'Safe' })} (${score})`;
            ShieldIcon = ShieldCheck;
        } else if (score <= 50) {
            badgeColor = 'text-[color:var(--color-cyber-amber)] bg-[color:var(--color-cyber-amber)]/10 border-[color:var(--color-cyber-amber)]/20';
            badgeText = `${i18n.t('skills.security_caution', { defaultValue: 'Caution' })} (${score})`;
            ShieldIcon = ShieldAlert;
        } else {
            badgeColor = 'text-[color:var(--color-cyber-red)] bg-[color:var(--color-cyber-red)]/10 border-[color:var(--color-cyber-red)]/25';
            badgeText = `${i18n.t('skills.security_dangerous', { defaultValue: 'Dangerous' })} (${score})`;
            ShieldIcon = ShieldAlert;
        }
    }

    return (
        <div className="bg-[color:var(--color-background)] border border-[color:var(--color-border)] p-5 rounded-xl transition-all duration-300 hover:border-emerald-500/30 hover:shadow-[0_0_15px_rgba(16,185,129,0.15)] group relative overflow-hidden shadow-sm">
            <div className="neural-grid opacity-[0.03]" />
            <div className="absolute top-4 right-4 flex gap-2 opacity-0 group-hover:opacity-100 group-focus-within:opacity-100 transition-opacity z-20">
                <Tooltip content={i18n.t('skills.tooltip_edit_skill')} position="top">
                    <button onClick={() => on_edit(skill)} className="text-zinc-500 hover:text-green-400 bg-[color:var(--color-surface)] hover:bg-zinc-800 p-1.5 rounded cursor-pointer">
                        <Edit2 className="w-3.5 h-3.5" />
                    </button>
                </Tooltip>
                <Tooltip content={i18n.t('agent_manager.tooltip_assign')} position="top">
                    <button onClick={() => on_assign(skill.name)} className="text-zinc-500 hover:text-emerald-400 bg-[color:var(--color-surface)] hover:bg-zinc-800 p-1.5 rounded transition-colors cursor-pointer">
                        <Users className="w-3.5 h-3.5" />
                    </button>
                </Tooltip>
                <Tooltip content={i18n.t('skills.tooltip_delete_skill')} position="top">
                    <button onClick={() => on_delete(skill.name)} className="text-zinc-500 hover:text-red-400 bg-[color:var(--color-surface)] hover:bg-zinc-800 p-1.5 rounded cursor-pointer">
                        <Trash2 className="w-3.5 h-3.5" />
                    </button>
                </Tooltip>
            </div>
            <div className="relative z-10 flex flex-col h-full justify-between">
                <div>
                    <div className="flex items-center gap-3 mb-2 pr-16 text-zinc-300 font-bold tracking-wide">
                        <div className="w-2 h-2 rounded-full bg-emerald-500/30 group-hover:bg-emerald-400 group-hover:shadow-[0_0_8px_rgba(16,185,129,0.5)] transition-all shrink-0 mt-0.5"></div>
                        <h3 className="font-mono text-sm">{skill.name}</h3>
                    </div>
                    <p className="text-zinc-500 text-xs line-clamp-2 mb-4 h-8 leading-relaxed font-mono">{skill.description}</p>
                    <div className="bg-zinc-950/40 border border-[color:var(--color-border)]/50 p-2.5 rounded font-mono text-[10px] text-zinc-300 flex items-center gap-2 overflow-x-auto">
                        <Terminal className="w-3 h-3 flex-shrink-0 text-zinc-500" />
                        <span className="whitespace-nowrap">{skill.execution_command}</span>
                    </div>
                </div>

                <div className="mt-4 flex items-center justify-between border-t border-[color:var(--color-border)]/30 pt-3">
                    <div className={`flex items-center gap-1 px-2.5 py-0.5 rounded border text-[10px] font-mono ${badgeColor}`}>
                        <ShieldIcon className="w-3 h-3" />
                        <span>{badgeText}</span>
                    </div>
                    {score !== undefined && score !== null && on_view_report && (
                        <button
                            onClick={() => on_view_report(skill)}
                            className="text-[10px] font-mono text-zinc-500 hover:text-zinc-300 transition-colors cursor-pointer underline underline-offset-2"
                        >
                            {i18n.t('skills.view_security_report', { defaultValue: 'View Report' })}
                        </button>
                    )}
                </div>
            </div>
        </div>
    );
};
