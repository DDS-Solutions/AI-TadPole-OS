/**
 * @docs ARCHITECTURE:UI-Components
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Model / Modality_Badge
 * - **Primary Entrypoints**: `ModalityBadge`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import type { Model_Entry } from '../../stores/provider_store';

interface ModalityBadgeProps {
    modality: Model_Entry['modality'] | string;
}

export function ModalityBadge({ modality }: ModalityBadgeProps) {
    const modalityName = modality || 'llm';
    let colorClass = 'bg-zinc-800/50 border-white/5 text-zinc-500';

    if (modalityName === 'vision') {
        colorClass = 'bg-amber-500/10 border-amber-500/20 text-amber-500';
    } else if (modalityName === 'voice') {
        colorClass = 'bg-green-500/10 border-green-500/20 text-green-500';
    } else if (modalityName === 'reasoning') {
        colorClass = 'bg-green-500/10 border-green-500/20 text-green-500';
    }

    return (
        <span className={`text-[9px] font-bold uppercase tracking-widest px-2 py-0.5 rounded-full border ${colorClass}`}>
            {modalityName}
        </span>
    );
}
