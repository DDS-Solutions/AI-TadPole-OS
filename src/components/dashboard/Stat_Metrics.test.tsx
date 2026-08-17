/**
 * @docs ARCHITECTURE:Testing
 *
 * ### AI Assist Note
 * Regression coverage for the adjacent production module and its public contracts.
 *
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Contract, rendering, state transition, or error-handling regression.
 * - **Trace Scope**: Vitest assertions and test-local mocks.
 */

import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Stat_Metrics } from './Stat_Metrics';

describe('Stat_Metrics Component', () => {
    it('renders swarm statistics and token formats correctly', () => {
        render(
            <Stat_Metrics
                active_agents={3}
                online_count={8}
                total_cost={0.045}
                total_tokens={15400}
                total_input_tokens={10000}
                total_output_tokens={5400}
                total_budget={5.00}
                budget_util={0.9}
                recruit_velocity={2}
            />
        );

        expect(screen.getByText('3/8')).toBeDefined();
        expect(screen.getByText('15.4k')).toBeDefined();
        expect(screen.getByText('0.9%')).toBeDefined();
    });

    it('formats megatokens correctly when token count exceeds 1M', () => {
        render(
            <Stat_Metrics
                active_agents={5}
                online_count={10}
                total_cost={12.50}
                total_tokens={2500000}
                budget_util={25.0}
                recruit_velocity={1}
            />
        );

        expect(screen.getByText('2.50M')).toBeDefined();
    });
});
