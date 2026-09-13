/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Voice / Live_Voice_Hub.test
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
import { render, screen, fireEvent, act } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { Live_Voice_Hub } from './Live_Voice_Hub';

// Mock MockWebSocket class
class MockWebSocket {
    static CONNECTING = 0;
    static OPEN = 1;
    static CLOSING = 2;
    static CLOSED = 3;

    static instances: MockWebSocket[] = [];
    
    url: string;
    readyState: number = 0;
    onopen: (() => void) | null = null;
    onclose: (() => void) | null = null;
    onmessage: ((event: any) => void) | null = null;
    onerror: ((event: any) => void) | null = null;
    
    sent_messages: string[] = [];
    closed = false;

    constructor(url: string) {
        this.url = url;
        MockWebSocket.instances.push(this);
        setTimeout(() => {
            this.readyState = 1; // OPEN
            if (this.onopen) this.onopen();
        }, 10);
    }

    send(data: string) {
        this.sent_messages.push(data);
    }

    close() {
        this.closed = true;
        this.readyState = 3; // CLOSED
        if (this.onclose) this.onclose();
    }
}

// Mock AudioContext
class MockAudioContext {
    state = 'suspended';
    currentTime = 0;
    destination = {};
    
    createMediaStreamSource = vi.fn().mockReturnValue({
        connect: vi.fn(),
    });
    
    createScriptProcessor = vi.fn().mockReturnValue({
        connect: vi.fn(),
        disconnect: vi.fn(),
        onaudioprocess: null,
    });
    
    resume = vi.fn().mockResolvedValue(undefined);
    close = vi.fn().mockResolvedValue(undefined);
    createBufferSource = vi.fn().mockReturnValue({
        connect: vi.fn(),
        start: vi.fn(),
    });
    decodeAudioData = vi.fn().mockImplementation((buf, cb) => {
        cb({ duration: 0.5 });
        return Promise.resolve({ duration: 0.5 });
    });
}

// Mock i18n
vi.mock('../../i18n', () => ({
    i18n: {
        t: (key: string) => key,
    },
}));

describe('Live_Voice_Hub', () => {
    const mock_on_close = vi.fn();
    let mock_tracks: any[];

    beforeEach(() => {
        vi.stubGlobal('WebSocket', MockWebSocket);
        vi.stubGlobal('AudioContext', MockAudioContext);
        if (typeof window !== 'undefined') {
            window.AudioContext = MockAudioContext as any;
        }
        MockWebSocket.instances = [];
        
        mock_tracks = [{ stop: vi.fn() }];
        
        vi.stubGlobal('navigator', {
            mediaDevices: {
                getUserMedia: vi.fn().mockResolvedValue({
                    getTracks: () => mock_tracks
                })
            }
        });
        
        vi.clearAllMocks();
    });

    afterEach(() => {
        vi.unstubAllGlobals();
    });

    it('renders the layout header and visualizer', async () => {
        await act(async () => {
            render(<Live_Voice_Hub agent_id="test-agent" theme_color="#10b981" on_close={mock_on_close} />);
        });
        
        expect(screen.getByText('voice.service_gemini')).toBeInTheDocument();
        
        // Background overlay close button
        const close_btn = screen.getByLabelText('voice.close');
        await act(async () => {
            fireEvent.click(close_btn);
        });
        expect(mock_on_close).toHaveBeenCalled();
    });

    it('establishes WebSocket on mount and sends setup frame', async () => {
        await act(async () => {
            render(<Live_Voice_Hub agent_id="test-agent" theme_color="#10b981" on_close={mock_on_close} />);
        });
        
        await act(async () => {
            await new Promise(resolve => setTimeout(resolve, 20));
        });
        
        expect(MockWebSocket.instances).toHaveLength(1);
        const ws = MockWebSocket.instances[0];
        
        expect(ws.sent_messages).toHaveLength(1);
        const setup_frame = JSON.parse(ws.sent_messages[0]);
        expect(setup_frame.setup.agent_id).toBe('test-agent');
    });

    it('toggles recording and releases media streams on stop', async () => {
        await act(async () => {
            render(<Live_Voice_Hub agent_id="test-agent" theme_color="#10b981" on_close={mock_on_close} />);
        });
        
        await act(async () => {
            await new Promise(resolve => setTimeout(resolve, 20));
        });
        
        // Find mic toggle button (initial state is "voice.start")
        const toggle_btn = screen.getByLabelText('voice.start');
        
        // Start microphone
        await act(async () => {
            fireEvent.click(toggle_btn);
        });
        
        expect(navigator.mediaDevices.getUserMedia).toHaveBeenCalled();
        
        // Stop microphone
        await act(async () => {
            fireEvent.click(toggle_btn);
        });
        
        expect(mock_tracks[0].stop).toHaveBeenCalled();
    });

    it('handles microphone access rejection gracefully', async () => {
        vi.stubGlobal('navigator', {
            mediaDevices: {
                getUserMedia: vi.fn().mockRejectedValue(new Error('Permission denied'))
            }
        });

        await act(async () => {
            render(<Live_Voice_Hub agent_id="test-agent" theme_color="#10b981" on_close={mock_on_close} />);
        });

        const toggle_btn = screen.getByLabelText('voice.start');
        
        await act(async () => {
            fireEvent.click(toggle_btn);
        });

        expect(screen.getByLabelText('voice.start')).toBeInTheDocument();
    });

    it('processes and transmits PCM 16-bit chunk on audio process event', async () => {
        let audioprocess_callback: any = null;
        
        const mock_processor = {
            connect: vi.fn(),
            disconnect: vi.fn(),
            set onaudioprocess(cb: any) {
                audioprocess_callback = cb;
            },
            get onaudioprocess() {
                return audioprocess_callback;
            }
        };

        class CustomMockAudioContext extends MockAudioContext {
            createScriptProcessor = vi.fn().mockReturnValue(mock_processor);
        }
        vi.stubGlobal('AudioContext', CustomMockAudioContext);
        if (typeof window !== 'undefined') {
            window.AudioContext = CustomMockAudioContext as any;
        }

        await act(async () => {
            render(<Live_Voice_Hub agent_id="test-agent" theme_color="#10b981" on_close={mock_on_close} />);
        });

        await act(async () => {
            await new Promise(resolve => setTimeout(resolve, 20));
        });

        const ws = MockWebSocket.instances[0];

        const toggle_btn = screen.getByLabelText('voice.start');
        
        // Start mic
        await act(async () => {
            fireEvent.click(toggle_btn);
        });

        expect(audioprocess_callback).not.toBeNull();

        const input_buffer = {
            getChannelData: vi.fn().mockReturnValue(new Float32Array([0.5, -0.5, 0.0]))
        };
        const mock_event = {
            inputBuffer: input_buffer
        };

        // Trigger audio processing callback
        act(() => {
            audioprocess_callback(mock_event);
        });

        expect(ws.sent_messages.length).toBeGreaterThan(1);
        const last_sent = JSON.parse(ws.sent_messages[ws.sent_messages.length - 1]);
        expect(last_sent.realtime_input).toBeDefined();
        expect(last_sent.realtime_input.media_chunks[0].mime_type).toBe('audio/pcm;rate=16000');
    });
});
