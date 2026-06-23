/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Template_Store_Header]` in observability traces.
 */

import { Store, ShieldCheck } from 'lucide-react';
import { i18n } from '../../i18n';

export function Template_Store_Header() {
    return (
        <div className="flex flex-col md:flex-row justify-between items-start md:items-center gap-4 border-b border-[color:var(--color-border)]/50 pb-6 mb-4">
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
    );
}

// Metadata: [Template_Store_Header]
