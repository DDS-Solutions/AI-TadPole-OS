/**
 * @docs ARCHITECTURE:Logic
 * @docs OPERATIONS_MANUAL:Navigation
 * 
 * ### AI Assist Note
 * **UI State Hook**: Manages the positioning, minimization, and detachment logic for the "Sovereign Chat" window. 
 * Implements proportional scaling for window transforms using `framer-motion` to prevent off-screen spawning.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Viewport overflow (if `window.innerHeight` changes unexpectedly), detachment failure (if pop-up blocker is active), or transform ratio glitches on mobile.
 * - **Telemetry Link**: Uses `useMotionValue` for `x`/`y` vectors. Search `[useChatWindow]` in UI traces.
 */

import { useState, useRef, useEffect } from 'react';
import { useMotionValue } from 'framer-motion';
import { use_sovereign_store } from '../stores/sovereign_store';

export function useChatWindow() {
    const { is_detached, set_detached } = use_sovereign_store();
    const [is_minimized, set_is_minimized] = useState(false);
    const telemetry_source = '[useChatWindow]';

    const constraints_ref = useRef<HTMLDivElement>(null);

    // Shared drag state for both open and minimized states
    const x_open = useMotionValue(0);
    const y_open = useMotionValue(0);
    const x_min = useMotionValue(0);
    const y_min = useMotionValue(0);

    // Prevent open window from spawning off-screen if maximized from a high minimized position
    useEffect(() => {
        if (!is_minimized) {
            const h = window.innerHeight;
            const w = window.innerWidth;
            const max_neg_y = -(h - 600 - 48); // 600px height + 48px combined padding
            const max_neg_x = -(w - 400 - 48);  // 400px width + 48px combined padding

            if (y_open.get() < max_neg_y) y_open.set(Math.min(0, max_neg_y));
            if (x_open.get() < max_neg_x) x_open.set(Math.min(0, max_neg_x));

            if (y_open.get() > 0) y_open.set(0);
            if (x_open.get() > 0) x_open.set(0);
        }
    }, [is_minimized, x_open, y_open]);

    const toggle_detach = () => {
        console.debug(`${telemetry_source} Toggling detachment: ${!is_detached}`);
        set_detached(!is_detached);
        if (!is_detached) set_is_minimized(false); // Reset minimized state if detaching
    };

    const perform_minimize_transform = () => {
        console.debug(`${telemetry_source} Initiating minimize transform`);
        const h = window.innerHeight;
        const w = window.innerWidth;
        const max_neg_y_open = Math.min(-1, -(h - 600 - 48)); // 600px height, 48px combined padding
        const max_neg_x_open = Math.min(-1, -(w - 400 - 48)); // 400px width

        // Safe ratio 0 to 1 depending on where open window sits
        const ratio_y = Math.min(1, Math.max(0, y_open.get() / max_neg_y_open));
        const ratio_x = Math.min(1, Math.max(0, x_open.get() / max_neg_x_open));

        // Scale offsets proportionally
        const y_shift = -552 * ratio_y;
        const x_shift = -180 * ratio_x;

        x_min.set(x_open.get() + x_shift);
        y_min.set(y_open.get() + y_shift);
        set_is_minimized(true);
    };

    const perform_maximize_transform = () => {
        console.debug(`${telemetry_source} Initiating maximize transform`);
        const h = window.innerHeight;
        const w = window.innerWidth;
        const max_neg_y_min = Math.min(-1, -(h - 48 - 48)); // 48px height approx button
        const max_neg_x_min = Math.min(-1, -(w - 220 - 48)); // 220px width approx button

        const ratio_y = Math.min(1, Math.max(0, y_min.get() / max_neg_y_min));
        const ratio_x = Math.min(1, Math.max(0, x_min.get() / max_neg_x_min));

        const y_shift = 552 * ratio_y;
        const x_shift = 180 * ratio_x;

        y_open.set(y_min.get() + y_shift);
        x_open.set(x_min.get() + x_shift);

        const max_neg_y_open = -(h - 600 - 48);
        const max_neg_x_open = -(w - 400 - 48);

        if (y_open.get() < max_neg_y_open) y_open.set(Math.min(0, max_neg_y_open));
        if (x_open.get() < max_neg_x_open) x_open.set(Math.min(0, max_neg_x_open));
        if (y_open.get() > 0) y_open.set(0);
        if (x_open.get() > 0) x_open.set(0);

        set_is_minimized(false);
    };

    return {
        is_detached,
        is_minimized,
        constraints_ref,
        x_open,
        y_open,
        x_min,
        y_min,
        set_detached,
        toggle_detach,
        perform_minimize_transform,
        perform_maximize_transform
    };
}

