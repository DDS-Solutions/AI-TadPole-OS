/**
 * @docs ARCHITECTURE:State
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Services / Chat / question_decisions.test
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { describe, it, expect, vi } from 'vitest';
import { create_question_decisions } from './question_decisions';
import type { Sovereign_State } from '../../domain/chat/types';

function makeHarness(submitImpl: (id: string, decision: 'approved' | 'rejected', answer: string) => Promise<unknown>) {
    const state: Partial<Sovereign_State> = {
        active_scope: 'swarm',
        messages: [
            {
                id: 'msg-1',
                sender_id: 'agent-1',
                sender_name: 'Architect',
                agent_id: 'agent-1',
                text: 'Proceed?',
                timestamp: Date.now(),
                scope: 'swarm',
                parts: [
                    {
                        type: 'question',
                        question_id: 'q-1',
                        question: 'Proceed?',
                        options: ['Yes', 'No'],
                        approve_option: 'Yes',
                        status: 'pending',
                    },
                ],
            },
        ],
        update_message: vi.fn((id, patch) => {
            state.messages = (state.messages ?? []).map((m) => (m.id === id ? { ...m, ...patch } : m));
        }),
        add_message: vi.fn((msg) => {
            state.messages = [...(state.messages ?? []), msg];
        }),
        get_message_by_id: vi.fn((id) => (state.messages ?? []).find((m) => m.id === id)),
    };

    const remove_message = vi.fn((id: string) => {
        state.messages = (state.messages ?? []).filter((m) => m.id !== id);
    });
    const report_failure = vi.fn();
    const submit = vi.fn(submitImpl);
    const decide = create_question_decisions({
        get: () => state as Sovereign_State,
        remove_message,
        submit,
        report_failure,
    });

    return { state, decide, submit, remove_message, report_failure };
}

describe('create_question_decisions', () => {
    it('submits approved when selected answer matches approve_option and marks question answered', async () => {
        const { state, decide, submit } = makeHarness(() => Promise.resolve());
        await decide('q-1', 'Yes');
        expect(submit).toHaveBeenCalledWith('q-1', 'approved', 'Yes');
        const part = state.messages?.[0].parts?.[0];
        expect(part?.type === 'question' && part.status).toBe('answered');
    });

    it('rolls back optimistic answer and operator message when submit fails', async () => {
        const { state, decide, remove_message, report_failure } = makeHarness(() => Promise.reject(new Error('network')));
        await decide('q-1', 'No');
        expect(remove_message).toHaveBeenCalledTimes(1);
        expect(report_failure).toHaveBeenCalledWith('agent-1');
        const part = state.messages?.[0].parts?.[0];
        expect(part?.type === 'question' && part.status).toBe('pending');
    });
});
