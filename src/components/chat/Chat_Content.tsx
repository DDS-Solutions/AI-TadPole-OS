/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Assist Note
 * **UI Assembly Component**: Composes the extracted Sovereign Chat sub-components
 * (Header, Breadcrumb, Scope Selector, Message List, Input Bar) into a
 * unified content layout. Used by both the inline and detached (Portal) render paths.
 * Replaces the former inline `SovereignChatContent` that had a 35-prop interface.
 *
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Scroll container not auto-scrolling (if scroll_ref not attached).
 * - **Telemetry Link**: Search for `[Chat_Content]` in React DevTools Profiler.
 */

import React, { useRef, useEffect } from 'react';
import { type DragControls } from 'framer-motion';
import { type Voice_Status } from '../../services/voice_client';
import { type Sovereign_Scope, type Chat_Message } from '../../stores/sovereign_store';
import { type Mission_Cluster } from '../../stores/workspace_store';
import type { Agent } from '../../types';
import { Chat_Header } from './Chat_Header';
import { Chat_Lineage_Breadcrumb } from './Chat_Lineage_Breadcrumb';
import { Chat_Scope_Selector } from './Chat_Scope_Selector';
import { Chat_Message_List } from './Chat_Message_List';
import { Chat_Input_Bar } from './Chat_Input_Bar';
import { Buffered_Transcript_View } from '../transcript/Buffered_Transcript_View';

export interface Chat_Content_Props {
    is_detached: boolean;
    active_scope: Sovereign_Scope;
    target_node: string;
    target_agent: string;
    target_cluster: string;
    selected_agent_id: string | null;

    // Voice state
    is_speaking: boolean;
    voice_status: Voice_Status;

    // Transcript toggle
    show_transcript: boolean;
    set_show_transcript: (show: boolean) => void;

    // Messages
    messages: Chat_Message[];
    max_rendered_messages: number;

    // Input & dispatch
    input_text: string;
    set_input_text: (text: string) => void;
    on_send: () => Promise<void>;

    // Voice controls
    on_toggle_voice: () => void;
    on_toggle_speech: () => void;
    is_speech_enabled: boolean;

    // Safety
    on_toggle_safety: () => void;
    is_safe_mode: boolean;

    // Window controls
    on_toggle_detach: () => void;
    on_clear_history: () => void;
    on_minimize: () => void;

    // Scope & target
    on_set_scope: (scope: Sovereign_Scope) => void;
    open_dropdown: 'agent' | 'cluster' | null;
    set_open_dropdown: (val: 'agent' | 'cluster' | null) => void;
    sorted_agents: Agent[];
    set_target_agent: (name: string) => void;
    set_selected_agent_id: (id: string) => void;
    set_target_cluster: (name: string) => void;
    clusters: Mission_Cluster[];

    // Optional: inline-only
    drag_controls?: DragControls;
    on_header_click?: () => void;
    container_props?: React.HTMLAttributes<HTMLDivElement>;
}

/**
 * Chat_Content
 * Assembly component that composes Header, Breadcrumb, Scope Selector,
 * Message List / Transcript View, and Input Bar.
 * Used by both inline and Portal render paths.
 */
export const Chat_Content: React.FC<Chat_Content_Props> = ({
    is_detached,
    active_scope,
    target_node,
    target_agent,
    target_cluster,
    selected_agent_id,
    is_speaking,
    voice_status,
    show_transcript,
    set_show_transcript,
    messages,
    max_rendered_messages,
    input_text,
    set_input_text,
    on_send,
    on_toggle_voice,
    on_toggle_speech,
    is_speech_enabled,
    on_toggle_safety,
    is_safe_mode,
    on_toggle_detach,
    on_clear_history,
    on_minimize,
    on_set_scope,
    open_dropdown,
    set_open_dropdown,
    sorted_agents,
    set_target_agent,
    set_selected_agent_id,
    set_target_cluster,
    clusters,
    drag_controls,
    on_header_click,
    container_props,
}) => {
    const scroll_ref = useRef<HTMLDivElement>(null);

    useEffect(() => {
        if (scroll_ref.current) {
            scroll_ref.current.scrollTo({
                top: scroll_ref.current.scrollHeight,
                behavior: 'smooth'
            });
        }
    }, [messages]);

    return (
        <div className="w-full h-full flex flex-col relative" {...container_props}>
            {!is_detached && <div className="neural-grid opacity-[0.05] absolute inset-0 pointer-events-none" />}

            <Chat_Header
                is_detached={is_detached}
                active_scope={active_scope}
                target_node={target_node}
                is_speaking={is_speaking}
                voice_status={voice_status}
                show_transcript={show_transcript}
                on_toggle_transcript={() => set_show_transcript(!show_transcript)}
                on_minimize={on_minimize}
                on_toggle_detach={on_toggle_detach}
                on_clear_history={on_clear_history}
                drag_controls={drag_controls}
                on_header_click={on_header_click}
            />

            {active_scope === 'agent' && (
                <Chat_Lineage_Breadcrumb target_agent={target_agent} />
            )}

            <Chat_Scope_Selector
                active_scope={active_scope}
                on_set_scope={on_set_scope}
                target_agent={target_agent}
                target_cluster={target_cluster}
                open_dropdown={open_dropdown}
                set_open_dropdown={set_open_dropdown}
                sorted_agents={sorted_agents}
                clusters={clusters}
                set_target_agent={set_target_agent}
                set_selected_agent_id={set_selected_agent_id}
                set_target_cluster={set_target_cluster}
                set_input_text={set_input_text}
            />

            {/* Messages Window */}
            <div
                ref={scroll_ref}
                className="relative z-10 flex-1 overflow-y-auto p-5 space-y-6 custom-scrollbar"
            >
                {show_transcript ? (
                    <Buffered_Transcript_View agent_id={selected_agent_id || undefined} />
                ) : (
                    <Chat_Message_List
                        messages={messages}
                        max_rendered={max_rendered_messages}
                        active_scope={active_scope}
                        target_node={target_node}
                    />
                )}
            </div>

            <Chat_Input_Bar
                active_scope={active_scope}
                is_safe_mode={is_safe_mode}
                is_speech_enabled={is_speech_enabled}
                is_speaking={is_speaking}
                input_value={input_text}
                on_change={set_input_text}
                on_send={on_send}
                on_toggle_voice={on_toggle_voice}
                on_toggle_speech={on_toggle_speech}
                on_toggle_safety={on_toggle_safety}
                is_listening={voice_status !== 'idle'}
            />
        </div>
    );
};

// Metadata: [Chat_Content]
