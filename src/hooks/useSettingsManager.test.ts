/**
 * @docs ARCHITECTURE:Testing
 *
 * ### AI Assist Note
 * Regression coverage for the adjacent production module and its public contracts.
 *
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Contract, rendering, state transition, or error-handling regression.
 * - **Trace Scope**: Vitest assertions and test-local mocks.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useSettingsManager } from './useSettingsManager';
import * as settingsStore from '../stores/settings_store';

vi.mock('../services/system_api_service', () => ({
    system_api_service: {
        oversight: {
            update_governance_settings: vi.fn().mockResolvedValue({})
        }
    }
}));

describe('useSettingsManager', () => {
    beforeEach(() => {
        vi.restoreAllMocks();
        localStorage.clear();
    });

    it('initializes with stored settings', () => {
        const { result } = renderHook(() => useSettingsManager());
        expect(result.current.settings).toBeDefined();
        expect(result.current.is_saved).toBe(false);
    });

    it('handles input changes and numeric changes', () => {
        const { result } = renderHook(() => useSettingsManager());

        act(() => {
            result.current.handle_change({
                target: { name: 'active_profile', value: 'production', type: 'text' }
            } as any);
        });

        expect(result.current.settings.active_profile).toBe('production');

        act(() => {
            result.current.handle_numeric_change('neural_port', 9000);
        });

        expect(result.current.settings.neural_port).toBe(9000);
    });

    it('saves valid settings to store', async () => {
        const save_spy = vi.spyOn(settingsStore, 'save_settings').mockReturnValue(null);
        const { result } = renderHook(() => useSettingsManager());

        await act(async () => {
            await result.current.handle_save();
        });

        expect(save_spy).toHaveBeenCalled();
        expect(result.current.is_saved).toBe(true);
    });
});
