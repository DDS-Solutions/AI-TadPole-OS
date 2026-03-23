import React, { useState, useRef, useEffect, useCallback } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { createPortal } from 'react-dom';
import clsx from 'clsx';

interface TooltipProps {
    content: string | React.ReactNode;
    children: React.ReactNode;
    position?: 'top' | 'bottom' | 'left' | 'right';
    delay?: number;
    className?: string;
}

/**
 * Tooltip Component
 * A premium, glassmorphic tooltip with smart positioning and smooth animations.
 */
export const Tooltip: React.FC<TooltipProps> = ({
    content,
    children,
    position = 'top',
    delay = 300,
    className
}) => {
    const [isVisible, setIsVisible] = useState(false);
    const [coords, setCoords] = useState({ x: 0, y: 0 });
    const [actualPosition, setActualPosition] = useState(position);
    const triggerRef = useRef<HTMLDivElement>(null);
    const tooltipRef = useRef<HTMLDivElement>(null);
    const timeoutRef = useRef<NodeJS.Timeout | null>(null);

    const updatePosition = useCallback(() => {
        if (!triggerRef.current) return;
        const rect = triggerRef.current.getBoundingClientRect();

        let x = 0;
        let y = 0;
        let newPos = position;

        const padding = 8;
        const viewWidth = window.innerWidth;
        const viewHeight = window.innerHeight;

        // Basic edge detection / flipping logic
        if (position === 'top' && rect.top < 40) newPos = 'bottom';
        if (position === 'bottom' && viewHeight - rect.bottom < 40) newPos = 'top';
        if (position === 'left' && rect.left < 80) newPos = 'right';
        if (position === 'right' && viewWidth - rect.right < 80) newPos = 'left';

        setActualPosition(newPos);

        switch (newPos) {
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

        setCoords({ x, y });
    }, [position]);

    // Re-calculate position if window resizes while visible
    useEffect(() => {
        if (isVisible) {
            window.addEventListener('scroll', updatePosition, true);
            window.addEventListener('resize', updatePosition);
        }
        return () => {
            window.removeEventListener('scroll', updatePosition, true);
            window.removeEventListener('resize', updatePosition);
        };
    }, [isVisible, updatePosition]);

    const handleMouseEnter = () => {
        timeoutRef.current = setTimeout(() => {
            updatePosition();
            setIsVisible(true);
        }, delay);
    };

    const handleMouseLeave = () => {
        if (timeoutRef.current) clearTimeout(timeoutRef.current);
        setIsVisible(false);
    };

    return (
        <div
            ref={triggerRef}
            className={clsx("inline-block", className)}
            onMouseEnter={handleMouseEnter}
            onMouseLeave={handleMouseLeave}
        >
            {children}
            {createPortal(
                <AnimatePresence>
                    {isVisible && (
                        <motion.div
                            key="tooltip"
                            ref={tooltipRef}
                            role="tooltip"
                            initial={{ opacity: 0, scale: 0.9, y: actualPosition === 'top' ? 5 : actualPosition === 'bottom' ? -5 : 0 }}
                            animate={{ opacity: 1, scale: 1, y: 0 }}
                            exit={{ opacity: 0, scale: 0.9 }}
                            transition={{ duration: 0.15, ease: "easeOut" }}
                            style={{
                                position: 'fixed',
                                left: coords.x,
                                top: coords.y,
                                transform: actualPosition === 'top' ? 'translate(-50%, -100%)' :
                                    actualPosition === 'bottom' ? 'translate(-50%, 0)' :
                                        actualPosition === 'left' ? 'translate(-100%, -50%)' :
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
