/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Portal-based contextual metadata provider. 
 * Employs real-time edge detection, smart flipping (top/bottom/left/right), and glassmorphic aesthetics for zero-collision viewport rendering.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Flip failure (viewport too small), portal detaching from main tree during navigation, or focus starvation for screen readers.
 * - **Telemetry Link**: Search for `[Tooltip]` or `createPortal` in UI tracing.
 */

import React, { useState, useRef, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { createPortal } from 'react-dom';
import clsx from 'clsx';
import { Z_INDEX_MAP } from './theme_tokens';
import { useViewportPosition } from '../../hooks/use_viewport_position';

interface Tooltip_Props {
    content: string | React.ReactNode;
    children: React.ReactNode;
    position?: 'top' | 'bottom' | 'left' | 'right';
    delay?: number;
    class_name?: string;
}

/**
 * Tooltip
 * A premium, glassmorphic tooltip with smart positioning and smooth animations.
 * Features automatic edge detection and portal-based rendering.
 */
export const Tooltip: React.FC<Tooltip_Props> = ({
    content,
    children,
    position = 'top',
    delay = 300,
    class_name
}) => {
    const [is_visible, set_is_visible] = useState(false);
    const trigger_ref = useRef<HTMLDivElement>(null);
    const tooltip_ref = useRef<HTMLDivElement>(null);
    const timeout_ref = useRef<NodeJS.Timeout | null>(null);

    const { coords, actual_position, update_position } = useViewportPosition({
        trigger_ref,
        content_ref: tooltip_ref,
        position,
        is_visible,
        offset: 8,
        padding: 8
    });

    // Re-calculate position if window resizes or scrolls while visible
    useEffect(() => {
        const handle_scroll = () => {
            set_is_visible(false); // Hide on scroll to prevent flickering
        };

        if (is_visible) {
            window.addEventListener('scroll', handle_scroll, true);
            window.addEventListener('resize', update_position);
        }
        return () => {
            window.removeEventListener('scroll', handle_scroll, true);
            window.removeEventListener('resize', update_position);
        };
    }, [is_visible, update_position]);

    const handle_mouse_enter = () => {
        timeout_ref.current = setTimeout(() => {
            update_position();
            set_is_visible(true);
        }, delay);
    };

    const handle_mouse_leave = () => {
        if (timeout_ref.current) clearTimeout(timeout_ref.current);
        set_is_visible(false);
    };

    return (
        <div
            ref={trigger_ref}
            className={clsx("inline-block", class_name)}
            onMouseEnter={handle_mouse_enter}
            onMouseLeave={handle_mouse_leave}
        >
            {children}
            {createPortal(
                <AnimatePresence>
                    {is_visible && (
                        <motion.div
                            key="tooltip"
                            ref={tooltip_ref}
                            role="tooltip"
                            initial={{ opacity: 0, scale: 0.9, y: actual_position === 'top' ? 5 : actual_position === 'bottom' ? -5 : 0 }}
                            animate={{ opacity: 1, scale: 1, y: 0 }}
                            exit={{ opacity: 0, scale: 0.9 }}
                            transition={{ duration: 0.15, ease: "easeOut" }}
                            style={{
                                position: 'fixed',
                                left: coords.x,
                                top: coords.y,
                                transform: actual_position === 'top' ? 'translate(-50%, -100%)' :
                                    actual_position === 'bottom' ? 'translate(-50%, 0)' :
                                        actual_position === 'left' ? 'translate(-100%, -50%)' :
                                            'translate(0, -50%)',
                                zIndex: Z_INDEX_MAP.tooltip,
                                pointerEvents: 'none',
                            }}
                            className="px-2.5 py-1.5 bg-zinc-900/90 backdrop-blur-md border border-zinc-500/20 rounded-lg text-[10px] font-bold text-zinc-100 uppercase tracking-widest shadow-2xl whitespace-nowrap"
                        >
                            <div className="relative z-10">
                                {content}
                            </div>
                            {/* Optional subtle glow */}
                            <div className="absolute inset-0 bg-green-500/5 blur-md rounded-lg -z-10" />
                        </motion.div>
                    )}
                </AnimatePresence>,
                document.getElementById('portal-root') || document.body
            )}
        </div>
    );
};


// Metadata: [Tooltip]
