/**
 * @docs ARCHITECTURE:UI-Components
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Settings / Connection_Settings.test
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Connection_Settings } from './Connection_Settings';
import type { Tadpole_Settings } from '../../stores/settings_store';

const mockSettings: Tadpole_Settings = {
    tadpole_os_url: 'http://127.0.0.1:8000',
    tadpole_os_api_key: 'deploy-token-1234567890',
    tadpole_os_admin_key: '',
} as Tadpole_Settings;

describe('Connection_Settings', () => {
    it('renders operational token and session-only step-up administrative token inputs', () => {
        const handleChange = vi.fn();
        render(
            <Connection_Settings
                settings={mockSettings}
                handle_change={handleChange}
                validation_error={null}
            />
        );

        const adminInput = screen.getByLabelText(/Step-Up Administrative Token/i) as HTMLInputElement;
        expect(adminInput).toBeInTheDocument();
        expect(adminInput.type).toBe('password');
        expect(screen.getByText(/Session-Only \/ In-Memory/i)).toBeInTheDocument();

        fireEvent.change(adminInput, { target: { name: 'tadpole_os_admin_key', value: 'admin-secret-token-2026' } });
        expect(handleChange).toHaveBeenCalled();
    });
});
