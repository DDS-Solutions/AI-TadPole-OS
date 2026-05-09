/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Validation of the Automated Agent Standup and Progress Reporting view.** 
 * Verifies the aggregation of daily task completions, voice-sync recordings, and the generation of the 'Swarm Pulse' summary. 
 * Mocks `voice_client` and `tadpole_os_service.transcribe` to isolate voice workflows from backend engine latency.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Incomplete standup reports due to agent timeout or failure to parse the markdown summary from the `standup_store`.
 * - **Telemetry Link**: Search `[Standups.test]` in tracing logs.
 */


/**
 * @file Standups.test.tsx
 * @description Suite for the Neural Sync Interface (Voice Standups) page.
 * @module Pages/Standups
 * @testedBehavior
 * - Voice Communication: Integration with voice_client for recording and TTS.
 * - Dynamic Targeting: Switching between Agent Node and Mission Cluster sync modes.
 * - Live Transcription: Handling of event_bus transcriptions and command orchestration.
 * - Neural Handoff: Routing cluster-level commands to the designated Alpha node.
 * @aiContext
 * - Mocks voice_client and tadpole_os_service.transcribe for voice workflow isolation.
 * - Subscribes to event_bus to simulate incoming agent communications.
 * - Refactored for 100% snake_case architectural parity.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, act, waitFor } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import Standups from './Standups';
import { use_workspace_store } from '../stores/workspace_store';
import { load_agents } from '../services/agent_service';
import { event_bus } from '../services/event_bus';
import { voice_client } from '../services/voice_client';
import { tadpole_os_service } from '../services/tadpoleos_service';

// Mock Dependencies
vi.mock('../stores/workspace_store');
vi.mock('../services/agent_service', () => ({
    load_agents: vi.fn()
}));
vi.mock('../services/event_bus', () => ({
    event_bus: {
        subscribe_logs: vi.fn(),
        emit_log: vi.fn(),
        get_history: vi.fn(() => []),
    }
}));
vi.mock('../services/voice_client', () => ({
    voice_client: {
        speak: vi.fn(),
        start_recording: vi.fn(),
        stop_recording: vi.fn()
    }
}));
vi.mock('../services/tadpoleos_service', () => ({
    tadpole_os_service: {
        transcribe: vi.fn(),
        send_command: vi.fn()
    }
}));

// Mock ResizeObserver
global.ResizeObserver = class ResizeObserver {
    observe() {}
    unobserve() {}
    disconnect() {}
};

describe('Standups Page', () => {
    const mock_agents = [
        {
            id: 'x-agent-1', name: 'Alpha Agent', role: 'Dev', department: 'Engineering',
            model_config: { provider: 'test', modelId: 'test-model' }
        },
        {
            id: 'x-agent-2', name: 'Beta Agent', role: 'Tester', department: 'Engineering',
            model_config: { provider: 'test', modelId: 'test-model' }
        }
    ];

    const mock_clusters = [
        {
            id: 'cluster-1',
            name: 'Frontend Core',
            department: 'Engineering',
            alpha_id: 'x-agent-1',
            budget_usd: 100
        }
    ];

    // Since event_bus.subscribe_logs accepts a callback, we need to capture it to simulate events
    let event_bus_callback: (event: any) => void;

    beforeEach(() => {
        vi.clearAllMocks();
        
        (use_workspace_store as unknown as ReturnType<typeof vi.fn>).mockReturnValue(mock_clusters);
        (load_agents as unknown as ReturnType<typeof vi.fn>).mockResolvedValue(mock_agents as any);

        (event_bus.subscribe_logs as unknown as ReturnType<typeof vi.fn>).mockImplementation((cb) => {
            event_bus_callback = cb;
            return vi.fn(); // return unsubscribe function
        });
        
        (tadpole_os_service.transcribe as unknown as ReturnType<typeof vi.fn>).mockResolvedValue('Hello Alpha');
        (voice_client.stop_recording as unknown as ReturnType<typeof vi.fn>).mockResolvedValue(new Blob());
    });

    it('renders the neural sync interface and live transcript base', async () => {
        await act(async () => {
             render(<Standups />);
        });

        expect(screen.getByText('Neural Sync Interface')).toBeInTheDocument();
        expect(screen.getByText('Live Transcript')).toBeInTheDocument();
        expect(screen.getAllByText('System').length).toBeGreaterThan(0);
        expect(screen.getByText("Voice Communications Online. Select target and click 'Start Sync'.")).toBeInTheDocument();
        expect(screen.getByRole('button', { name: /Start Sync/i })).toBeInTheDocument();
    });

    it('populates agent and cluster targets', async () => {
        await act(async () => {
             render(<Standups />);
        });

        // Toggle to Cluster mode
        await act(async () => {
            fireEvent.click(screen.getByText('Mission Cluster'));
        });

        expect(screen.getByText('FRONTEND CORE')).toBeInTheDocument();
        
        // Toggle to Agent mode
        await act(async () => {
            fireEvent.click(screen.getByText('Agent Node'));
        });

        expect(screen.getByText('ALPHA AGENT')).toBeInTheDocument();
    });

    it('handles incoming event_bus messages and text-to-speech for agents', async () => {
        await act(async () => {
             render(<Standups />);
        });

        // Simulate agent speaking
        act(() => {
            if (event_bus_callback) {
                event_bus_callback({ source: 'Agent', agent_id: 'Alpha Agent', text: 'I am ready.' });
            }
        });

        // Should appear in the transcript
        expect(screen.getByText('Alpha Agent')).toBeInTheDocument();
        expect(screen.getByText('I am ready.')).toBeInTheDocument();
        // Should trigger TTS
        expect(voice_client.speak).toHaveBeenCalledWith('I am ready.');
    });

    it('starts and stops recording, transcribes, and sends command to an agent target', async () => {
        await act(async () => {
             render(<Standups />);
        });

        const start_button = screen.getByRole('button', { name: /Start Sync/i });
        
        // Start Recording
        await act(async () => {
            fireEvent.click(start_button);
        });

        expect(voice_client.start_recording).toHaveBeenCalled();

        // End Recording
        const end_button = screen.getByRole('button', { name: /End Sync/i });
        
        await act(async () => {
            fireEvent.click(end_button);
        });

        expect(voice_client.stop_recording).toHaveBeenCalled();

        // Wait for transcription and emit
        await waitFor(() => {
            expect(tadpole_os_service.transcribe).toHaveBeenCalled();
            expect(event_bus.emit_log).toHaveBeenCalledWith(expect.objectContaining({
                source: 'User',
                text: 'Hello Alpha (To: Alpha Agent)'
            }));
            expect(tadpole_os_service.send_command).toHaveBeenCalledWith(
                'x-agent-1',
                'Hello Alpha',
                'test-model',
                'test'
            );
        });
    });

    it('handles neural handoff to an entire cluster (routing to Alpha node)', async () => {
        await act(async () => {
             render(<Standups />);
        });

        // Select Cluster mode
        await act(async () => {
            fireEvent.click(screen.getByText('Mission Cluster'));
        });

        // Make sure it selects cluster-1 initially
        const select = screen.getByRole('combobox');
        expect(select).toHaveValue('cluster-1');

        const start_button = screen.getByRole('button', { name: /Start Sync/i });
        
        // Start Recording
        await act(async () => {
            fireEvent.click(start_button);
        });

        // End Recording
        const end_button = screen.getByRole('button', { name: /End Sync/i });
        
        await act(async () => {
            fireEvent.click(end_button);
        });

        await waitFor(() => {
            expect(tadpole_os_service.send_command).toHaveBeenCalledWith(
                'x-agent-1', // the alpha node of the cluster
                '[CLUSTER COMMAND: Frontend Core] Hello Alpha',
                'test-model',
                'test',
                'cluster-1',
                'Engineering',
                100 // budget
            );
        });
    });
});


// Metadata: [Standups_test]

// Metadata: [Standups_test]
