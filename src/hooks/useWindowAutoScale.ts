/**
 * @docs ARCHITECTURE:Hooks
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend React Hooks / useWindowAutoScale
 * - **Primary Entrypoints**: `useWindowAutoScale`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` React hook lifecycle adheres to Rules of Hooks without conditional execution branches.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: `[useWindowAutoScale]`
 * - **Witness Tests**: none declared
 */

import { useEffect } from 'react';

export function useWindowAutoScale(base_w = 1200, base_h = 800, min_scale = 0.55, max_scale = 1.0) {
    useEffect(() => {
        const update_scale = () => {
            const current_w = window.innerWidth || base_w;
            const current_h = window.innerHeight || base_h;
            const scale_w = current_w / base_w;
            const scale_h = current_h / base_h;
            const scale = Math.min(max_scale, Math.max(min_scale, Math.min(scale_w, scale_h)));

            const target = document.body;
            if (target) {
                const style = target.style as CSSStyleDeclaration & { zoom?: string };
                if (typeof style.zoom !== 'undefined') {
                    style.zoom = scale.toFixed(3);
                }
            }

            if (process.env.NODE_ENV === 'development') {
                console.debug(`[useWindowAutoScale] Viewport auto-scale factor computed: ${scale.toFixed(3)}`);
            }
        };

        update_scale();
        window.addEventListener('resize', update_scale);
        return () => window.removeEventListener('resize', update_scale);
    }, [base_w, base_h, min_scale, max_scale]);
}
