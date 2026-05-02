/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Modality_Badge]` in observability traces.
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

// Metadata: [Modality_Badge]
