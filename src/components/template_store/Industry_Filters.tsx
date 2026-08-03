/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Industry_Filters]` in observability traces.
 */

import { Filter } from 'lucide-react';

interface IndustryFiltersProps {
    industries: string[];
    selected: string;
    onSelect: (industry: string) => void;
}

export function Industry_Filters({ industries, selected, onSelect }: IndustryFiltersProps) {
    return (
        <div data-testid="industry-filters" className="flex items-center gap-2 overflow-x-auto pb-2 custom-scrollbar w-full">
            <Filter className="text-zinc-500 mr-2 flex-shrink-0" size={16} />
            {industries.map((ind) => (
                <button
                    key={ind}
                    onClick={() => onSelect(ind)}
                    className={`px-4 py-1.5 rounded-full text-xs font-bold whitespace-nowrap transition-colors ${
                        selected === ind
                            ? 'bg-green-600 text-white shadow-lg shadow-green-500/20'
                            : 'bg-[color:var(--color-surface)] text-zinc-400 hover:bg-zinc-800 border border-[color:var(--color-border)]'
                    }`}
                >
                    {ind}
                </button>
            ))}
        </div>
    );
}

// Metadata: [Industry_Filters]
