/**
 * @docs ARCHITECTURE:Testing
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend React Hooks / useSettingsManager.test
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
import { useSettingsManager } from './useSettingsManager';
import * as settingsStore from '../stores/settings_store';

vi.mock('../services/system_api_service', () => ({
    system_api_service: {
        oversight: {
            get_governance_settings: vi.fn().mockResolvedValue({}),
            update_governance_settings: vi.fn().mockResolvedValue({})
        }
    }
}));

describe('useSettingsManager', () => {
    beforeEach(() => {
        vi.restoreAllMocks();
        localStorage.clear();
    });

    it('initializes with stored settings', async () => {
        let hookResult: any;
        await act(async () => {
            hookResult = renderHook(() => useSettingsManager());
        });
        expect(hookResult.result.current.settings).toBeDefined();
        expect(hookResult.result.current.is_saved).toBe(false);
    });

    it('handles input changes and numeric changes', async () => {
        let hookResult: any;
        await act(async () => {
            hookResult = renderHook(() => useSettingsManager());
        });

        act(() => {
            hookResult.result.current.handle_change({
                target: { name: 'active_profile', value: 'production', type: 'text' }
            } as any);
        });

        expect(hookResult.result.current.settings.active_profile).toBe('production');

        act(() => {
            hookResult.result.current.handle_numeric_change('neural_port', 9000);
        });

        expect(hookResult.result.current.settings.neural_port).toBe(9000);
    });

    it('saves valid settings to store', async () => {
        const save_spy = vi.spyOn(settingsStore, 'save_settings').mockReturnValue(null);
        let hookResult: any;
        await act(async () => {
            hookResult = renderHook(() => useSettingsManager());
        });

        await act(async () => {
            await hookResult.result.current.handle_save();
        });

        expect(save_spy).toHaveBeenCalled();
        expect(hookResult.result.current.is_saved).toBe(true);
    });
});
