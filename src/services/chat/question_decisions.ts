/**
 * @docs ARCHITECTURE:State
 * ### AI Assist Note
 * Pending question commands own optimistic answers and selective failure recovery.
 * ### 🔍 Debugging & Observability
 * Submission failures retain unrelated transcript edits and report one operator alert.
 */
import type { Sovereign_State } from '../../domain/chat/types';

interface Decision_Ports {
    get: () => Sovereign_State;
    remove_message: (id: string) => void;
    submit: (id: string, decision: 'approved' | 'rejected', answer: string) => Promise<unknown>;
    report_failure: (agent_id?: string) => void;
}

export function create_question_decisions(ports: Decision_Ports): (id: string, answer: string) => Promise<void> {
    const pending = new Set<string>();
    return async (id, answer) => {
        if (pending.has(id)) return;
        const message = ports.get().messages.find(m => m.parts?.some(p => p.type === 'question' && p.question_id === id));
        const question = message?.parts?.find(p => p.type === 'question' && p.question_id === id);
        if (!message || question?.type !== 'question' || question.status === 'answered') return;
        pending.add(id);
        const operator_id = crypto.randomUUID();
        ports.get().update_message(message.id, { parts: message.parts?.map(p => p === question
            ? { ...p, selected_option: answer, status: 'answered' } : p) });
        ports.get().add_message({
            id: operator_id, sender_id: '0', sender_name: 'Operator', agent_id: message.agent_id,
            text: answer, scope: ports.get().active_scope,
            parts: [{ type: 'text', content: answer, status: 'complete' }]
        });
        try {
            const approve = question.approve_option ?? question.options?.[0];
            await ports.submit(id, answer === approve ? 'approved' : 'rejected', answer);
        } catch {
            const current = ports.get().get_message_by_id(message.id);
            if (current) ports.get().update_message(message.id, { parts: current.parts?.map(p =>
                p.type === 'question' && p.question_id === id && p.selected_option === answer
                    ? { ...p, status: 'pending', selected_option: undefined } : p) });
            ports.remove_message(operator_id);
            ports.report_failure(message.agent_id);
        } finally {
            pending.delete(id);
        }
    };
}
