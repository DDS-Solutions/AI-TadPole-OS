/**
 * @docs ARCHITECTURE:State
 * ### AI Assist Note
 * Transcript fixtures cover pruning, stable identity, replay and selective removal.
 * ### 🔍 Debugging & Observability
 * No browser or network is required for transcript transitions.
 */
import { describe, expect, it } from 'vitest';
import { empty_transcript, reduce_transcript } from './transcript';
import type { Chat_Message } from './types';

const message = (id: string): Chat_Message => ({ id, sender_id: 'a', sender_name: 'Agent', text: 'hello', scope: 'agent', timestamp: 1 });
describe('transcript reducer', () => {
    it('keeps indexes consistent after bounded pruning and selective removal', () => {
        const messages = Array.from({ length: 5000 }, (_, index) => message(String(index)));
        const state = { messages, message_index_by_id: Object.fromEntries(messages.map((m, i) => [m.id, i])) };
        const pruned = reduce_transcript(state, { type: 'add', message: message('new') });
        expect(pruned.messages).toHaveLength(4801);
        expect(pruned.message_index_by_id['0']).toBeUndefined();
        const removed = reduce_transcript(pruned, { type: 'remove', id: '201' });
        removed.messages.forEach((m, i) => expect(removed.message_index_by_id[m.id]).toBe(i));
        expect(removed.message_index_by_id['201']).toBeUndefined();
    });
    it('does not duplicate a replayed message or reset an answered question', () => {
        const m = { ...message('q'), parts: [{ type: 'question' as const, question_id: 'request', question: 'Proceed?', options: ['Yes', 'No'], status: 'answered' as const, selected_option: 'Yes' }] };
        const state = reduce_transcript(empty_transcript(), { type: 'add', message: m });
        expect(reduce_transcript(state, { type: 'add', message: m })).toBe(state);
        expect(reduce_transcript(state, { type: 'append', id: 'q', part: { ...m.parts[0], status: 'pending' } })).toBe(state);
    });
    it('keeps message IDs stable during updates and supports reserved object keys', () => {
        let state = reduce_transcript(empty_transcript(), { type: 'add', message: message('__proto__') });
        state = reduce_transcript(state, { type: 'update', id: '__proto__', updates: { id: 'changed', text: 'updated' } });
        expect(state.messages[0]).toMatchObject({ id: '__proto__', text: 'updated' });
        expect(state.message_index_by_id.__proto__).toBe(0);
    });
});
