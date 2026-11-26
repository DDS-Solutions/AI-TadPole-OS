/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Chat / Chat_Input_Bar.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import '@testing-library/jest-dom';
import { render, screen, fireEvent, act } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { Chat_Input_Bar } from './Chat_Input_Bar';

// Mock i18n
vi.mock('../../i18n', () => ({
    i18n: {
        t: (key: string) => key,
    },
}));

describe('Chat_Input_Bar', () => {
    const mock_props = {
        active_scope: 'agent',
        is_safe_mode: true,
        is_speech_enabled: false,
        is_speaking: false,
        input_value: '',
        on_change: vi.fn(),
        on_send: vi.fn().mockResolvedValue(undefined),
        on_toggle_voice: vi.fn(),
        on_toggle_speech: vi.fn(),
        on_toggle_safety: vi.fn(),
        is_listening: false,
    };

    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('renders input field and control buttons', () => {
        render(<Chat_Input_Bar {...mock_props} />);
        
        expect(screen.getByPlaceholderText('chat.input_placeholder')).toBeInTheDocument();
        expect(screen.getByRole('group', { name: 'Execution mode' })).toBeInTheDocument();
        const buttons = screen.getAllByRole('button');
        expect(buttons).toHaveLength(7);
    });

    it('triggers on_change when user types', () => {
        render(<Chat_Input_Bar {...mock_props} />);
        
        const input = screen.getByPlaceholderText('chat.input_placeholder');
        fireEvent.change(input, { target: { value: 'hello agent' } });
        
        expect(mock_props.on_change).toHaveBeenCalledWith('hello agent');
    });

    it('triggers on_send when user clicks send or presses Enter', async () => {
        const props = { ...mock_props, input_value: 'send message' };
        render(<Chat_Input_Bar {...props} />);
        
        const buttons = screen.getAllByRole('button');
        const send_btn = buttons[buttons.length - 1]; // The last button is Send
        
        await act(async () => {
            fireEvent.click(send_btn);
        });
        
        expect(mock_props.on_send).toHaveBeenCalled();
        
        // Also verify Enter keypress on input field
        vi.clearAllMocks();
        const input = screen.getByPlaceholderText('chat.input_placeholder');
        await act(async () => {
            fireEvent.keyDown(input, { key: 'Enter', code: 'Enter', charCode: 13 });
        });
        expect(mock_props.on_send).toHaveBeenCalled();
    });

    it('triggers on_toggle_voice when clicking mic', () => {
        render(<Chat_Input_Bar {...mock_props} />);
        
        const voice_btn = screen.getByRole('button', { name: 'chat.start_listening_tooltip' });
        fireEvent.click(voice_btn);
        
        expect(mock_props.on_toggle_voice).toHaveBeenCalled();
    });

    it('triggers on_toggle_speech when clicking speaker', () => {
        render(<Chat_Input_Bar {...mock_props} />);
        
        const speech_btn = screen.getByRole('button', { name: 'chat.enable_output_tooltip' });
        fireEvent.click(speech_btn);
        
        expect(mock_props.on_toggle_speech).toHaveBeenCalled();
    });

    it('triggers on_toggle_safety when clicking brain toggle', () => {
        render(<Chat_Input_Bar {...mock_props} />);
        
        const buttons = screen.getAllByRole('button');
        const safety_btn = buttons[buttons.length - 2]; // Safety Mode toggle sits just before Send
        fireEvent.click(safety_btn);
        
        expect(mock_props.on_toggle_safety).toHaveBeenCalled();
    });

    it('renders 3-mode Execution_Mode_Selector and Swarm handoff when wired', () => {
        const on_change_execution_mode = vi.fn();
        const on_launch_swarm = vi.fn();

        render(
            <Chat_Input_Bar
                {...mock_props}
                execution_mode="chat"
                on_change_execution_mode={on_change_execution_mode}
                can_launch_swarm={true}
                on_launch_swarm={on_launch_swarm}
            />
        );

        fireEvent.click(screen.getByRole('button', { name: 'Plan mode' }));
        expect(on_change_execution_mode).toHaveBeenCalledWith('plan');

        fireEvent.click(screen.getByRole('button', { name: 'Launch Swarm Mission from brainstorm' }));
        expect(on_launch_swarm).toHaveBeenCalledTimes(1);
    });
});
