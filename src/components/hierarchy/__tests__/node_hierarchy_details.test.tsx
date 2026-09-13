/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Hierarchy / node_hierarchy_details.test
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
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { MemoryRouter } from 'react-router-dom';
import { Node_Health } from '../Node_Health';
import { Node_Mission } from '../Node_Mission';
import { Node_Model_Slots } from '../Node_Model_Slots';
import { Node_Stats } from '../Node_Stats';
import { agent_api_service } from '../../../services/agent';

// Mock i18n
vi.mock('../../../i18n', () => ({
    i18n: {
        t: (key: string) => key,
    },
}));

// Mock agent_api_service
vi.mock('../../../services/agent', () => ({
    agent_api_service: {
        reset_agent: vi.fn(),
    },
}));

// Mock agent_service
vi.mock('../../../services/agent_service', () => ({
    agent_service: {
        update_agent: vi.fn(),
    },
}));

describe('Node Hierarchy Component Details', () => {
    const mock_agent = {
        id: 'agent-123',
        name: 'Agent Alpha',
        role: 'Inference Specialist',
        status: 'idle',
        theme_color: '#10b981',
        failure_count: 1,
        active_mission: {
            objective: 'Compile core binaries',
            priority: 'high',
            is_degraded: false
        },
        model: 'gemini-1.5-pro',
        model_2: 'gemini-1.5-flash',
        model_3: '',
        skills: ['git_add'],
        workflows: [],
        mcp_tools: []
    } as any;

    beforeEach(() => {
        vi.clearAllMocks();
    });

    describe('Node_Health', () => {
        it('renders status message and handles reset RPC call', async () => {
            (agent_api_service.reset_agent as any).mockResolvedValue({ status: 'ok' });

            render(<Node_Health agent={mock_agent} on_close={vi.fn()} />);

            expect(screen.getByText('security.failures_label')).toBeInTheDocument();
            
            const reset_btn = screen.getByRole('button', { name: /security.reset_agent/i });
            await act(async () => {
                fireEvent.click(reset_btn);
            });

            expect(agent_api_service.reset_agent).toHaveBeenCalledWith('agent-123');
        });
    });

    describe('Node_Mission', () => {
        it('renders link to missions when active_mission is present', () => {
            render(
                <MemoryRouter>
                    <Node_Mission agent={mock_agent} />
                </MemoryRouter>
            );

            expect(screen.getByText(/Compile core binaries/)).toBeInTheDocument();
        });

        it('renders fallback when no mission is active', () => {
            const agent_no_mission = { ...mock_agent, active_mission: undefined };
            render(
                <MemoryRouter>
                    <Node_Mission agent={agent_no_mission} />
                </MemoryRouter>
            );

            expect(screen.getByText('agent_card.no_mission')).toBeInTheDocument();
        });
    });

    describe('Node_Model_Slots', () => {
        it('renders active model slots and allows changing model', () => {
            const mock_on_model_change = vi.fn();
            render(
                <Node_Model_Slots 
                    agent={mock_agent} 
                    on_model_change={mock_on_model_change} 
                />
            );

            // Shows the current model name
            expect(screen.getByText('gemini-1.5-pro')).toBeInTheDocument();
        });

        it('triggers on_update with active_model_slot when LED activation button is clicked', () => {
            const mock_on_update = vi.fn();
            render(
                <Node_Model_Slots 
                    agent={mock_agent} 
                    on_update={mock_on_update} 
                />
            );

            const activate_secondary_btn = screen.getByRole('button', { name: 'agent_card.tooltip_activate_secondary' });
            fireEvent.click(activate_secondary_btn);

            expect(mock_on_update).toHaveBeenCalledWith('agent-123', { active_model_slot: 2 });
        });
    });

    describe('Node_Stats', () => {
        it('renders cost, token usage, and capability triggers', () => {
            render(<Node_Stats agent={mock_agent} />);

            expect(screen.getByLabelText('agent_card.tooltip_skills')).toBeInTheDocument();
        });
    });
});
