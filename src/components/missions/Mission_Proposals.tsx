/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Mission_Proposals]` in observability traces.
 */

import React from 'react';
import { Cpu } from 'lucide-react';
import { i18n } from '../../i18n';
import { get_theme_colors } from '../../utils/agent_uiutils';

interface MissionProposalsProps {
    proposal: { reasoning: string } | undefined;
    theme: string;
    on_dismiss: () => void;
    on_authorize: () => void;
}

export const Mission_Proposals: React.FC<MissionProposalsProps> = ({ 
    proposal, 
    theme, 
    on_dismiss, 
    on_authorize 
}) => {
    if (!proposal) return null;

    const colors = get_theme_colors(theme);

    return (
        <div className={`p-4 rounded-xl border ${colors.bg} ${colors.border} animate-in slide-in-from-top-4`}>
            <div className="flex justify-between items-start gap-4">
                <div className="flex gap-4">
                    <div className={`p-2 rounded-lg bg-[color:var(--color-surface)] border ${colors.border}`}>
                        <Cpu size={20} className={colors.text} />
                    </div>
                    <div className="space-y-1">
                        <h4 className="text-xs font-bold text-zinc-100 uppercase tracking-tight">{i18n.t('missions.proposal_title')}</h4>
                        <p className="text-xs text-zinc-400 font-mono leading-relaxed max-w-xl">
                            {proposal.reasoning}
                        </p>
                    </div>
                </div>
                <div className="flex gap-2">
                    <button
                        onClick={on_dismiss}
                        className="px-3 py-1.5 rounded-lg border border-zinc-700 bg-[color:var(--color-surface)] text-zinc-500 text-xs font-bold uppercase transition-colors"
                    >
                        {i18n.t('missions.btn_dismiss')}
                    </button>
                    <button
                        onClick={on_authorize}
                        className={`px-3 py-1.5 rounded-lg border ${colors.text} ${colors.border} bg-[color:var(--color-surface)] text-xs font-bold uppercase shadow-lg ${colors.glow}`}
                    >
                        {i18n.t('missions.btn_authorize')}
                    </button>
                </div>
            </div>
        </div>
    );
};

// Metadata: [Mission_Proposals]
