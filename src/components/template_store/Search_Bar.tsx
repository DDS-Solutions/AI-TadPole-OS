/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Search_Bar]` in observability traces.
 */

import { Search } from 'lucide-react';
import { i18n } from '../../i18n';

interface SearchBarProps {
    value: string;
    onChange: (value: string) => void;
}

export function Search_Bar({ value, onChange }: SearchBarProps) {
    return (
        <div className="relative w-full">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-zinc-500" size={18} />
            <input
                type="text"
                placeholder={i18n.t('template_store.search_placeholder')}
                value={value}
                onChange={(e) => onChange(e.target.value)}
                className="w-full bg-[color:var(--color-surface)]/50 border border-[color:var(--color-border)] rounded-lg py-2.5 pl-10 pr-4 text-sm text-zinc-100 focus:outline-none focus:border-green-500 transition-colors"
            />
        </div>
    );
}

// Metadata: [Search_Bar]
