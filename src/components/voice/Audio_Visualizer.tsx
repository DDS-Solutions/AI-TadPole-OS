/**
 * @docs ARCHITECTURE:UI-Components
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Voice / Audio_Visualizer
 * - **Primary Entrypoints**: `Audio_Visualizer`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 * - `[Structural]` Real-time bar heights update via RAF DOM refs without triggering React re-renders.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { useEffect, useRef } from 'react';
import { voice_client } from '../../services/voice_client';

interface AudioVisualizerProps {
    is_active: boolean;
}

const BAR_WEIGHTS = [0.55, 0.78, 0.95, 1.0, 0.9, 0.82, 0.68, 0.5];

export function Audio_Visualizer({ is_active }: AudioVisualizerProps) {
    const bar_refs = useRef<(HTMLDivElement | null)[]>([]);

    useEffect(() => {
        if (!is_active) {
            bar_refs.current.forEach((bar) => {
                if (bar) bar.style.height = '4px';
            });
            return;
        }

        let raf_id: number | null = null;
        let phase = 0;

        const tick = () => {
            phase += 0.12;
            const live_level = voice_client.get_input_level?.() ?? 0;
            bar_refs.current.forEach((bar, i) => {
                if (!bar) return;
                if (live_level > 0.01) {
                    const shaped = Math.min(100, Math.max(15, live_level * BAR_WEIGHTS[i] * 100));
                    bar.style.height = `${shaped.toFixed(1)}%`;
                } else {
                    const idle_wave = 16 + (Math.sin(phase + i * 0.9) * 10 + 10);
                    bar.style.height = `${idle_wave.toFixed(1)}%`;
                }
            });
            raf_id = requestAnimationFrame(tick);
        };

        raf_id = requestAnimationFrame(tick);
        return () => {
            if (raf_id !== null) cancelAnimationFrame(raf_id);
        };
    }, [is_active]);

    return (
        <div
            role="img"
            aria-label={is_active ? 'Live microphone audio visualizer' : 'Audio visualizer idle'}
            data-active={is_active ? 'true' : 'false'}
            className="flex items-end justify-center gap-1 h-12 w-32"
        >
            {BAR_WEIGHTS.map((_, i) => (
                <div
                    key={i}
                    ref={(el) => {
                        bar_refs.current[i] = el;
                    }}
                    className={`w-2 rounded-t transition-[height] duration-75 ${is_active ? 'bg-emerald-500' : 'h-1 bg-zinc-800'}`}
                    style={{ height: is_active ? `${20 + (Math.sin(i * 1.5) * 30 + 30)}%` : '4px' }}
                ></div>
            ))}
        </div>
    );
}
