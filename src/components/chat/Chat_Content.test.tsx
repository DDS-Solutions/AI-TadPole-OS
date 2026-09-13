/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Chat / Chat_Content.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import '@testing-library/jest-dom';
import { render, screen } from '@testing-library/react';
import { describe, it, expect, vi, afterEach } from 'vitest';
import { Chat_Content } from './Chat_Content';

// Mock i18n
vi.mock('../../i18n', () => ({
    i18n: {
        t: (key: string) => key,
    },
}));

describe('Chat_Content', () => {
    const mock_props = {
        is_detached: false,
        active_scope: 'agent' as const,
        target_node: 'Alpha Node',
        target_agent: 'Alpha Node',
        target_cluster: '',
        selected_agent_id: 'alpha-id',

        is_speaking: false,
        voice_status: 'idle' as const,

        show_transcript: false,
        set_show_transcript: vi.fn(),

        messages: [
            { id: '1', sender_id: '0', sender_name: 'Overlord', text: 'Hello Alpha', scope: 'agent' as const, timestamp: Date.now() },
            { id: '2', sender_id: 'alpha-id', sender_name: 'Alpha Node', text: 'Responding...', scope: 'agent' as const, timestamp: Date.now() },
        ],
        max_rendered_messages: 50,

        input_text: '',
        set_input_text: vi.fn(),
        on_send: vi.fn().mockResolvedValue(undefined),

        on_toggle_voice: vi.fn(),
        on_toggle_speech: vi.fn(),
        is_speech_enabled: false,

        on_toggle_safety: vi.fn(),
        is_safe_mode: true,

        on_toggle_detach: vi.fn(),
        on_clear_history: vi.fn(),
        on_minimize: vi.fn(),

        on_set_scope: vi.fn(),
        open_dropdown: null as any,
        set_open_dropdown: vi.fn(),
        sorted_agents: [
            { id: 'alpha-id', name: 'Alpha Node' }
        ] as any,
        set_target_agent: vi.fn(),
        set_selected_agent_id: vi.fn(),
        set_target_cluster: vi.fn(),
        clusters: [],
    };

    afterEach(() => {
        vi.restoreAllMocks();
    });

    it('renders header, message feed, and input bar', () => {
        render(<Chat_Content {...mock_props} />);

        // Header Title
        expect(screen.getAllByText('Alpha Node')[0]).toBeInTheDocument();
        
        // Scope Selector Buttons
        expect(screen.getAllByText('chat.scope_agent')[0]).toBeInTheDocument();
        expect(screen.getAllByText('chat.scope_cluster')[0]).toBeInTheDocument();
        expect(screen.getAllByText('chat.scope_swarm')[0]).toBeInTheDocument();

        // Chat Input Bar
        expect(screen.getByPlaceholderText('chat.input_placeholder')).toBeInTheDocument();

        // Message texts rendered
        expect(screen.getByText('Hello Alpha')).toBeInTheDocument();
        expect(screen.getByText('Responding...')).toBeInTheDocument();
    });
});
