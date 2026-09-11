/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend React Hooks / use_chat_dispatch.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` React hook lifecycle adheres to Rules of Hooks without conditional execution branches.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { renderHook, act } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { useChatDispatch } from './use_chat_dispatch';
import { use_settings_store } from '../stores/settings_store';
import { use_sovereign_store } from '../stores/sovereign_store';
import { process_command } from '../logic/command_processor';

// Mock command_processor
vi.mock('../logic/command_processor', () => ({
    process_command: vi.fn().mockResolvedValue(undefined),
}));

// Mock i18n
vi.mock('../i18n', () => ({
    i18n: {
        t: (key: string, options?: any) => {
            if (key === 'chat.fault_detected' && options) {
                return `fault:${options.message}`;
            }
            return key;
        },
    },
}));

describe('useChatDispatch', () => {
    const mock_add_message = vi.fn();
    const mock_agents = [
        { id: 'alpha-agent', name: 'Alpha Agent' }
    ] as any;

    beforeEach(() => {
        vi.clearAllMocks();
        // Setup initial store values
        use_settings_store.setState({
            settings: { is_safe_mode: true } as any
        });
        use_sovereign_store.setState({
            active_scope: 'agent',
            target_agent: 'Alpha Agent',
            target_cluster: '',
        });
    });

    it('sets input text correctly', () => {
        const { result } = renderHook(() =>
            useChatDispatch('agent', 'Alpha Agent', mock_agents, 'alpha-agent', mock_add_message)
        );

        act(() => {
            result.current.set_input_text('test command');
        });

        expect(result.current.input_text).toBe('test command');
    });

    it('toggles safety mode', () => {
        const { result } = renderHook(() =>
            useChatDispatch('agent', 'Alpha Agent', mock_agents, 'alpha-agent', mock_add_message)
        );

        act(() => {
            result.current.toggle_safety();
        });

        const store_state = use_settings_store.getState();
        expect(store_state.settings.is_safe_mode).toBe(false);
    });

    it('ignores empty sends', async () => {
        const { result } = renderHook(() =>
            useChatDispatch('agent', 'Alpha Agent', mock_agents, 'alpha-agent', mock_add_message)
        );

        await act(async () => {
            await result.current.handle_send();
        });

        expect(mock_add_message).not.toHaveBeenCalled();
        expect(process_command).not.toHaveBeenCalled();
    });

    it('sends prompt, clears input, and calls process_command', async () => {
        const { result } = renderHook(() =>
            useChatDispatch('agent', 'Alpha Agent', mock_agents, 'alpha-agent', mock_add_message)
        );

        act(() => {
            result.current.set_input_text('who are you');
        });

        await act(async () => {
            await result.current.handle_send();
        });

        expect(mock_add_message).toHaveBeenCalledWith({
            sender_id: '0',
            sender_name: 'chat.overlord_name',
            text: 'who are you',
            scope: 'agent',
            target_node: 'Alpha Agent'
        });

        expect(result.current.input_text).toBe('');
        expect(process_command).toHaveBeenCalledWith(
            'who are you',
            mock_agents,
            true, // is_safe_mode
            'agent',
            'Alpha Agent'
        );
    });

    it('handles swarm scope messages', async () => {
        use_sovereign_store.setState({
            active_scope: 'swarm',
            target_agent: '',
            target_cluster: '',
        });

        const { result } = renderHook(() =>
            useChatDispatch('swarm', '', mock_agents, null, mock_add_message)
        );

        act(() => {
            result.current.set_input_text('swarm broad');
        });

        await act(async () => {
            await result.current.handle_send();
        });

        expect(mock_add_message).toHaveBeenCalledWith({
            sender_id: '0',
            sender_name: 'chat.overlord_name',
            text: 'swarm broad',
            scope: 'swarm',
            target_node: undefined
        });
    });

    it('handles cluster scope messages', async () => {
        use_sovereign_store.setState({
            active_scope: 'cluster',
            target_agent: '',
            target_cluster: 'Omega-Cluster',
        });

        const { result } = renderHook(() =>
            useChatDispatch('cluster', 'Omega-Cluster', mock_agents, null, mock_add_message)
        );

        act(() => {
            result.current.set_input_text('cluster run');
        });

        await act(async () => {
            await result.current.handle_send();
        });

        expect(mock_add_message).toHaveBeenCalledWith({
            sender_id: '0',
            sender_name: 'chat.overlord_name',
            text: 'cluster run',
            scope: 'cluster',
            target_node: 'Omega-Cluster'
        });

        expect(process_command).toHaveBeenCalledWith(
            'cluster run',
            mock_agents,
            true,
            'cluster',
            'Omega-Cluster'
        );
    });

    it('appends system message on command processor error', async () => {
        vi.mocked(process_command).mockRejectedValueOnce(new Error('Backend offline'));

        const { result } = renderHook(() =>
            useChatDispatch('agent', 'Alpha Agent', mock_agents, 'alpha-agent', mock_add_message)
        );

        act(() => {
            result.current.set_input_text('fail command');
        });

        await act(async () => {
            await result.current.handle_send();
        });

        expect(mock_add_message).toHaveBeenCalledWith({
            sender_id: '0',
            sender_name: 'chat.overlord_name',
            text: 'fail command',
            scope: 'agent',
            target_node: 'Alpha Agent'
        });

        expect(mock_add_message).toHaveBeenCalledWith({
            sender_id: 'system',
            sender_name: 'chat.system_name',
            text: 'fault:Backend offline',
            scope: 'agent'
        });
    });

    it('handles selected agent not found in agents list', async () => {
        const { result } = renderHook(() =>
            useChatDispatch('agent', 'Alpha Agent', mock_agents, 'non-existent-agent', mock_add_message)
        );

        act(() => {
            result.current.set_input_text('test dispatch');
        });

        await act(async () => {
            await result.current.handle_send();
        });

        expect(process_command).toHaveBeenCalledWith(
            'test dispatch',
            mock_agents,
            true,
            'agent',
            'Alpha Agent'
        );
    });
});
