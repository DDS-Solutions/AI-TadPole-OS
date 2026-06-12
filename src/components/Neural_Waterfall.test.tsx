/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Verification of the Neural Waterfall's lifecycle**, including timeline zoom/scroll updates, dynamic localized row ticking, trace span selection, and detail panel flyout/detach.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Memory leaks in row tickers, zoom calculation failures, or portal render exceptions.
 * - **Telemetry Link**: Search `[Neural_Waterfall.test]` in tracing logs.
 */

import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { Neural_Waterfall } from './Neural_Waterfall';
import { use_trace_store } from '../stores/trace_store';
import { use_agent_store } from '../stores/agent_store';
import { use_tab_store } from '../stores/tab_store';
import type { Trace_Node } from '../types';

// Mock Zustand Stores
vi.mock('../stores/trace_store', () => ({
    use_trace_store: vi.fn(),
}));

vi.mock('../stores/agent_store', () => ({
    use_agent_store: vi.fn(),
}));

vi.mock('../stores/tab_store', () => ({
    use_tab_store: vi.fn(),
}));

vi.mock('../i18n', () => ({
    i18n: {
        t: (key: string) => key,
    }
}));

// Mock Portal_Window to render children directly instead of calling window.open
vi.mock('./ui/Portal_Window', () => ({
    Portal_Window: ({ children }: any) => <div data-testid="portal-window">{children}</div>,
}));

describe('Neural_Waterfall Component', () => {
    const mock_span: Trace_Node & { depth: number } = {
        id: 'span-1',
        trace_id: 'trace-123',
        name: 'Task Alpha',
        agent_id: 'agent-1',
        mission_id: 'mission-1',
        status: 'success',
        start_time: Date.now() - 1000,
        end_time: Date.now(),
        attributes: {
            'tool.name': 'read_file',
            'file.path': '/workspace/test.txt'
        },
        children: [],
        depth: 0
    };

    const mock_agent = {
        id: 'agent-1',
        name: 'Agent Alpha',
        role: 'Researcher'
    };

    beforeEach(() => {
        vi.clearAllMocks();

        // Setup store return values
        (use_trace_store as any).mockReturnValue({
            active_trace_id: 'trace-123',
            get_trace_tree: () => [mock_span]
        });

        (use_agent_store as any).mockReturnValue({
            get_agent: () => mock_agent
        });

        (use_tab_store as any).mockReturnValue({
            is_trace_stream_detached: false,
            toggle_trace_stream_detachment: vi.fn()
        });
    });

    it('renders the Gantt timeline and trace nodes', () => {
        render(<Neural_Waterfall />);
        
        expect(screen.getByText('trace_stream.title')).toBeInTheDocument();
        expect(screen.getByText('Agent Alpha')).toBeInTheDocument();
        expect(screen.getByText('Task Alpha')).toBeInTheDocument();
    });

    it('renders the Zoom slider and updates multiplier value on slide', () => {
        render(<Neural_Waterfall />);
        
        const zoomSlider = screen.getByLabelText('Timeline Zoom');
        expect(zoomSlider).toBeInTheDocument();
        expect(zoomSlider).toHaveValue('1');

        fireEvent.change(zoomSlider, { target: { value: '2' } });
        expect(zoomSlider).toHaveValue('2');
        expect(screen.getByText('200%')).toBeInTheDocument();
    });

    it('opens detail flyout when clicking on a span row', async () => {
        render(<Neural_Waterfall />);
        
        const spanRow = screen.getByText('Task Alpha');
        fireEvent.click(spanRow);

        // Verify Flyout header is visible
        expect(screen.getAllByText('Task Alpha')).toHaveLength(2);
        
        // Verify span attributes are shown
        expect(screen.getByText('tool.name')).toBeInTheDocument();
        expect(screen.getByText('read_file')).toBeInTheDocument();
        expect(screen.getByText('file.path')).toBeInTheDocument();
        expect(screen.getByText('/workspace/test.txt')).toBeInTheDocument();
    });

    it('moves detail panel to a Portal_Window when clicked Detach, renders overlay placeholder, and allows recall', () => {
        render(<Neural_Waterfall />);
        
        // Select span
        const spanRow = screen.getByText('Task Alpha');
        fireEvent.click(spanRow);

        // Detach button
        const detachBtn = screen.getByTitle('Detach Details Window');
        fireEvent.click(detachBtn);

        // Verify Portal_Window is rendered
        expect(screen.getByTestId('portal-window')).toBeInTheDocument();

        // Verify inline overlay placeholder is rendered
        const overlayPlaceholder = screen.getByTestId('detached-overlay-placeholder');
        expect(overlayPlaceholder).toBeInTheDocument();
        expect(screen.getByText(/DETAILS_DETACHED/)).toBeInTheDocument();

        // Click "RECALL SECTOR" button
        const recallBtn = screen.getByText(/recall_sector|RECALL SECTOR/i);
        fireEvent.click(recallBtn);

        // Verify Portal_Window is closed and inline details are back
        expect(screen.queryByTestId('portal-window')).not.toBeInTheDocument();
        expect(screen.queryByTestId('detached-overlay-placeholder')).not.toBeInTheDocument();
        expect(screen.getAllByText('Task Alpha')).toHaveLength(2); // One in row list, one in open detail panel
    });

    it('redacts sensitive values (SEC-004) and avoids double HTML-escaping in attributes display (SEC-005)', () => {
        const test_span: Trace_Node & { depth: number } = {
            id: 'span-2',
            trace_id: 'trace-123',
            name: 'Task Beta',
            agent_id: 'agent-1',
            mission_id: 'mission-1',
            status: 'success',
            start_time: Date.now() - 500,
            end_time: Date.now(),
            attributes: {
                'auth_header': 'Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c',
                'html_val': '<script>alert("xss")</script>'
            },
            children: [],
            depth: 0
        };

        (use_trace_store as any).mockReturnValue({
            active_trace_id: 'trace-123',
            get_trace_tree: () => [test_span]
        });

        render(<Neural_Waterfall />);
        
        // Open details
        const spanRow = screen.getByText('Task Beta');
        fireEvent.click(spanRow);

        // Verify Bearer Token / JWT is redacted
        expect(screen.queryByText(/eyJhbGciOi/)).not.toBeInTheDocument();
        expect(screen.getAllByText('[REDACTED]')).toHaveLength(1);

        // Verify HTML tags are rendered as literal text without double-escaping
        expect(screen.getByText('<script>alert("xss")</script>')).toBeInTheDocument();
    });
});
