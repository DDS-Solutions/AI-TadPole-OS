/**
 * @docs ARCHITECTURE:Logic
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend React Hooks / use_chat_window
 * - **Primary Entrypoints**: `useChatWindow`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` React hook lifecycle adheres to Rules of Hooks without conditional execution branches.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { useState, useRef, useEffect, useCallback } from 'react';
import { useMotionValue } from 'framer-motion';
import { use_sovereign_store } from '../stores/sovereign_store';

// Centralized layout dimensions (matching SovereignChat.tsx w-[440px] h-[600px] styling)
const CHAT_PANEL_WIDTH = 440;
const CHAT_PANEL_HEIGHT = 600;
const BUTTON_WIDTH = 220;
const BUTTON_HEIGHT = 48;
const PADDING = 48; // Combined margins/padding (24px each side)

const Y_TRANSLATION = -(CHAT_PANEL_HEIGHT - BUTTON_HEIGHT); // -552px
const X_TRANSLATION = -(CHAT_PANEL_WIDTH - BUTTON_WIDTH);   // -220px

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
            const max_neg_y = -(h - CHAT_PANEL_HEIGHT - PADDING);
            const max_neg_x = -(w - CHAT_PANEL_WIDTH - PADDING);

            if (y_open.get() < max_neg_y) y_open.set(Math.min(0, max_neg_y));
            if (x_open.get() < max_neg_x) x_open.set(Math.min(0, max_neg_x));

            if (y_open.get() > 0) y_open.set(0);
            if (x_open.get() > 0) x_open.set(0);
        }
    }, [is_minimized, x_open, y_open]);

    // Re-clamp window positions dynamically when viewport changes (debounced resize listener)
    useEffect(() => {
        let resize_timeout: ReturnType<typeof setTimeout>;

        const handle_resize = () => {
            const h = window.innerHeight;
            const w = window.innerWidth;
            const max_neg_y_open = -(h - CHAT_PANEL_HEIGHT - PADDING);
            const max_neg_x_open = -(w - CHAT_PANEL_WIDTH - PADDING);

            if (is_minimized) {
                const max_neg_y_min = -(h - BUTTON_HEIGHT - PADDING);
                const max_neg_x_min = -(w - BUTTON_WIDTH - PADDING);
                if (y_min.get() < max_neg_y_min) y_min.set(Math.min(0, max_neg_y_min));
                if (x_min.get() < max_neg_x_min) x_min.set(Math.min(0, max_neg_x_min));
                if (y_min.get() > 0) y_min.set(0);
                if (x_min.get() > 0) x_min.set(0);
            } else {
                if (y_open.get() < max_neg_y_open) y_open.set(Math.min(0, max_neg_y_open));
                if (x_open.get() < max_neg_x_open) x_open.set(Math.min(0, max_neg_x_open));
                if (y_open.get() > 0) y_open.set(0);
                if (x_open.get() > 0) x_open.set(0);
            }
        };

        const debounced_resize = () => {
            clearTimeout(resize_timeout);
            resize_timeout = setTimeout(handle_resize, 150);
        };

        window.addEventListener('resize', debounced_resize);
        return () => {
            clearTimeout(resize_timeout);
            window.removeEventListener('resize', debounced_resize);
        };
    }, [is_minimized, x_open, y_open, x_min, y_min]);

    const toggle_detach = useCallback(() => {
        console.debug(`${telemetry_source} Toggling detachment: ${!is_detached}`);
        set_detached(!is_detached);
        if (!is_detached) set_is_minimized(false); // Reset minimized state if detaching
    }, [is_detached, set_detached]);

    const perform_minimize_transform = useCallback(() => {
        console.debug(`${telemetry_source} Initiating minimize transform`);
        const h = window.innerHeight;
        const w = window.innerWidth;
        const max_neg_y_open = Math.min(-1, -(h - CHAT_PANEL_HEIGHT - PADDING));
        const max_neg_x_open = Math.min(-1, -(w - CHAT_PANEL_WIDTH - PADDING));

        // Safe ratio 0 to 1 depending on where open window sits
        const ratio_y = Math.min(1, Math.max(0, y_open.get() / max_neg_y_open));
        const ratio_x = Math.min(1, Math.max(0, x_open.get() / max_neg_x_open));

        // Scale offsets proportionally
        const y_shift = Y_TRANSLATION * ratio_y;
        const x_shift = X_TRANSLATION * ratio_x;

        x_min.set(x_open.get() + x_shift);
        y_min.set(y_open.get() + y_shift);
        set_is_minimized(true);
    }, [x_open, y_open, x_min, y_min]);

    const perform_maximize_transform = useCallback(() => {
        console.debug(`${telemetry_source} Initiating maximize transform`);
        const h = window.innerHeight;
        const w = window.innerWidth;
        const max_neg_y_min = Math.min(-1, -(h - BUTTON_HEIGHT - PADDING));
        const max_neg_x_min = Math.min(-1, -(w - BUTTON_WIDTH - PADDING));

        const ratio_y = Math.min(1, Math.max(0, y_min.get() / max_neg_y_min));
        const ratio_x = Math.min(1, Math.max(0, x_min.get() / max_neg_x_min));

        const y_shift = -Y_TRANSLATION * ratio_y;
        const x_shift = -X_TRANSLATION * ratio_x;

        y_open.set(y_min.get() + y_shift);
        x_open.set(x_min.get() + x_shift);

        const max_neg_y_open = -(h - CHAT_PANEL_HEIGHT - PADDING);
        const max_neg_x_open = -(w - CHAT_PANEL_WIDTH - PADDING);

        if (y_open.get() < max_neg_y_open) y_open.set(Math.min(0, max_neg_y_open));
        if (x_open.get() < max_neg_x_open) x_open.set(Math.min(0, max_neg_x_open));
        if (y_open.get() > 0) y_open.set(0);
        if (x_open.get() > 0) x_open.set(0);

        set_is_minimized(false);
    }, [x_open, y_open, x_min, y_min]);

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
