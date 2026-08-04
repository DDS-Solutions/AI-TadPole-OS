/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Chat_Input_Bar Unit Tests**: Validates that typing, text dispatching,
 * safety triggers, and speech toggle buttons react correctly to props and events.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Keyboard submission bypass, or missing action dispatch callbacks.
 * - **Telemetry Link**: Search `[Chat_Input_Bar.test]` in tracing logs.
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
        const buttons = screen.getAllByRole('button');
        expect(buttons).toHaveLength(4);
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
        const send_btn = buttons[3]; // The last button is Send
        
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
        
        const buttons = screen.getAllByRole('button');
        const voice_btn = buttons[0]; // The first button is Voice/Mic
        fireEvent.click(voice_btn);
        
        expect(mock_props.on_toggle_voice).toHaveBeenCalled();
    });

    it('triggers on_toggle_speech when clicking speaker', () => {
        render(<Chat_Input_Bar {...mock_props} />);
        
        const buttons = screen.getAllByRole('button');
        const speech_btn = buttons[1]; // The second button is Speech/Audio
        fireEvent.click(speech_btn);
        
        expect(mock_props.on_toggle_speech).toHaveBeenCalled();
    });

    it('triggers on_toggle_safety when clicking brain toggle', () => {
        render(<Chat_Input_Bar {...mock_props} />);
        
        const buttons = screen.getAllByRole('button');
        const safety_btn = buttons[2]; // The third button is Safety Mode
        fireEvent.click(safety_btn);
        
        expect(mock_props.on_toggle_safety).toHaveBeenCalled();
    });
});

// Metadata: [Chat_Input_Bar_test]
