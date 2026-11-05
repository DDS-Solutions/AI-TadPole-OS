/**
 * @docs ARCHITECTURE:Testing
 * ### AI Assist Note
 * Covers pending, rejected, and confirmed agent configuration writes.
 * ### 🔍 Debugging & Observability
 * - Telemetry: [ConfigPanel], event_bus save failure/success messages.
 */
import { describe, it, expect, vi } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useAgentConfig } from './useAgentConfig';
import { event_bus } from '../../services/event_bus';
import type { Agent } from '../../contracts/agent';

const agent = { id: 'agent-save', name: 'Saved Agent', role: 'assistant', department: 'Operations', status: 'idle', model: 'gemini-1.5-flash', skills: [], workflows: [], tokens_used: 0, cost_usd: 0, budget_usd: 1, theme_color: '#10b981', category: 'user' } as Agent;

describe('agent configuration save outcome', () => {
    it('keeps the dialog open and saving active until persistence finishes', async () => {
        let complete!: (saved: boolean) => void;
        const update = vi.fn(() => new Promise<boolean>(resolve => { complete = resolve; }));
        const close = vi.fn();
        const { result } = renderHook(() => useAgentConfig(agent, update, close));
        let pending!: Promise<void>;
        act(() => { pending = result.current.handleSave(); });
        expect(result.current.state.ui.saving).toBe(true);
        expect(close).not.toHaveBeenCalled();
        await act(async () => { complete(true); await pending; });
        expect(close).toHaveBeenCalledOnce();
        expect(result.current.state.ui.saving).toBe(false);
    });

    it.each(['false', 'rejection'])('keeps the dialog open and reports failure on %s', async outcome => {
        const log = vi.spyOn(event_bus, 'emit_log');
        const update = outcome === 'false' ? vi.fn().mockResolvedValue(false) : vi.fn().mockRejectedValue(new Error('offline'));
        const close = vi.fn();
        const { result } = renderHook(() => useAgentConfig(agent, update, close));
        await act(async () => { await result.current.handleSave(); });
        expect(close).not.toHaveBeenCalled();
        expect(log).toHaveBeenCalledWith(expect.objectContaining({ severity: 'error' }));
        expect(log).not.toHaveBeenCalledWith(expect.objectContaining({ severity: 'success' }));
        expect(result.current.state.ui.saving).toBe(false);
        log.mockRestore();
    });
});
