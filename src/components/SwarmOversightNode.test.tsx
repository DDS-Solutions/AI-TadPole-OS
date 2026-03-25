import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { SwarmOversightNode } from './SwarmOversightNode';
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

vi.mock('../stores/workspaceStore', () => ({
    useWorkspaceStore: vi.fn(() => ({
        activeProposals: mockProposals,
        clusters: [{ id: 'cl-1', name: 'Test Cluster' }],
        dismissProposal: mockDismissProposal
    }))
}));

vi.mock('./ui', () => ({
    Tooltip: ({ children, content }: any) => <div title={content}>{children}</div>
}));

describe('SwarmOversightNode', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('renders proposals for the specific clusterId', () => {
        render(<SwarmOversightNode clusterId="cl-1" />);
        expect(screen.getByText(/Test Cluster/i)).toBeInTheDocument();
        expect(screen.getByText('Test reasoning')).toBeInTheDocument();
    });

    it('renders nothing if no proposal for clusterId', () => {
        const { container } = render(<SwarmOversightNode clusterId="cl-none" />);
        expect(container.firstChild).toBeNull();
    });

    it('calls onClose when X button is clicked', () => {
        const onClose = vi.fn();
        render(<SwarmOversightNode clusterId="cl-1" onClose={onClose} />);
        
        const closeBtn = screen.getByLabelText(i18n.t('common.dismiss'));
        fireEvent.click(closeBtn);
        expect(onClose).toHaveBeenCalled();
    });

    it('calls dismissProposal and onClose when Dismiss button is clicked', () => {
        const onClose = vi.fn();
        render(<SwarmOversightNode clusterId="cl-1" onClose={onClose} />);
        
        const dismissBtn = screen.getByLabelText(i18n.t('oversight.btn_dismiss'));
        fireEvent.click(dismissBtn);
        expect(mockDismissProposal).toHaveBeenCalledWith('cl-1');
        expect(onClose).toHaveBeenCalled();
    });
});
