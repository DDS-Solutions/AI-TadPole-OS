/**
 * @file Standups.test.tsx
 * @description Suite for the Neural Sync Interface (Voice Standups) page.
 * @module Pages/Standups
 * @testedBehavior
 * - Voice Communication: Integration with voiceClient for recording and TTS.
 * - Dynamic Targeting: Switching between Agent Node and Mission Cluster sync modes.
 * - Live Transcription: Handling of event_bus transcriptions and command orchestration.
 * - Neural Handoff: Routing cluster-level commands to the designated Alpha node.
 * @aiContext
 * - Mocks voiceClient and tadpole_os_service.transcribe for voice workflow isolation.
 * - Subscribes to event_bus to simulate incoming agent communications.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, act, waitFor } from '@testing-library/react';
import Standups from './Standups';
import { use_workspace_store } from '../stores/workspace_store';
import { loadAgents } from '../services/agent_service';
import { event_bus } from '../services/event_bus';
import { voiceClient } from '../services/voice_client';
import { tadpole_os_service } from '../services/tadpoleos_service';

// Mock Dependencies
vi.mock('../stores/workspace_store');
vi.mock('../services/agent_service', () => ({
    loadAgents: vi.fn()
}));
vi.mock('../services/event_bus', () => ({
    event_bus: {
        subscribe: vi.fn(),
        emit: vi.fn(),
        getHistory: vi.fn(() => []),
    }
}));
vi.mock('../services/voice_client', () => ({
    voiceClient: {
        speak: vi.fn(),
        startRecording: vi.fn(),
        stopRecording: vi.fn()
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
    const mockAgents = [
        {
            id: 'x-agent-1', name: 'Alpha Agent', role: 'Dev', department: 'Engineering',
            modelConfig: { provider: 'test', modelId: 'test-model' }
        },
        {
            id: 'x-agent-2', name: 'Beta Agent', role: 'Tester', department: 'Engineering',
            modelConfig: { provider: 'test', modelId: 'test-model' }
        }
    ];

    const mockClusters = [
        {
            id: 'cluster-1',
            name: 'Frontend Core',
            department: 'Engineering',
            alphaId: 'x-agent-1',
            budget_usd: 100
        }
    ];

    // Since event_bus.subscribe_logs accepts a callback, we need to capture it to simulate events
    let eventBusCallback: (event: any) => void;

    beforeEach(() => {
        vi.clearAllMocks();
        
        (use_workspace_store as unknown as ReturnType<typeof vi.fn>).mockReturnValue(mockClusters);
        (loadAgents as unknown as ReturnType<typeof vi.fn>).mockResolvedValue(mockAgents as any);

        (event_bus.subscribe_logs as unknown as ReturnType<typeof vi.fn>).mockImplementation((cb) => {
            eventBusCallback = cb;
            return vi.fn(); // return unsubscribe function
        });
        
        (tadpole_os_service.transcribe as unknown as ReturnType<typeof vi.fn>).mockResolvedValue('Hello Alpha');
        (voiceClient.stopRecording as unknown as ReturnType<typeof vi.fn>).mockResolvedValue(new Blob());
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
            if (eventBusCallback) {
                eventBusCallback({ source: 'Agent', agent_id: 'Alpha Agent', text: 'I am ready.' });
            }
        });

        // Should appear in the transcript
        expect(screen.getByText('Alpha Agent')).toBeInTheDocument();
        expect(screen.getByText('I am ready.')).toBeInTheDocument();
        // Should trigger TTS
        expect(voiceClient.speak).toHaveBeenCalledWith('I am ready.');
    });

    it('starts and stops recording, transcribes, and sends command to an agent target', async () => {
        await act(async () => {
             render(<Standups />);
        });

        const startButton = screen.getByRole('button', { name: /Start Sync/i });
        
        // Start Recording
        await act(async () => {
            fireEvent.click(startButton);
        });

        expect(voiceClient.startRecording).toHaveBeenCalled();

        // End Recording
        const endButton = screen.getByRole('button', { name: /End Sync/i });
        
        await act(async () => {
            fireEvent.click(endButton);
        });

        expect(voiceClient.stopRecording).toHaveBeenCalled();

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

        const startButton = screen.getByRole('button', { name: /Start Sync/i });
        
        // Start Recording
        await act(async () => {
            fireEvent.click(startButton);
        });

        // End Recording
        const endButton = screen.getByRole('button', { name: /End Sync/i });
        
        await act(async () => {
            fireEvent.click(endButton);
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
