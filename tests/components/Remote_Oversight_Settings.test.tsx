/**
 * @docs ARCHITECTURE:UI-Tests
 *
 * ### AI Context Alignment
 * - **Subsystem**: Test Verification Suite / Remote_Oversight_Settings.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import React from 'react';
import '@testing-library/jest-dom';
import { render, screen, within, act } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { api_request } from '../../src/services/base_api_service';
import { Remote_Oversight_Settings } from '../../src/components/settings/Remote_Oversight_Settings';

vi.mock('../../src/services/base_api_service', async (importOriginal) => ({
    ...await importOriginal<typeof import('../../src/services/base_api_service')>(),
    api_request: vi.fn()
}));

const mocked_api_request = vi.mocked(api_request);
const server_device = (overrides: Record<string, string> = {}) => ({
    id: 'dev-server-01',
    name: 'Android Smartphone (Pixel 8)',
    user_name: 'Sovereign Operator',
    paired_at: '2026-07-31 13:45',
    public_key: `ed25519:${'ab'.repeat(32)}`,
    status: 'Authorized',
    ...overrides
});

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

    it('renders header and network endpoint information', async () => {
        render(<Remote_Oversight_Settings />);
        await act(async () => { await Promise.resolve(); });

        expect(screen.getByText(/Remote Oversight & Mobile Mesh Settings/i)).toBeInTheDocument();
        expect(screen.getByText(/Zero-Trust Remote Companion Bridge/i)).toBeInTheDocument();
        expect(screen.getByText(/Active \/ Listening/i)).toBeInTheDocument();
        expect(screen.getByDisplayValue(/10.0.0.1:8000/i)).toBeInTheDocument();
    });

    it('opens request dialog box for device name and user name before QR screen', async () => {
        const user = userEvent.setup();
        render(<Remote_Oversight_Settings />);

        const displayPairButton = screen.getByRole('button', { name: /Display Pairing QR Code/i });
        expect(displayPairButton).toBeInTheDocument();

        // Click to open Step 1: Request Dialog Box
        await user.click(displayPairButton);
        expect(await screen.findByText(/Configure Companion Device Details/i)).toBeInTheDocument();

        const userInput = screen.getByLabelText(/User \/ Operator Name/i);
        const deviceInput = screen.getByLabelText(/Companion Device Name/i);

        await user.clear(userInput);
        await user.type(userInput, 'Alex Mercer');
        await user.clear(deviceInput);
        await user.type(deviceInput, 'Pixel 9 Pro');

        // Click Generate QR Code to move to Step 2: QR Screen
        const generateQrButton = screen.getByRole('button', { name: /Generate QR Code/i });
        await user.click(generateQrButton);

        expect(await screen.findByText(/Scan QR Code to Complete Pairing/i)).toBeInTheDocument();
        expect(screen.getByText(/Alex Mercer/i)).toBeInTheDocument();
        expect(screen.getByText(/Pixel 9 Pro/i)).toBeInTheDocument();
        expect(screen.getByText(/Pairing Challenge Code:/i)).toBeInTheDocument();
    });

    it('adds paired device with key pair, date and time, user and device name upon scan completion and logs audit record', async () => {
        const user = userEvent.setup();
        let device_reads = 0;
        mocked_api_request.mockImplementation((path: string) => {
            if (path.endsWith('/pairing-token')) {
                return Promise.resolve({ token: 'TP-PAIR-test-token', expires_in_seconds: 180 } as never);
            }
            device_reads += 1;
            return Promise.resolve({
                devices: device_reads >= 3
                    ? [server_device({ name: 'Galaxy Tab S9', user_name: 'Elena Rostova' })]
                    : []
            } as never);
        });
        render(<Remote_Oversight_Settings />);

        // Open pairing flow
        await user.click(screen.getByRole('button', { name: /Display Pairing QR Code/i }));

        const userInput = screen.getByLabelText(/User \/ Operator Name/i);
        const deviceInput = screen.getByLabelText(/Companion Device Name/i);

        await user.clear(userInput);
        await user.type(userInput, 'Elena Rostova');
        await user.clear(deviceInput);
        await user.type(deviceInput, 'Galaxy Tab S9');

        // Proceed to QR
        await user.click(screen.getByRole('button', { name: /Generate QR Code/i }));

        // Refresh until the backend confirms that the companion consumed the token.
        const scanButton = await screen.findByRole('button', { name: /Refresh Pairing Status/i });
        await user.click(scanButton);

        // Verify newly authorized device appears in Paired Devices container specifically
        const pairedContainer = screen.getByTestId('paired-devices-container');
        expect(await within(pairedContainer).findByText(/Galaxy Tab S9/i)).toBeInTheDocument();
        expect(within(pairedContainer).getByText(/Elena Rostova/i)).toBeInTheDocument();
        expect(within(pairedContainer).getAllByText(/Key:/i).length).toBeGreaterThan(0);

        // Verify security audit log record was appended in Audit Log container specifically
        const auditContainer = screen.getByTestId('audit-log-container');
        expect(within(auditContainer).getByText(/Galaxy Tab S9/i)).toBeInTheDocument();
        expect(within(auditContainer).getByText(/Elena Rostova/i)).toBeInTheDocument();
        expect(within(auditContainer).getAllByText(/PAIRED/i).length).toBeGreaterThan(0);
    });

    it('allows editing device name and user name and logs audit record in specific containers', async () => {
        const user = userEvent.setup();
        mocked_api_request.mockImplementation((path: string, options?: RequestInit) => {
            if (options?.method === 'PUT') {
                return Promise.resolve(server_device({
                    name: 'Pixel 8 (Field Edition)',
                    user_name: 'Lead Engineer'
                }) as never);
            }
            return Promise.resolve({ devices: [server_device()] } as never);
        });
        render(<Remote_Oversight_Settings />);

        const pairedContainer = screen.getByTestId('paired-devices-container');
        expect(await within(pairedContainer).findByText(/Android Smartphone \(Pixel 8\)/i)).toBeInTheDocument();

        const editButton = within(pairedContainer).getByTitle('Edit Device & User Details');
        await user.click(editButton);

        expect(await screen.findByText(/Edit Authorized Paired Device/i)).toBeInTheDocument();

        const userInput = screen.getByLabelText('Edit User Name');
        const nameInput = screen.getByLabelText('Edit Device Name');

        await user.clear(userInput);
        await user.type(userInput, 'Lead Engineer');
        await user.clear(nameInput);
        await user.type(nameInput, 'Pixel 8 (Field Edition)');

        await user.click(screen.getByRole('button', { name: /Save Changes/i }));

        // Check within Paired Devices container
        expect(await within(pairedContainer).findByText(/Pixel 8 \(Field Edition\)/i)).toBeInTheDocument();
        expect(within(pairedContainer).getByText(/Lead Engineer/i)).toBeInTheDocument();

        // Check within Audit Log container
        const auditContainer = screen.getByTestId('audit-log-container');
        expect(within(auditContainer).getByText(/Pixel 8 \(Field Edition\)/i)).toBeInTheDocument();
        expect(within(auditContainer).getByText(/EDITED/i)).toBeInTheDocument();
    });

    it('allows revoking an authorized companion device and logs audit record', async () => {
        const user = userEvent.setup();
        mocked_api_request.mockImplementation((_path: string, options?: RequestInit) => Promise.resolve((
            options?.method === 'POST' ? { status: 'revoked' } : { devices: [server_device()] }
        ) as never));
        render(<Remote_Oversight_Settings />);

        const pairedContainer = screen.getByTestId('paired-devices-container');
        expect(await within(pairedContainer).findByText(/Android Smartphone \(Pixel 8\)/i)).toBeInTheDocument();

        const revokeButton = within(pairedContainer).getByTitle('Revoke Device Access');
        await user.click(revokeButton);

        expect(within(pairedContainer).queryByTitle('Revoke Device Access')).not.toBeInTheDocument();
        expect(within(pairedContainer).getByText(/No mobile companion devices paired yet/i)).toBeInTheDocument();

        const auditContainer = screen.getByTestId('audit-log-container');
        expect(within(auditContainer).getByText(/REVOKED/i)).toBeInTheDocument();
    });

    it('renders server-confirmed device metadata when pairing inputs are cleared', async () => {
        const user = userEvent.setup();
        let device_reads = 0;
        mocked_api_request.mockImplementation((path: string) => {
            if (path.endsWith('/pairing-token')) {
                return Promise.resolve({ token: 'TP-PAIR-test-token', expires_in_seconds: 180 } as never);
            }
            device_reads += 1;
            return Promise.resolve({
                devices: device_reads >= 3
                    ? [server_device({ name: 'Android Companion Device', user_name: 'Sovereign Operator' })]
                    : []
            } as never);
        });
        render(<Remote_Oversight_Settings />);

        await user.click(screen.getByRole('button', { name: /Display Pairing QR Code/i }));

        const userInput = screen.getByLabelText(/User \/ Operator Name/i);
        const deviceInput = screen.getByLabelText(/Companion Device Name/i);

        await user.clear(userInput);
        await user.clear(deviceInput);

        await user.click(screen.getByRole('button', { name: /Generate QR Code/i }));
        const scanButton = await screen.findByRole('button', { name: /Refresh Pairing Status/i });
        await user.click(scanButton);

        const pairedContainer = screen.getByTestId('paired-devices-container');
        expect(await within(pairedContainer).findByText(/Android Companion Device/i)).toBeInTheDocument();
        expect(within(pairedContainer).getAllByText(/Sovereign Operator/i).length).toBeGreaterThan(0);
    });

    it('does not open the pairing flow when the server cannot mint a token', async () => {
        const user = userEvent.setup();
        mocked_api_request.mockImplementation((path: string) => {
            if (path.endsWith('/pairing-token')) {
                return Promise.reject(new Error('Pairing token service unavailable'));
            }
            return Promise.resolve({ devices: [] } as never);
        });
        render(<Remote_Oversight_Settings />);

        await user.click(screen.getByRole('button', { name: /Display Pairing QR Code/i }));

        expect(await screen.findByRole('alert')).toHaveTextContent(/Pairing token service unavailable/i);
        expect(screen.queryByText(/Configure Companion Device Details/i)).not.toBeInTheDocument();
    });
});
