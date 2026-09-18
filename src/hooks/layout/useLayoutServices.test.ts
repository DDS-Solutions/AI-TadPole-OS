/**
 * @docs ARCHITECTURE:Testing
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend React Hooks / useLayoutServices.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` React hook lifecycle adheres to Rules of Hooks without conditional execution branches.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useLayoutServices } from './useLayoutServices';
import { get_tadpole_os_socket } from '../../services/socket';
import { provider_service } from '../../services/provider_service';
import { agent_service } from '../../services/agent_service';
import { event_bus } from '../../services/event_bus';
import { use_notification_store } from '../../stores/notification_store';
import { use_skill_store } from '../../stores/skill_store';

vi.mock('../../services/socket', () => {
    const mock_socket = {
        connect: vi.fn(),
        subscribe_pulse: vi.fn(() => vi.fn())
    };
    return {
        get_tadpole_os_socket: () => mock_socket
    };
});

vi.mock('../../services/provider_service', () => ({
    provider_service: {
        init: vi.fn(() => vi.fn()),
        sync_with_backend: vi.fn().mockResolvedValue(undefined)
    }
}));

vi.mock('../../services/agent_service', () => ({
    agent_service: {
        init: vi.fn(() => vi.fn()),
        load_agents_into_store: vi.fn().mockResolvedValue(undefined)
    }
}));

describe('useLayoutServices', () => {
    beforeEach(() => {
        vi.restoreAllMocks();
        use_notification_store.setState({ notifications: [] });
    });

    it('initializes socket pulses, provider services, and agent synchronizers', () => {
        renderHook(() => useLayoutServices());

        expect(get_tadpole_os_socket().connect).toHaveBeenCalled();
        expect(provider_service.init).toHaveBeenCalled();
        expect(provider_service.sync_with_backend).toHaveBeenCalled();
        expect(agent_service.init).toHaveBeenCalled();
        expect(agent_service.load_agents_into_store).toHaveBeenCalled();
    });

    it('routes socket pulse events to skill store handle_pulse', () => {
        let pulse_callback: any;
        vi.mocked(get_tadpole_os_socket().subscribe_pulse).mockImplementation((cb) => {
            pulse_callback = cb;
            return vi.fn();
        });

        const pulse_spy = vi.spyOn(use_skill_store.getState(), 'handle_pulse');

        renderHook(() => useLayoutServices());

        act(() => {
            pulse_callback({ tool: 'web_search', status: 'success', latency: 45 });
        });

        expect(pulse_spy).toHaveBeenCalledWith('web_search', 'success', 45);
    });

    it('pipes high-priority security and error events from event_bus to notification hub', () => {
        let log_listener: any;
        vi.spyOn(event_bus, 'subscribe_logs').mockImplementation((listener) => {
            log_listener = listener;
            return vi.fn();
        });

        renderHook(() => useLayoutServices());

        act(() => {
            log_listener({
                source: 'Agent',
                agent_name: 'Security Guard',
                text: 'Security check: Budget threshold exceeded',
                severity: 'warning'
            });
        });

        const notifications = use_notification_store.getState().notifications;
        expect(notifications.length).toBeGreaterThanOrEqual(1);
        expect(notifications[0].title).toBe('Agent Alert: Security Guard');
        expect(notifications[0].persistent).toBe(true);
    });
});
