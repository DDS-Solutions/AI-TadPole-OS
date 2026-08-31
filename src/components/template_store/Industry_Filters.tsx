/**
 * @docs ARCHITECTURE:UI-Components
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Template_Store / Industry_Filters
 * - **Primary Entrypoints**: `Industry_Filters`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
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
