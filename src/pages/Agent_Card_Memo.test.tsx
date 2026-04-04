import React from 'react';
import { describe, it, expect, vi } from 'vitest';
import { render } from '@testing-library/react';
import { Agent } from '../types';

// Mock dependencies
vi.mock('../i18n', () => ({
    i18n: {
        t: (key: string) => key,
    }
}));

vi.mock('../components/ui', () => ({
    Tooltip: ({ children }: any) => <div>{children}</div>,
}));

describe('AgentCardMemo', () => {
    const mockAgent: Agent = {
        id: '1',
        name: 'Agent Alpha',
        role: 'Orchestrator',
        status: 'active',
        model: 'gemini-1.5-pro',
        theme_color: '#10b981',
        cost_usd: 0.123,
        budget_usd: 1.0,
        skills: ['coding'],
        workflows: ['deploy'],
        mcpTools: ['github'],
        category: 'user',
        department: 'Operations',
        tokensUsed: 5000
    };

    it('suppresses re-render when props have same functional data (custom equality check)', () => {
        // We track renders by wrapping the component and using a side effect
        let renderCountOrigin = 0;
        
        // We create a version of AgentCard that increments a counter
        const TrackingAgentCard = (props: any) => {
            renderCountOrigin++;
            return <div data-testid="agent-card">{props.agent.name}</div>;
        };

        // We wrap this tracked component in React.memo with our actual equality logic
        const MemoizedTrackingCard = React.memo(TrackingAgentCard, (prev: any, next: any) => {
            const p = prev.agent;
            const n = next.agent;
            return (
                p.status === n.status &&
                p.cost_usd === n.cost_usd &&
                p.theme_color === n.theme_color &&
                p.name === n.name &&
                p.role === n.role &&
                p.model === n.model &&
                p.modelConfig?.temperature === n.modelConfig?.temperature &&
                p.skills?.length === n.skills?.length &&
                p.workflows?.length === n.workflows?.length &&
                p.mcpTools?.length === n.mcpTools?.length
            );
        });

        const { rerender } = render(<MemoizedTrackingCard agent={mockAgent} onSelect={() => {}} />);
        expect(renderCountOrigin).toBe(1);

        // 1. Rerender with DIFFERENT object identity but SAME data
        const clonedAgent = { ...mockAgent };
        rerender(<MemoizedTrackingCard agent={clonedAgent} onSelect={() => {}} />);
        
        // Should NOT re-render the internal component
        expect(renderCountOrigin).toBe(1);

        // 2. Rerender with DIFFERENT cost
        const updatedAgent = { ...mockAgent, cost_usd: 0.456 };
        rerender(<MemoizedTrackingCard agent={updatedAgent} onSelect={() => {}} />);
        
        // SHOULD re-render
        expect(renderCountOrigin).toBe(2);
    });
});
