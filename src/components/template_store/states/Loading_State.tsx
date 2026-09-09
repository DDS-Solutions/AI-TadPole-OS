/**
 * @docs ARCHITECTURE:UI-Components
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Template_Store / Loading_State
 * - **Primary Entrypoints**: `Loading_State`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { i18n } from '../../../i18n';

export function Loading_State() {
    return (
        <div className="flex flex-col items-center justify-center py-20 text-zinc-500">
            <div className="w-8 h-8 rounded-full border-2 border-green-500 border-t-transparent animate-spin mb-4"></div>
            <p>{i18n.t('template_store.connecting')}</p>
        </div>
    );
}
