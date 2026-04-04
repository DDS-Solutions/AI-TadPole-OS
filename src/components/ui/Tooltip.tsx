import React, { useState, useRef, useEffect, useCallback } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { createPortal } from 'react-dom';
import clsx from 'clsx';

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
    const [coords, set_coords] = useState({ x: 0, y: 0 });
    const [actual_position, set_actual_position] = useState(position);
    const trigger_ref = useRef<HTMLDivElement>(null);
    const tooltip_ref = useRef<HTMLDivElement>(null);
    const timeout_ref = useRef<NodeJS.Timeout | null>(null);

    const update_position = useCallback(() => {
        if (!trigger_ref.current) return;
        const rect = trigger_ref.current.getBoundingClientRect();

        let x = 0;
        let y = 0;
        let new_pos = position;

        const padding = 8;
        const view_width = window.innerWidth;
        const view_height = window.innerHeight;

        // Basic edge detection / flipping logic
        if (position === 'top' && rect.top < 40) new_pos = 'bottom';
        if (position === 'bottom' && view_height - rect.bottom < 40) new_pos = 'top';
        if (position === 'left' && rect.left < 80) new_pos = 'right';
        if (position === 'right' && view_width - rect.right < 80) new_pos = 'left';

        set_actual_position(new_pos);

        switch (new_pos) {
            case 'top':
                x = rect.left + rect.width / 2;
                y = rect.top - padding;
                break;
            case 'bottom':
                x = rect.left + rect.width / 2;
                y = rect.bottom + padding;
                break;
            case 'left':
                x = rect.left - padding;
                y = rect.top + rect.height / 2;
                break;
            case 'right':
                x = rect.right + padding;
                y = rect.top + rect.height / 2;
                break;
        }

        set_coords({ x, y });
    }, [position]);

    // Re-calculate position if window resizes while visible
    useEffect(() => {
        if (is_visible) {
            window.addEventListener('scroll', update_position, true);
            window.addEventListener('resize', update_position);
        }
        return () => {
            window.removeEventListener('scroll', update_position, true);
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
                                zIndex: 10000,
                                pointerEvents: 'none',
                            }}
                            className="px-2.5 py-1.5 bg-zinc-900/90 backdrop-blur-md border border-zinc-500/20 rounded-lg text-[10px] font-bold text-zinc-100 uppercase tracking-widest shadow-2xl whitespace-nowrap"
                        >
                            <div className="relative z-10">
                                {content}
                            </div>
                            {/* Optional subtle glow */}
                            <div className="absolute inset-0 bg-blue-500/5 blur-md rounded-lg -z-10" />
                        </motion.div>
                    )}
                </AnimatePresence>,
                document.body
            )}
        </div>
    );
};
