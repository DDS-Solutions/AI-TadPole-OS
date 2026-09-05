/**
 * @docs ARCHITECTURE:UI-Components
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Template_Store / Empty_State
 * - **Primary Entrypoints**: `Empty_State`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { Search } from 'lucide-react';
import { i18n } from '../../../i18n';

export function Empty_State() {
    return (
        <div className="flex flex-col items-center justify-center py-20 text-zinc-500">
            <Search size={48} className="mb-4 opacity-20" />
            <p>{i18n.t('template_store.empty_title')}</p>
        </div>
    );
}
