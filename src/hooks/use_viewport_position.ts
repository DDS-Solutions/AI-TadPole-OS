/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Assist Note
 * **Hook**: Calculates the optimal viewport-constrained position for contextual UI overlays
 * (tooltips, dropdowns). Handles smart flipping when near viewport edges and sub-pixel
 * clamping via a two-pass layout effect.
 *
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Tooltip renders off-screen when `content_ref` dimensions are 0
 *   (element not yet painted). Ensure `is_visible` is only `true` after mount.
 * - **Telemetry Link**: Search `[use_viewport_position]` in UI interaction traces.
 */

import { useState, useCallback, useLayoutEffect } from 'react';
import type { RefObject } from 'react';

export type Position = 'top' | 'bottom' | 'left' | 'right';

interface UseViewportPositionProps {
    trigger_ref: RefObject<HTMLElement | null>;
    content_ref: RefObject<HTMLElement | null>;
    position?: Position;
    padding?: number;
    offset?: number;
    is_visible?: boolean;
}

/**
 * use_viewport_position
 * Standardized hook for calculating the position of a contextual UI element (tooltip, dropdown)
 * relative to a trigger element, with smart flipping and viewport boundary constraints.
 */
export const useViewportPosition = ({
    trigger_ref,
    content_ref,
    position = 'top',
    padding = 8,
    offset = 8,
    is_visible = false,
}: UseViewportPositionProps) => {
    const [coords, set_coords] = useState({ x: 0, y: 0 });
    const [actual_position, set_actual_position] = useState<Position>(position);

    const update_position = useCallback(() => {
        if (!trigger_ref.current) return;
        const rect = trigger_ref.current.getBoundingClientRect();

        let x = 0;
        let y = 0;
        let new_pos = position;

        const view_width = window.innerWidth;
        const view_height = window.innerHeight;

        // Base flipping logic
        if (position === 'top' && rect.top < 60) new_pos = 'bottom';
        if (position === 'bottom' && view_height - rect.bottom < 60) new_pos = 'top';
        if (position === 'left' && rect.left < 100) new_pos = 'right';
        if (position === 'right' && view_width - rect.right < 100) new_pos = 'left';

        set_actual_position(new_pos);

        switch (new_pos) {
            case 'top':
                x = rect.left + rect.width / 2;
                y = rect.top - offset;
                break;
            case 'bottom':
                x = rect.left + rect.width / 2;
                y = rect.bottom + offset;
                break;
            case 'left':
                x = rect.left - offset;
                y = rect.top + rect.height / 2;
                break;
            case 'right':
                x = rect.right + offset;
                y = rect.top + rect.height / 2;
                break;
        }

        set_coords({ x, y });
    }, [position, offset, trigger_ref]);

    useLayoutEffect(() => {
        if (is_visible && content_ref.current && trigger_ref.current) {
            const content_rect = content_ref.current.getBoundingClientRect();
            const view_width = window.innerWidth;
            const view_height = window.innerHeight;
            
            let { x, y } = coords;
            let adjusted = false;

            const max_width = view_width - padding * 2;
            const content_width = content_rect.width;

            // Constrain horizontally: if content is wider than viewport, center it
            if (content_width >= max_width) {
                x = view_width / 2;
                adjusted = true;
            } else {
                if (x - content_width / 2 < padding) {
                    x = content_width / 2 + padding;
                    adjusted = true;
                } else if (x + content_width / 2 > view_width - padding) {
                    x = view_width - content_width / 2 - padding;
                    adjusted = true;
                }
            }

            const max_height = view_height - padding * 2;
            const content_height = content_rect.height;

            // Constrain vertically: if content is taller than viewport, center it
            if (content_height >= max_height) {
                y = view_height / 2;
                adjusted = true;
            } else {
                if (y - content_height < padding && actual_position === 'top') {
                    y = padding + content_height;
                    adjusted = true;
                } else if (y + content_height > view_height - padding && actual_position === 'bottom') {
                    y = view_height - content_height - padding;
                    adjusted = true;
                }
            }

            if (adjusted && (x !== coords.x || y !== coords.y)) {
                // Intentional two-pass layout measurement: useLayoutEffect reads DOM dimensions
                // post-paint and clamps coords synchronously before repaint (per React docs).
                // eslint-disable-next-line react-hooks/set-state-in-effect
                set_coords({ x, y });
            }
        }
    }, [is_visible, coords, actual_position, padding, content_ref, trigger_ref]);

    return { coords, actual_position, update_position };
};

/**
 * snake_case alias for backward compatibility.
 * New code should import `useViewportPosition` directly.
 */
export const use_viewport_position = useViewportPosition;



// Metadata: [use_viewport_position]
