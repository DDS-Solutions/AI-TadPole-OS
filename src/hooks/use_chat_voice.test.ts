/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **useChatVoice Unit Tests**: Validates the chat voice hook's integration with the audio service,
 * status listener subscriptions, auto-speak logic, and toggle callbacks.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Memory leak on rapid state changes, or incorrect trigger condition for speaking.
 * - **Telemetry Link**: Search `[useChatVoice.test]` in browser traces.
 */

import { renderHook, act } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { useChatVoice } from './use_chat_voice';
import { voice_client } from '../services/voice_client';

// Mock the voice client
vi.mock('../services/voice_client', () => {
    let current_status: any = 'idle';
    const status_listeners = new Set<(status: any) => void>();

    return {
        voice_client: {
            on_status_change: vi.fn((cb: any) => {
                status_listeners.add(cb);
                cb(current_status);
                return () => status_listeners.delete(cb);
            }),
            speak: vi.fn().mockResolvedValue(undefined),
            start_listening: vi.fn().mockImplementation(() => {
                current_status = 'initializing';
                status_listeners.forEach(cb => cb(current_status));
            }),
            stop_listening: vi.fn().mockImplementation(() => {
                current_status = 'idle';
                status_listeners.forEach(cb => cb(current_status));
            }),
            trigger_status_update: (status: any) => {
                current_status = status;
                status_listeners.forEach(cb => cb(current_status));
            }
        }
    };
});

describe('useChatVoice', () => {
    const mock_agents = [
        { id: 'alpha-agent', name: 'Alpha Agent', voice_id: 'voice-1', voice_engine: 'browser' }
    ] as any;

    beforeEach(() => {
        vi.clearAllMocks();
        // Reset state inside mock voice_client
        (voice_client as any).trigger_status_update('idle');
    });

    it('initializes state correctly', () => {
        const { result } = renderHook(() =>
            useChatVoice([], 'alpha-agent', mock_agents)
        );

        expect(result.current.voice_status).toBe('idle');
        expect(result.current.is_speech_enabled).toBe(false);
        expect(result.current.is_speaking).toBe(false);
    });

    it('subscribes to status updates', () => {
        const { result } = renderHook(() =>
            useChatVoice([], 'alpha-agent', mock_agents)
        );

        expect(result.current.voice_status).toBe('idle');

        act(() => {
            (voice_client as any).trigger_status_update('active');
        });

        expect(result.current.voice_status).toBe('active');
    });

    it('toggles speech output enabled state', () => {
        const { result } = renderHook(() =>
            useChatVoice([], 'alpha-agent', mock_agents)
        );

        expect(result.current.is_speech_enabled).toBe(false);

        act(() => {
            result.current.toggle_speech();
        });

        expect(result.current.is_speech_enabled).toBe(true);
    });

    it('handles voice listening toggles', () => {
        const { result } = renderHook(() =>
            useChatVoice([], 'alpha-agent', mock_agents)
        );

        // Start listening
        act(() => {
            result.current.toggle_voice();
        });

        expect(voice_client.start_listening).toHaveBeenCalled();
        expect(result.current.voice_status).toBe('initializing');

        // Stop listening
        act(() => {
            result.current.toggle_voice();
        });

        expect(voice_client.stop_listening).toHaveBeenCalled();
        expect(result.current.voice_status).toBe('idle');
    });

    it('triggers speak on new agent message when speech is enabled', async () => {
        // Render hook with speech initially disabled
        const { result, rerender } = renderHook(
            ({ messages }) => useChatVoice(messages, 'alpha-agent', mock_agents),
            { initialProps: { messages: [] as any[] } }
        );

        // Enable speech
        act(() => {
            result.current.toggle_speech();
        });

        const new_messages = [
            { id: 'msg-1', sender_id: 'alpha-agent', text: 'Hello, I am Alpha' }
        ];

        // Receive new message
        act(() => {
            rerender({ messages: new_messages });
        });

        expect(voice_client.speak).toHaveBeenCalledWith('Hello, I am Alpha', 'voice-1', 'browser');
    });

    it('updates is_speaking state using timers when speaking starts and ends', async () => {
        vi.useFakeTimers();

        // Render hook with speech initially disabled
        const { result, rerender } = renderHook(
            ({ messages }) => useChatVoice(messages, 'alpha-agent', mock_agents),
            { initialProps: { messages: [] as any[] } }
        );

        // Enable speech
        act(() => {
            result.current.toggle_speech();
        });

        const new_messages = [
            { id: 'msg-1', sender_id: 'alpha-agent', text: 'Hello, I am Alpha' }
        ];

        // Receive new message
        act(() => {
            rerender({ messages: new_messages });
        });

        // Flush the microtasks to let the speak promise .finally run
        await act(async () => {
            await Promise.resolve();
        });

        // Fast-forward to trigger the speaking start timeout
        act(() => {
            vi.advanceTimersByTime(10);
        });

        expect(result.current.is_speaking).toBe(true);

        // Fast-forward to trigger the speaking end timeout (delay is length 17 * 60 = 1020ms)
        act(() => {
            vi.advanceTimersByTime(1100);
        });

        expect(result.current.is_speaking).toBe(false);

        vi.useRealTimers();
    });

    it('does not speak error messages or security alerts', async () => {
        const { result, rerender } = renderHook(
            ({ messages }) => useChatVoice(messages, 'alpha-agent', mock_agents),
            { initialProps: { messages: [] as any[] } }
        );

        act(() => {
            result.current.toggle_speech();
        });

        const error_messages = [
            { id: 'msg-1', sender_id: 'alpha-agent', text: '❌ System Offline' },
            { id: 'msg-2', sender_id: 'alpha-agent', text: '🛡️ Intruder Alert' },
            { id: 'msg-3', sender_id: 'alpha-agent', text: 'Critical Error: timeout' }
        ];

        for (const msg of error_messages) {
            act(() => {
                rerender({ messages: [msg] });
            });
            expect(voice_client.speak).not.toHaveBeenCalled();
        }
    });

    it('clears active speech timeouts on new message or unmount', async () => {
        vi.useFakeTimers();

        const { result, rerender, unmount } = renderHook(
            ({ messages }) => useChatVoice(messages, 'alpha-agent', mock_agents),
            { initialProps: { messages: [] as any[] } }
        );

        act(() => {
            result.current.toggle_speech();
        });

        act(() => {
            rerender({ messages: [{ id: 'msg-1', sender_id: 'alpha-agent', text: 'First message' }] });
        });

        // Immediately trigger second message before timers advance
        act(() => {
            rerender({ messages: [
                { id: 'msg-1', sender_id: 'alpha-agent', text: 'First message' },
                { id: 'msg-2', sender_id: 'alpha-agent', text: 'Second message' }
            ] });
        });

        // Unmount
        unmount();
        
        vi.useRealTimers();
    });
});

// Metadata: [use_chat_voice_test]
