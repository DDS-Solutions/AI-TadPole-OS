/**
 * @docs ARCHITECTURE:State
 * ### AI Assist Note
 * Cover channel lifetime and remote delivery through replication ports.
 * ### 🔍 Debugging & Observability
 * A deterministic channel fixture records listeners and outgoing envelopes.
 */
import { afterEach, describe, expect, it, vi } from 'vitest';
import { create_chat_replication } from './replication';
import type { Sovereign_State } from '../../domain/chat/types';

describe('chat replication lifetime', () => {
    afterEach(() => vi.unstubAllGlobals());

    it('shares one listener, releases it once, and reconnects after disposal', () => {
        const channels: FakeChannel[] = [];
        class FakeChannel {
            addEventListener = vi.fn();
            removeEventListener = vi.fn();
            close = vi.fn();
            postMessage = vi.fn();
            constructor() { channels.push(this); }
        }
        vi.stubGlobal('BroadcastChannel', FakeChannel);
        const store = { add_message: vi.fn(), update_message: vi.fn() } as unknown as Sovereign_State;
        const remove_message = vi.fn();
        const cleanup = vi.fn();
        const subscribe_logs = vi.fn(() => cleanup);
        const replication = create_chat_replication({ get: () => store, remove_message, subscribe_logs });
        const first = replication.acquire();
        const second = replication.acquire();
        expect(channels).toHaveLength(1);
        expect(subscribe_logs).toHaveBeenCalledTimes(1);
        const receive = channels[0].addEventListener.mock.calls[0][1];
        receive({ data: { type: 'ADD_MESSAGE', payload: { sender_id: 'agent', text: 'remote' } } });
        expect(store.add_message).toHaveBeenCalledWith({ sender_id: 'agent', text: 'remote' }, true);
        receive({ data: { type: 'REMOVE_MESSAGE', payload: 'failed-operator-message' } });
        expect(remove_message).toHaveBeenCalledWith('failed-operator-message');
        expect(channels[0].postMessage).not.toHaveBeenCalled();
        first(); first();
        expect(cleanup).not.toHaveBeenCalled();
        second();
        expect(cleanup).toHaveBeenCalledTimes(1);
        expect(channels[0].close).toHaveBeenCalledTimes(1);
        const third = replication.acquire();
        expect(channels).toHaveLength(2);
        expect(subscribe_logs).toHaveBeenCalledTimes(2);
        third();
    });
});
