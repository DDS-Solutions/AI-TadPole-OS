/**
 * @docs ARCHITECTURE:UI-Components
 *
 * ### AI Assist Note
 * - Portals parameter viewer outside scroll containers to prevent boundary clipping.
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Oversight / Params_Cell
 * - **Primary Entrypoints**: `Params_Cell`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import React, { useState, useRef, useEffect, useMemo } from 'react';
import { createPortal } from 'react-dom';
import { X } from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';
import clsx from 'clsx';
import { Z_INDEX_MAP } from '../ui/theme_tokens';
import { useViewportPosition } from '../../hooks/use_viewport_position';
import { i18n } from '../../i18n';

export interface ParamsCellProps {
    params: unknown;
}

const EMPTY_PARAMS: Record<string, unknown> = {};

export const Params_Cell: React.FC<ParamsCellProps> = ({ params }) => {
    const [is_open, set_is_open] = useState(false);
    const [is_hovered, set_is_hovered] = useState(false);
    const cell_ref = useRef<HTMLDivElement>(null);
    const popover_ref = useRef<HTMLDivElement>(null);

    const show_box = is_open || is_hovered;

    const { coords, actual_position, update_position } = useViewportPosition({
        trigger_ref: cell_ref,
        content_ref: popover_ref,
        position: 'top',
        is_visible: show_box,
        offset: 8,
        padding: 8
    });

    const raw_params = params || EMPTY_PARAMS;
    const formatted_json = useMemo(() => {
        if (!show_box) return '';
        if (typeof raw_params === 'string') {
            try {
                return JSON.stringify(JSON.parse(raw_params), null, 2);
            } catch {
                return raw_params;
            }
        }
        return JSON.stringify(raw_params, null, 2);
    }, [raw_params, show_box]);

    const inline_json = useMemo(() => {
        return typeof raw_params === 'string' ? raw_params : JSON.stringify(raw_params);
    }, [raw_params]);

    // Handle scroll/resize updates while visible
    useEffect(() => {
        if (!show_box) return;

        const handle_scroll_or_resize = () => {
            update_position();
        };

        window.addEventListener('scroll', handle_scroll_or_resize, true);
        window.addEventListener('resize', handle_scroll_or_resize);

        return () => {
            window.removeEventListener('scroll', handle_scroll_or_resize, true);
            window.removeEventListener('resize', handle_scroll_or_resize);
        };
    }, [show_box, update_position]);

    // Handle click outside to close pinned popover
    useEffect(() => {
        if (!is_open) return;

        const handle_click_outside = (e: MouseEvent) => {
            const target = e.target as Node;
            if (
                cell_ref.current && !cell_ref.current.contains(target) &&
                popover_ref.current && !popover_ref.current.contains(target)
            ) {
                set_is_open(false);
            }
        };

        const handle_key_down = (e: KeyboardEvent) => {
            if (e.key === 'Escape') {
                set_is_open(false);
            }
        };

        document.addEventListener('mousedown', handle_click_outside);
        document.addEventListener('keydown', handle_key_down);
        return () => {
            document.removeEventListener('mousedown', handle_click_outside);
            document.removeEventListener('keydown', handle_key_down);
        };
    }, [is_open]);

    return (
        <div 
            ref={cell_ref} 
            className="relative inline-block"
            onMouseEnter={() => {
                update_position();
                set_is_hovered(true);
            }}
            onMouseLeave={() => set_is_hovered(false)}
        >
            <button
                type="button"
                onClick={() => {
                    update_position();
                    set_is_open(prev => !prev);
                }}
                className={clsx(
                    "max-w-[140px] truncate text-[10px] font-mono px-2 py-1 rounded border transition-all cursor-pointer select-none text-left flex items-center justify-between gap-1",
                    is_open 
                        ? "bg-green-500/20 text-green-300 border-green-500/50 ring-1 ring-green-500/30" 
                        : "bg-zinc-950/30 hover:bg-zinc-950/50 text-zinc-400 border-zinc-800/50 hover:text-zinc-200"
                )}
                title={i18n.t('oversight.click_to_pin_params') || "Click to pin parameters box"}
            >
                <span className="truncate">{inline_json}</span>
            </button>

            {/* Pinned / Hover Popover Box Portaled to Document Body to ensure zero clipping & top z-index */}
            {createPortal(
                <AnimatePresence>
                    {show_box && (
                        <motion.div
                            key="params-popover"
                            ref={popover_ref}
                            initial={{ opacity: 0, scale: 0.95, y: actual_position === 'top' ? 4 : -4 }}
                            animate={{ opacity: 1, scale: 1, y: 0 }}
                            exit={{ opacity: 0, scale: 0.95 }}
                            transition={{ duration: 0.15, ease: "easeOut" }}
                            style={{
                                position: 'fixed',
                                left: coords.x,
                                top: coords.y,
                                transform: actual_position === 'top' 
                                    ? 'translate(-50%, -100%)' 
                                    : 'translate(-50%, 0)',
                                zIndex: Z_INDEX_MAP.dialog + 50,
                            }}
                            className={clsx(
                                "w-96 max-w-md bg-zinc-950/95 border rounded-xl shadow-2xl p-3 backdrop-blur-xl flex flex-col text-left normal-case tracking-normal pointer-events-auto",
                                is_open ? "border-green-500/60 ring-2 ring-green-500/30 shadow-green-950/50" : "border-zinc-700/60"
                            )}
                            onMouseEnter={() => set_is_hovered(true)}
                            onMouseLeave={() => set_is_hovered(false)}
                            onClick={(e) => e.stopPropagation()}
                        >
                            {/* Box Header */}
                            <div className="text-[9px] font-bold text-zinc-400 uppercase tracking-wider mb-2 pb-1.5 border-b border-zinc-800/80 flex items-center justify-between select-none">
                                <span className="flex items-center gap-1.5 text-zinc-300">
                                    <span className={clsx("w-2 h-2 rounded-full", is_open ? "bg-green-400 animate-pulse" : "bg-cyan-400")} />
                                    {i18n.t('oversight.table_params') || 'Action Parameters'}
                                    {is_open && (
                                        <span className="text-[8px] bg-green-500/10 text-green-400 px-1.5 py-0.5 rounded border border-green-500/20 ml-1 font-bold">
                                            PINNED
                                        </span>
                                    )}
                                </span>

                                <div className="flex items-center gap-2">
                                    <span className="text-[8px] text-zinc-500 font-mono">JSON</span>
                                    {is_open && (
                                        <button
                                            type="button"
                                            onClick={() => set_is_open(false)}
                                            className="p-0.5 text-zinc-500 hover:text-zinc-200 hover:bg-zinc-800 rounded transition-colors cursor-pointer"
                                            title="Close"
                                        >
                                            <X size={12} />
                                        </button>
                                    )}
                                </div>
                            </div>

                            {/* Box Body (Scrollable formatted pre block) */}
                            <div className="relative">
                                <pre className="font-mono text-[11px] text-zinc-300 leading-relaxed overflow-auto custom-scrollbar p-2.5 bg-zinc-950/60 rounded-lg border border-zinc-800/60 max-h-64 whitespace-pre-wrap break-all select-text">
                                    {formatted_json}
                                </pre>
                            </div>

                            {/* Footer hint if hovered but not pinned */}
                            {!is_open && (
                                <div className="mt-1.5 text-[8px] font-mono text-zinc-500 text-right select-none">
                                    {i18n.t('oversight.click_to_lock') || 'Click box to pin open & scroll'}
                                </div>
                            )}
                        </motion.div>
                    )}
                </AnimatePresence>,
                document.getElementById('portal-root') || document.body
            )}
        </div>
    );
};

