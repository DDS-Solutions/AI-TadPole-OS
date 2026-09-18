/**
 * @docs ARCHITECTURE:UI-Components
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Template_Store / Error_State
 * - **Primary Entrypoints**: `Error_State`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { AlertTriangle } from 'lucide-react';

interface ErrorStateProps {
    error: string;
}

export function Error_State({ error }: ErrorStateProps) {
    return (
        <div className="flex flex-col items-center justify-center py-20 text-red-400">
            <AlertTriangle size={48} className="mb-4 opacity-50" />
            <p>{error}</p>
        </div>
    );
}
