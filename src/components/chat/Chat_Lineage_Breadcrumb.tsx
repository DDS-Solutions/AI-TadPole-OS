/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Chat / Chat_Lineage_Breadcrumb
 * - **Primary Entrypoints**: `Chat_Lineage_Breadcrumb`
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
import { i18n } from '../../i18n';

interface Chat_Lineage_Breadcrumb_Props {
    target_agent: string;
}

/**
 * Chat_Lineage_Breadcrumb
 * Renders the `Overlord / @Agent` neural lineage trail.
 * Only shown when scope is 'agent'.
 */
export const Chat_Lineage_Breadcrumb = React.memo<Chat_Lineage_Breadcrumb_Props>(({ target_agent }) => {
    return (
        <div className="bg-[color:var(--color-background)]/40 border-b border-[color:var(--color-border)]/30 px-4 py-2 flex items-center gap-2 overflow-x-auto no-scrollbar relative z-10 select-none">
            <span className="text-[10px] text-zinc-600 font-bold uppercase tracking-wider whitespace-nowrap">{i18n.t('chat.lineage_label')}</span>
            <div className="flex items-center gap-1.5 scroll-smooth">
                <span className="text-[10px] text-zinc-100 bg-zinc-800 px-2 py-0.5 rounded border border-zinc-700/50 hover:bg-zinc-700 transition-colors cursor-default shadow-sm">{i18n.t('chat.overlord_name')}</span>
                {target_agent !== 'CEO' && (
                    <>
                        <span className="text-zinc-700 text-[10px] animate-pulse">/</span>
                        <span className="text-[10px] text-green-400 bg-green-500/10 px-2 py-0.5 rounded border border-green-500/20 hover:bg-green-500/20 transition-all cursor-default shadow-[0_0_10px_rgba(59,130,246,0.15)]">{i18n.t('chat.agent_label', { name: target_agent })}</span>
                    </>
                )}
            </div>
        </div>
    );
});

Chat_Lineage_Breadcrumb.displayName = 'Chat_Lineage_Breadcrumb';
