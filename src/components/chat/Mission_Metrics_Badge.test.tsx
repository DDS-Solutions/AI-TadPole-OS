/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Chat / Mission_Metrics_Badge.test
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
import { render, screen } from '@testing-library/react';
import { describe, it, expect, beforeEach } from 'vitest';
import { Mission_Metrics_Badge } from './Mission_Metrics_Badge';
import { use_sovereign_store } from '../../stores/sovereign_store';

describe('Mission_Metrics_Badge', () => {
    beforeEach(() => {
        use_sovereign_store.setState({
            latest_metrics: null,
        });
    });

    it('returns null when there are no metrics or turns is 0', () => {
        const { container } = render(<Mission_Metrics_Badge />);
        expect(container.firstChild).toBeNull();
    });

    it('renders turns, tokens, and cost when metrics are present', () => {
        use_sovereign_store.setState({
            latest_metrics: {
                turns: 5,
                tool_calls_attempted: 10,
                tool_calls_failed: 0,
                cached_reads_hit: 4,
                files_modified_count: 2,
                total_input_tokens: 3000,
                total_output_tokens: 1500,
                peak_context_tokens: 6000,
                total_cost_micro_usd: 25000,
                total_summarizations: 0,
                total_sub_agents: 0,
                excessive_summarization_warning: false,
            },
        });

        render(<Mission_Metrics_Badge />);
        expect(screen.getByText('T:5')).toBeInTheDocument();
        expect(screen.getByText('4.5k')).toBeInTheDocument();
        expect(screen.getByText('$0.0250')).toBeInTheDocument();
        expect(screen.queryByText('Thrashing Risk')).not.toBeInTheDocument();
    });

    it('renders thrashing risk warning when excessive_summarization_warning is true', () => {
        use_sovereign_store.setState({
            latest_metrics: {
                turns: 8,
                tool_calls_attempted: 12,
                tool_calls_failed: 1,
                cached_reads_hit: 2,
                files_modified_count: 3,
                total_input_tokens: 25000,
                total_output_tokens: 5000,
                peak_context_tokens: 12000,
                total_cost_micro_usd: 75000,
                total_summarizations: 3,
                total_sub_agents: 2,
                excessive_summarization_warning: true,
            },
        });

        render(<Mission_Metrics_Badge />);
        expect(screen.getByText('Thrashing Risk')).toBeInTheDocument();
        expect(screen.getByText('3c')).toBeInTheDocument();
    });
});
