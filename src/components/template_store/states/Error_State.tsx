/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Error_State]` in observability traces.
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

// Metadata: [Error_State]
