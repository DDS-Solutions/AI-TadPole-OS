/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Loading_State]` in observability traces.
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

// Metadata: [Loading_State]
