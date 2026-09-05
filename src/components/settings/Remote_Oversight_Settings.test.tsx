/**
 * @docs ARCHITECTURE:Testing
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Settings / Remote_Oversight_Settings.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { render, screen, fireEvent, act } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { api_request } from '../../services/base_api_service';
import { Remote_Oversight_Settings } from './Remote_Oversight_Settings';

vi.mock('../../services/base_api_service', async (importOriginal) => ({
    ...await importOriginal<typeof import('../../services/base_api_service')>(),
    api_request: vi.fn()
}));

const mocked_api_request = vi.mocked(api_request);

describe('Remote_Oversight_Settings Component', () => {
    beforeEach(() => {
        localStorage.clear();
        mocked_api_request.mockReset();
        mocked_api_request.mockImplementation((path: string) => Promise.resolve((
            path.endsWith('/pairing-token')
                ? { token: 'TP-PAIR-test-token', expires_in_seconds: 180 }
                : { devices: [] }
        ) as never));
    });

    afterEach(() => {
        vi.clearAllMocks();
    });

    it('renders bridge configuration header and mode options', async () => {
        render(<Remote_Oversight_Settings />);
        await act(async () => { await Promise.resolve(); });

        expect(screen.getByText(/Zero-Trust Remote Companion Bridge/i)).toBeDefined();
        expect(screen.getByText(/Local LAN Wi-Fi/i)).toBeDefined();
        expect(screen.getByText(/Tailscale Mesh/i)).toBeDefined();
    });

    it('toggles pairing mode between LAN and Tailscale', async () => {
        render(<Remote_Oversight_Settings />);
        await act(async () => { await Promise.resolve(); });

        const tailscale_btn = screen.getByText(/Tailscale Mesh/i).closest('button');
        expect(tailscale_btn).toBeDefined();

        fireEvent.click(tailscale_btn!);
        expect(localStorage.getItem('tadpole_remote_pairing_mode')).toContain('tailscale');
    });

    it('opens pairing dialog when Display Pairing QR Code is clicked', async () => {
        const user = userEvent.setup();
        render(<Remote_Oversight_Settings />);

        const pair_btn = screen.getByText(/Display Pairing QR Code/i);
        await user.click(pair_btn);

        expect(await screen.findByText(/Configure Companion Device Details/i)).toBeDefined();
    });
});
