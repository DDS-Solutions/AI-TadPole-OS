/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Empty_State]` in observability traces.
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

// Metadata: [Empty_State]
