/**
 * @docs ARCHITECTURE:UI-Components
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Template_Store / Template_Store_Header
 * - **Primary Entrypoints**: `Template_Store_Header`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { Store, ShieldCheck } from 'lucide-react';
import { i18n } from '../../i18n';

export function Template_Store_Header() {
    return (
        <div className="flex flex-col border-b border-[color:var(--color-border)]/50 pb-6 mb-4">
            <div className="flex flex-col md:flex-row justify-between items-start md:items-center gap-4">
                <div>
                    <h1 className="text-2xl font-bold text-zinc-100 flex items-center gap-3 tracking-tight">
                        <Store className="text-green-500" size={28} />
                        {i18n.t('template_store.title')}
                    </h1>
                    <p className="text-sm text-zinc-400 mt-1">{i18n.t('template_store.desc')}</p>
                </div>
                <div className="flex items-center gap-2 px-3 py-1.5 bg-[color:var(--color-surface)] rounded-full border border-[color:var(--color-border)] text-xs font-mono text-zinc-400 flex-shrink-0">
                    <ShieldCheck size={14} className="text-emerald-500" />
                    {i18n.t('template_store.shield_active')}
                </div>
            </div>
        </div>
    );
}
