/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Assist Note
 * **UI Sub-Component**: Handles the detached window render path for
 * the Sovereign Chat. Renders a Portal_Window with an embedded
 * Chat_Content instance, plus the restore FAB and popup-blocked warning.
 *
 * ### 🧬 Logic Flow
 * When `is_detached` is active, the main chat UI is unmounted from the
 * layout and re-initialized within a `Portal_Window`.
 *
 * ### 🛰️ Context Persistence
 * Because state is held in global Zustand stores, the detached window
 * retains full parity with the primary workspace without prop-drilling
 * or event-bus remapping.
 *
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Pop-up blocker prevents window creation.
 * - **Telemetry Link**: Search for `[Chat_Detached_Portal]` in browser logs.
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

// Metadata: [Chat_Detached_Portal]
