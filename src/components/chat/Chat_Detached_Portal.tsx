/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Chat / Chat_Detached_Portal
 * - **Primary Entrypoints**: `Chat_Detached_Portal`, `Chat_Detached_Portal_Props`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import React from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Maximize2 } from 'lucide-react';
import { type Sovereign_Scope } from '../../stores/sovereign_store';
import { Tooltip } from '../ui';
import { Portal_Window } from '../ui/Portal_Window';
import { Chat_Content, type Chat_Content_Props } from './Chat_Content';
import { i18n } from '../../i18n';

export interface Chat_Detached_Portal_Props {
    active_scope: Sovereign_Scope;
    popup_blocked: boolean;
    on_restore: () => void;
    on_popup_block: () => void;
    content_props: Omit<Chat_Content_Props, 'is_detached' | 'drag_controls' | 'on_header_click' | 'container_props'>;
}

/**
 * Chat_Detached_Portal
 * Renders the floating restore button and the Portal_Window containing
 * the full Chat_Content in detached mode.
 */
export const Chat_Detached_Portal: React.FC<Chat_Detached_Portal_Props> = ({
    active_scope,
    popup_blocked,
    on_restore,
    on_popup_block,
    content_props,
}) => {
    return (
        <>
            <div className="fixed bottom-6 right-6 z-50 flex flex-col items-end gap-3">
                <AnimatePresence>
                    {popup_blocked && (
                        <motion.div
                            initial={{ opacity: 0, x: 20 }}
                            animate={{ opacity: 1, x: 0 }}
                            className="bg-red-500/90 text-white text-[10px] font-bold px-3 py-1.5 rounded-lg shadow-xl backdrop-blur-md"
                        >
                            ⚠️ {i18n.t('chat.popup_blocked_warning')}
                        </motion.div>
                    )}
                </AnimatePresence>
                <Tooltip content={i18n.t('chat.restore_tooltip')} position="top">
                    <button
                        onClick={on_restore}
                        aria-label="Restore window"
                        className="bg-[color:color-mix(in_srgb,var(--color-surface)_80%,transparent)] backdrop-blur-md border border-zinc-700/50 p-4 rounded-full text-zinc-400 hover:text-zinc-100 shadow-[0_0_20px_rgba(0,0,0,0.5)] transition-all hover:scale-110 active:scale-95 group"
                    >
                        <Maximize2 size={24} className="group-hover:rotate-12 transition-transform" />
                    </button>
                </Tooltip>
            </div>

            <Portal_Window
                id="sovereign-chat"
                title={`${i18n.t('chat.title')} - ${i18n.t(`chat.scope_${active_scope}`)}`}
                on_close={on_restore}
                on_popup_block={on_popup_block}
                width={440}
                height={720}
                url="/detached/chat"
            >
                <div className="w-full h-full bg-[color:var(--color-background)] text-white overflow-hidden flex flex-col">
                    <Chat_Content
                        {...content_props}
                        is_detached={true}
                        container_props={{}}
                    />
                </div>
            </Portal_Window>
        </>
    );
};
