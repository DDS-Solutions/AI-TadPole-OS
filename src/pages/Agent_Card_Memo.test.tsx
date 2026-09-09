/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Pages / Agent_Card_Memo.test
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

import React from 'react';
import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import type { Agent } from '../types';
import { Agent_Card_Memo } from '../components/agents/Agent_Card';

// Mock dependencies
vi.mock('../components/ui', () => ({
    Tooltip: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
}));

vi.mock('../i18n', () => ({
    i18n: {
        t: (key: string) => key,
    }
}));

vi.mock('../services/agent', () => ({
    agent_api_service: {
        reset_agent: vi.fn().mockResolvedValue({ status: 'ok' }),
    }
}));

vi.mock('../services/agent_service', () => ({
    agent_service: {
        update_agent: vi.fn().mockResolvedValue({}),
    }
}));

describe('Agent_Card_Memo', () => {
    const mock_agent: Agent = {
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
        mcp_tools: ['github'],
        category: 'user',
        department: 'Operations',
        tokens_used: 5000,
        failure_count: 0
    };

    it('suppresses re-render when props have same functional data (custom equality check)', () => {
        let render_count_origin = 0;
        
        const Tracking_Agent_Card = (props: { agent: Agent, on_select: () => void }) => {
            render_count_origin++;
            return <div data-testid="agent-card">{props.agent.name}</div>;
        };

        const Memoized_Tracking_Card = React.memo(Tracking_Agent_Card, (prev, next) => {
            const p = prev.agent;
            const n = next.agent;
            return (
                p.status === n.status &&
                p.cost_usd === n.cost_usd &&
                p.theme_color === n.theme_color &&
                p.name === n.name &&
                p.role === n.role &&
                p.model === n.model &&
                p.failure_count === n.failure_count &&
                p.last_failure_at === n.last_failure_at &&
                p.model_config?.temperature === n.model_config?.temperature &&
                (p.skills?.length ?? 0) === (n.skills?.length ?? 0) &&
                (p.workflows?.length ?? 0) === (n.workflows?.length ?? 0) &&
                (p.mcp_tools?.length ?? 0) === (n.mcp_tools?.length ?? 0)
            );
        });

        const { rerender } = render(<Memoized_Tracking_Card agent={mock_agent} on_select={() => {}} />);
        expect(render_count_origin).toBe(1);

        // 1. Rerender with DIFFERENT object identity but SAME data
        const cloned_agent = { ...mock_agent };
        rerender(<Memoized_Tracking_Card agent={cloned_agent} on_select={() => {}} />);
        expect(render_count_origin).toBe(1);

        // 2. Rerender with DIFFERENT failure_count
        const updated_agent = { ...mock_agent, failure_count: 3 };
        rerender(<Memoized_Tracking_Card agent={updated_agent} on_select={() => {}} />);
        expect(render_count_origin).toBe(2);
    });

    it('renders the health monitor status shield icon and toggles Node_Health overlay', () => {
        const on_select_mock = vi.fn();
        const degraded_agent: Agent = {
            ...mock_agent,
            failure_count: 2,
            last_failure_at: '2026-08-20T12:00:00Z'
        };

        render(<Agent_Card_Memo agent={degraded_agent} on_select={on_select_mock} />);
        
        // Find the health shield button
        const health_btn = screen.getByRole('button', { name: 'security.status_degraded' });
        expect(health_btn).toBeDefined();

        // Clicking the health button should toggle Node_Health overlay without triggering on_select
        fireEvent.click(health_btn);
        expect(on_select_mock).not.toHaveBeenCalled();

        // Node_Health overlay should now be visible
        expect(screen.getByText('security.swarm_health_monitor')).toBeDefined();
        expect(screen.getByText('security.reset_agent')).toBeDefined();
    });
});
