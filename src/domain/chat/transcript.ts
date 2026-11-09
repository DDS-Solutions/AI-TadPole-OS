/**
 * @docs ARCHITECTURE:State
 * ### AI Assist Note
 * Transcript transitions own message identity, indexes, and bounded history.
 * ### 🔍 Debugging & Observability
 * Pure transitions are exercised without browser or transport dependencies.
 */
import type { Message_Part } from '../../types';
import type { Chat_Message } from './types';

export interface Transcript_State {
    messages: Chat_Message[];
    message_index_by_id: Record<string, number>;
}

export type Transcript_Action =
    | { type: 'add'; message: Chat_Message }
    | { type: 'update'; id: string; updates: Partial<Chat_Message> }
    | { type: 'append'; id: string; part: Message_Part }
    | { type: 'remove'; id: string }
    | { type: 'clear' };

export const empty_transcript = (): Transcript_State => ({ messages: [], message_index_by_id: {} });

function indexed(messages: Chat_Message[]): Transcript_State {
    return { messages, message_index_by_id: Object.fromEntries(messages.map((m, i) => [m.id, i])) };
}

export function message_index(state: Transcript_State, id: string): number | undefined {
    return Object.hasOwn(state.message_index_by_id, id) ? state.message_index_by_id[id] : undefined;
}

function same_part(existing: Message_Part, incoming: Message_Part): boolean {
    if (existing.type !== incoming.type) return false;
    if (existing.type === 'question' && incoming.type === 'question') {
        return existing.question_id === incoming.question_id;
    }
    return 'content' in existing && 'content' in incoming
        ? existing.content === incoming.content
        : JSON.stringify(existing) === JSON.stringify(incoming);
}

export function reduce_transcript(state: Transcript_State, action: Transcript_Action): Transcript_State {
    if (action.type === 'clear') return empty_transcript();
    const id = action.type === 'add' ? action.message.id : action.id;
    const index = message_index(state, id);
    if (action.type === 'add') {
        if (index !== undefined) return state;
        if (state.messages.length >= 5000) return indexed([...state.messages.slice(200), action.message]);
        return {
            messages: [...state.messages, action.message],
            message_index_by_id: { ...state.message_index_by_id, [id]: state.messages.length }
        };
    }
    if (index === undefined) return state;
    if (action.type === 'remove') return indexed(state.messages.filter(m => m.id !== id));
    const current = state.messages[index];
    if (action.type === 'append' && current.parts?.some(p => same_part(p, action.part))) return state;
    const updated = action.type === 'update'
        ? { ...current, ...action.updates, id }
        : { ...current, parts: [...(current.parts || []), action.part] };
    const messages = [...state.messages];
    messages[index] = updated;
    return { messages, message_index_by_id: state.message_index_by_id };
}
