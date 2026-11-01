/**
 * @docs ARCHITECTURE:State
 * ### AI Assist Note
 * This adapter owns chat channel and diagnostic subscriptions, including teardown.
 * ### 🔍 Debugging & Observability
 * Remote and replay deliveries never republish through the chat channel.
 */
import type { Chat_Message, Sovereign_State, Mission_Execution_Metrics } from '../../domain/chat/types';
import type { log_entry, Log_Origin } from '../event_bus';
import { ingest_chat_log } from './log_projection';

export interface Chat_Envelope { type: string; payload?: unknown }
interface Replication_Ports {
    get: () => Sovereign_State;
    remove_message: (id: string) => void;
    subscribe_logs: (listener: (entry: log_entry, origin?: Log_Origin) => void) => () => void;
}

export function create_chat_replication(ports: Replication_Ports) {
    let channel: BroadcastChannel | null = null;
    let references = 0;
    let cleanup_logs: (() => void) | null = null;
    const get_channel = () => {
        if (!channel && typeof window !== 'undefined' && typeof BroadcastChannel !== 'undefined') {
            channel = new BroadcastChannel('tadpole-chat-sync');
        }
        return channel;
    };
    const receive = (event: MessageEvent<Chat_Envelope>) => {
        if (!event.data || typeof event.data !== 'object') return;
        const { type, payload } = event.data;
        const store = ports.get();
        switch (type) {
            case 'ADD_MESSAGE':
                if (payload && typeof payload === 'object' && 'sender_id' in payload && 'text' in payload) {
                    store.add_message(payload as Chat_Message, true);
                }
                break;
            case 'UPDATE_MESSAGE':
                if (payload && typeof payload === 'object' && 'id' in payload && typeof payload.id === 'string') {
                    store.update_message(payload.id, payload as Partial<Chat_Message>, true);
                }
                break;
            case 'REMOVE_MESSAGE': if (typeof payload === 'string') ports.remove_message(payload); break;
            case 'SET_SCOPE':
                if (payload === 'agent' || payload === 'cluster' || payload === 'swarm') store.set_scope(payload, true);
                break;
            case 'SET_AGENT': if (typeof payload === 'string' || payload === null) store.set_selected_agent_id(payload, true); break;
            case 'SET_TARGET_AGENT': if (typeof payload === 'string') store.set_target_agent(payload, true); break;
            case 'SET_TARGET_CLUSTER': if (typeof payload === 'string') store.set_target_cluster(payload, true); break;
            case 'SET_DETACHED': if (typeof payload === 'boolean') store.set_detached(payload, true); break;
            case 'SET_EXECUTION_METRICS':
                if (payload && typeof payload === 'object') store.set_execution_metrics(payload as Mission_Execution_Metrics, true);
                break;
            case 'CLEAR_HISTORY': store.clear_history(true); break;
        }
    };
    return {
        publish(envelope: Chat_Envelope) { get_channel()?.postMessage(envelope); },
        acquire(): () => void {
            if (references++ === 0) {
                get_channel()?.addEventListener('message', receive);
                cleanup_logs = ports.subscribe_logs((entry, origin) => ingest_chat_log(ports.get(), entry, origin));
            }
            let released = false;
            return () => {
                if (released) return;
                released = true;
                if (--references !== 0) return;
                cleanup_logs?.();
                cleanup_logs = null;
                channel?.removeEventListener('message', receive);
                channel?.close();
                channel = null;
            };
        }
    };
}
