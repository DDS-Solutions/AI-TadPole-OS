/**
 * @docs ARCHITECTURE:UI-Components
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Template_Store / Market_Selector
 * - **Primary Entrypoints**: `Market_Selector`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { NavLink } from 'react-router-dom';
import { i18n } from '../../i18n';

export function Market_Selector() {
    return (
        <div className="flex items-center gap-2 bg-[color:var(--color-surface)]/30 border border-[color:var(--color-border)]/50 rounded-2xl p-1.5 w-fit">
            <NavLink
                to="/infra/model-store"
                className={({ isActive }) =>
                    `px-4 py-2 rounded-xl text-xs font-bold uppercase tracking-wider transition-all duration-200 border ${
                        isActive
                            ? 'bg-green-600 border-green-500 text-white shadow-lg shadow-green-500/20'
                            : 'bg-transparent border-transparent text-zinc-400 hover:text-zinc-100 hover:bg-zinc-800/40'
                    }`
                }
            >
                {i18n.t('NAV_MODEL_STORE') || 'Model Store'}
            </NavLink>
            <NavLink
                to="/store"
                className={({ isActive }) =>
                    `px-4 py-2 rounded-xl text-xs font-bold uppercase tracking-wider transition-all duration-200 border ${
                        isActive
                            ? 'bg-green-600 border-green-500 text-white shadow-lg shadow-green-500/20'
                            : 'bg-transparent border-transparent text-zinc-400 hover:text-zinc-100 hover:bg-zinc-800/40'
                    }`
                }
            >
                {i18n.t('NAV_TEMPLATE_STORE') || 'Template Store'}
            </NavLink>
        </div>
    );
}
