/**
 * @docs ARCHITECTURE:State
 * ### AI Assist Note
 * Project diagnostics into chat once, preserving event identity across replay.
 * ### 🔍 Debugging & Observability
 * Delivery origin is supplied by the event bus, never inferred from log content.
 */
import type { log_entry, Log_Origin } from '../event_bus';
import type { Chat_Message, Sovereign_State } from '../../domain/chat/types';

export function ingest_chat_log(store: Sovereign_State, log: log_entry, origin: Log_Origin = 'local'): void {
    const type = log.metadata?.type;
    if (!log.agent_id || !type) return;
    const name = log.agent_name || log.agent_id;
    const remote = origin !== 'local';
    let message: Chat_Message;
    const base = {
        id: log.id, sender_id: log.agent_id, sender_name: name, agent_id: log.agent_id,
        timestamp: log.timestamp, scope: store.active_scope
    };
    if (type === 'agent:message') {
        const text = String(log.metadata?.text || log.metadata?.content || log.metadata?.message || log.text || 'Mission action complete.');
        message = { ...base, text, target_node: name, parts: [{ type: 'text', content: text }] };
    } else if (type === 'thought' || (type === 'log' && log.metadata?.level !== 'debug')) {
        const text = String(log.metadata?.message || log.metadata?.text || '');
        message = { ...base, text, parts: [type === 'thought'
            ? { type: 'thought', content: text, status: 'done' }
            : { type: 'text', content: text }] };
    } else if (type === 'agent:user_question_requested') {
        const question = String(log.metadata?.question || log.text || '');
        const options = Array.isArray(log.metadata?.options)
            ? log.metadata.options.filter((option): option is string => typeof option === 'string') : [];
        message = { ...base, text: question, target_node: name, parts: [{
            type: 'question', question, options, context: String(log.metadata?.context || ''),
            question_id: String(log.metadata?.question_id || log.id), status: 'pending'
        }] };
    } else return;
    const existing = store.get_message_by_id(log.id);
    if (!existing) store.add_message(message, remote);
    else for (const part of message.parts || []) store.append_message_part(log.id, part, remote);
}
