import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Swarm_Oversight_Node } from './Swarm_Oversight_Node';
import { i18n } from '../i18n';

const mockProposals = {
    'cl-1': {
        clusterId: 'cl-1',
        reasoning: 'Test reasoning',
        changes: [],
        timestamp: Date.now()
    }
};

const mockDismissProposal = vi.fn();

vi.mock('../stores/workspace_store', () => ({
    use_workspace_store: vi.fn(() => ({
        activeProposals: mockProposals,
        clusters: [{ id: 'cl-1', name: 'Test Cluster' }],
        dismissProposal: mockDismissProposal
    }))
}));

vi.mock('./ui', () => ({
    Tooltip: ({ children, content }: any) => <div title={content}>{children}</div>
}));

describe('Swarm_Oversight_Node', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('renders proposals for the specific clusterId', () => {
        render(<Swarm_Oversight_Node clusterId="cl-1" />);
        expect(screen.getByText(/Test Cluster/i)).toBeInTheDocument();
        expect(screen.getByText('Test reasoning')).toBeInTheDocument();
    });

    it('renders nothing if no proposal for clusterId', () => {
        const { container } = render(<Swarm_Oversight_Node clusterId="cl-none" />);
        expect(container.firstChild).toBeNull();
    });

    it('calls onClose when X button is clicked', () => {
        const onClose = vi.fn();
        render(<Swarm_Oversight_Node clusterId="cl-1" onClose={onClose} />);
        
        const closeBtn = screen.getByLabelText(i18n.t('common.dismiss'));
        fireEvent.click(closeBtn);
        expect(onClose).toHaveBeenCalled();
    });

    it('calls dismissProposal and onClose when Dismiss button is clicked', () => {
        const onClose = vi.fn();
        render(<Swarm_Oversight_Node clusterId="cl-1" onClose={onClose} />);
        
        const dismissBtn = screen.getByLabelText(i18n.t('oversight.btn_dismiss'));
        fireEvent.click(dismissBtn);
        expect(mockDismissProposal).toHaveBeenCalledWith('cl-1');
        expect(onClose).toHaveBeenCalled();
    });
});
