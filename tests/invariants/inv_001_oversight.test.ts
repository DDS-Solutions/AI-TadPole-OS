/**
 * @docs ARCHITECTURE:Quality:Verification
 * ### AI Assist Note
 * INV-001 exercises the actual oversight command through the sovereign store facade.
 * ### 🔍 Debugging & Observability
 * Network failure fixtures verify selective rollback and exact question matching.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { use_sovereign_store } from '../../src/stores/sovereign_store';
const { submit } = vi.hoisted(() => ({ submit: vi.fn() }));
vi.mock('../../src/services/system_api_service', () => ({ system_api_service: { oversight: { decide_oversight: submit } } }));

function question(id: string, approve_option?: string) {
    use_sovereign_store.getState().add_message({ id, sender_id: 'agent', sender_name: 'Agent', scope: 'agent', text: 'Same question',
        parts: [{ type: 'question', question: 'Same question', question_id: id, options: ['Yes', 'No'], approve_option, status: 'pending' }] }, true);
}

describe('INV-001: Oversight decisions', () => {
    beforeEach(() => {
        submit.mockReset().mockResolvedValue(undefined);
        use_sovereign_store.setState({ messages: [], message_index_by_id: {}, active_scope: 'agent' });
    });
    it.each([
        ['Yes', undefined, 'approved'], ['No', undefined, 'rejected'],
        ['No', 'No', 'approved'], ['Yes ', 'Yes', 'rejected'], ['yes', 'Yes', 'rejected']
    ] as const)('maps exact answer %s with explicit approval %s to %s', async (answer, approval, expected) => {
        question('q', approval);
        await use_sovereign_store.getState().resolve_user_question('q', answer);
        expect(submit).toHaveBeenCalledWith('q', expected, answer);
    });
    it('matches the question ID even when question text is identical', async () => {
        question('first'); question('second');
        await use_sovereign_store.getState().resolve_user_question('second', 'Yes');
        expect(submit).toHaveBeenCalledWith('second', 'approved', 'Yes');
        expect(use_sovereign_store.getState().get_message_by_id('first')?.parts?.[0].status).toBe('pending');
    });
    it('reverts only its answer and operator message when submission fails', async () => {
        let fail!: (error: Error) => void;
        submit.mockImplementationOnce(() => new Promise((_, reject) => { fail = reject; }));
        question('q');
        const pending = use_sovereign_store.getState().resolve_user_question('q', 'Yes');
        await vi.waitFor(() => expect(fail).toBeTypeOf('function'));
        use_sovereign_store.getState().add_message({ id: 'other', sender_id: 'other', sender_name: 'Other', text: 'Unrelated', scope: 'agent' }, true);
        fail(new Error('offline')); await pending;
        expect(use_sovereign_store.getState().get_message_by_id('q')?.parts?.[0]).toMatchObject({ status: 'pending', selected_option: undefined });
        expect(use_sovereign_store.getState().get_message_by_id('other')).toBeDefined();
        expect(use_sovereign_store.getState().messages.filter(m => m.sender_id === '0')).toEqual([]);
    });
});
