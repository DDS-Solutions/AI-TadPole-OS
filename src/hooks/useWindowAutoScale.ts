/**
 * @docs ARCHITECTURE:Hooks
 * 
 * ### AI Assist Note
 * **UI Custom Hook**: Window Auto-Scaler.
 * Dynamically computes scale factors based on viewport width/height relative to base dimensions
 * and applies proportional CSS zoom styling to maintain layout bounds in detached windows.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Fallback to 1.0 scale if window dimensions are unavailable.
 * - **Telemetry Link**: Search for `[useWindowAutoScale]` in UI tracing.
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
        };

        update_scale();
        window.addEventListener('resize', update_scale);
        return () => window.removeEventListener('resize', update_scale);
    }, [base_w, base_h, min_scale, max_scale]);
}

// Metadata: [useWindowAutoScale]
