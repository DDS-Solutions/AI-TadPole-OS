/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Chat / Question_Choice_Pill.test
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
import { Question_Choice_Pill } from './Question_Choice_Pill';
import { use_sovereign_store } from '../../stores/sovereign_store';

describe('Question_Choice_Pill', () => {
    beforeEach(() => {
        use_sovereign_store.setState({
            messages: [],
            message_index_by_id: {},
            active_scope: 'agent',
            selected_agent_id: null,
            target_agent: 'Agent of Nine',
            target_cluster: '',
            is_detached: false,
            latest_metrics: null,
        });
    });

    it('renders question, options, and recommended badge', () => {
        render(
            <Question_Choice_Pill
                question="Select deployment target:"
                options={['(Recommended) Staging Cluster', 'Production Cluster']}
                context="Staging allows pre-flight verification."
                question_id="q-101"
            />
        );

        expect(screen.getByText('Select deployment target:')).toBeInTheDocument();
        expect(screen.getByText('Staging Cluster')).toBeInTheDocument();
        expect(screen.getByText('Production Cluster')).toBeInTheDocument();
        expect(screen.getByText(/rec/i)).toBeInTheDocument();
        expect(screen.getByText('Staging allows pre-flight verification.')).toBeInTheDocument();
    });

    it('invokes resolve_user_question when an option is clicked', async () => {
        const resolve_spy = vi.fn().mockResolvedValue(undefined);
        use_sovereign_store.setState({
            resolve_user_question: resolve_spy,
        });

        render(
            <Question_Choice_Pill
                question="Select deployment target:"
                options={['Staging Cluster', 'Production Cluster']}
                question_id="q-101"
            />
        );

        const btn = screen.getByText('Staging Cluster');
        await act(async () => {
            fireEvent.click(btn);
        });

        expect(resolve_spy).toHaveBeenCalledWith('q-101', 'Staging Cluster');
    });

    it('renders answered state when status is answered', () => {
        render(
            <Question_Choice_Pill
                question="Select deployment target:"
                options={['Staging Cluster', 'Production Cluster']}
                question_id="q-101"
                selected_option="Staging Cluster"
                status="answered"
            />
        );

        expect(screen.getByText('Resolved')).toBeInTheDocument();
        expect(screen.getByText('Staging Cluster')).toBeInTheDocument();
        expect(screen.queryByText('+ Custom answer')).not.toBeInTheDocument();
    });
});
