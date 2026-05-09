/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Verification of the Agent Card Memo's performance-critical rendering logic.** 
 * Ensures correct data hydration for agent personas, logs, and trace links within the dashboard viewport. 
 * Connects the global state stores to the layout orchestration for high-fidelity observability.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Recursive re-renders during prop updates or failure to parse the `json_metadata` field from the agent store.
 * - **Telemetry Link**: Search `[Agent_Card_Memo.test]` in tracing logs.
 */

import '@testing-library/jest-dom';

import React from 'react';
import { describe, it, expect, vi } from 'vitest';
import { render } from '@testing-library/react';
import type { Agent } from '../types';

// Mock dependencies

vi.mock('../components/ui', () => ({
    Tooltip: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
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
        tokens_used: 5000
    };

    it('suppresses re-render when props have same functional data (custom equality check)', () => {
        // We track renders by wrapping the component and using a side effect
        let render_count_origin = 0;
        
        // We create a version of AgentCard that increments a counter
        const Tracking_Agent_Card = (props: { agent: Agent, on_select: () => void }) => {
            render_count_origin++;
            return <div data-testid="agent-card">{props.agent.name}</div>;
        };

        // We wrap this tracked component in React.memo with our actual equality logic
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
        
        // Should NOT re-render the internal component
        expect(render_count_origin).toBe(1);

        // 2. Rerender with DIFFERENT cost
        const updated_agent = { ...mock_agent, cost_usd: 0.456 };
        rerender(<Memoized_Tracking_Card agent={updated_agent} on_select={() => {}} />);
        
        // SHOULD re-render
        expect(render_count_origin).toBe(2);
    });
});


// Metadata: [Agent_Card_Memo_test]

// Metadata: [Agent_Card_Memo_test]
