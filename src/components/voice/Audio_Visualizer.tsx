/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Audio_Visualizer]` in observability traces.
 */

interface AudioVisualizerProps {
    is_active: boolean;
}

export function Audio_Visualizer({ is_active }: AudioVisualizerProps) {
    return (
        <div className="flex items-end justify-center gap-1 h-12 w-32">
            {[...Array(8)].map((_, i) => (
                <div
                    key={i}
                    className={`w-2 bg-emerald-500 rounded-t transition-all duration-150 ${is_active ? 'animate-pulse' : 'h-1 bg-zinc-800'}`}
                    style={{ height: is_active ? `${20 + (Math.sin(i * 1.5) * 30 + 30)}%` : '4px' }}
                ></div>
            ))}
        </div>
    );
}

// Metadata: [Audio_Visualizer]
