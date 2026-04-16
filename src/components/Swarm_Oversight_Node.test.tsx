/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Tests the Swarm Oversight Node's rendering of active proposals and cluster metadata.** 
 * Verifies `dismiss_proposal` signaling and modal closure callbacks during cluster reorganization. 
 * Mocks `workspace_store` to isolate specific proposal data mapping and dismissal logic.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Mismatched `cluster_id` causing empty state rendering or failure to clear the `active_proposals` in the `workspace_store`.
 * - **Telemetry Link**: Search `[Swarm_Oversight_Node.test]` in tracing logs.
 */


import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Swarm_Oversight_Node } from './Swarm_Oversight_Node';
import { i18n } from '../i18n';

const mock_proposals = {
    'cl-1': {
        cluster_id: 'cl-1',
        reasoning: 'Test reasoning',
        changes: [],
        timestamp: Date.now()
    }
};

const mock_dismiss_proposal = vi.fn();

vi.mock('../i18n', () => ({
    i18n: {
        t: (key: string) => key,
    },
}));

vi.mock('../stores/workspace_store', () => ({
    use_workspace_store: vi.fn(() => ({
        active_proposals: mock_proposals,
        clusters: [{ id: 'cl-1', name: 'Test Cluster' }],
        dismiss_proposal: mock_dismiss_proposal
    }))
}));

vi.mock('./ui', () => ({
    Tooltip: ({ children, content }: any) => <div title={content}>{children}</div>
}));

describe('Swarm_Oversight_Node', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('renders proposals for the specific cluster_id', () => {
        render(<Swarm_Oversight_Node cluster_id="cl-1" />);
        expect(screen.getByText(/Test Cluster/i)).toBeInTheDocument();
        expect(screen.getByText('Test reasoning')).toBeInTheDocument();
    });

    it('renders nothing if no proposal for cluster_id', () => {
        const { container } = render(<Swarm_Oversight_Node cluster_id="cl-none" />);
        expect(container.firstChild).toBeNull();
    });

    it('calls on_close when X button is clicked', () => {
        const on_close = vi.fn();
        render(<Swarm_Oversight_Node cluster_id="cl-1" on_close={on_close} />);
        
        const close_btn = screen.getByLabelText(i18n.t('common.dismiss'));
        fireEvent.click(close_btn);
        expect(on_close).toHaveBeenCalled();
    });

    it('calls dismiss_proposal and on_close when Dismiss button is clicked', () => {
        const on_close = vi.fn();
        render(<Swarm_Oversight_Node cluster_id="cl-1" on_close={on_close} />);
        
        const dismiss_btn = screen.getByLabelText(i18n.t('oversight.btn_dismiss'));
        fireEvent.click(dismiss_btn);
        expect(mock_dismiss_proposal).toHaveBeenCalledWith('cl-1');
        expect(on_close).toHaveBeenCalled();
    });
});

