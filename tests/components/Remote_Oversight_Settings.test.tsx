/**
 * @docs ARCHITECTURE:UI-Tests
 * 
 * ### AI Assist Note
 * Unit test suite for Remote_Oversight_Settings React component.
 * Refactored to use user-event for realistic user interactions, within() for scoped container assertions,
 * findByText for async state updates, and edge-case validation testing.
 * 
 * ### 🔍 Debugging & Observability
 * Traceability via `execution/parity_guard.py`.
 */

import { describe, it, expect, beforeEach } from 'vitest';
import React from 'react';
import '@testing-library/jest-dom';
import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Remote_Oversight_Settings } from '../../src/components/settings/Remote_Oversight_Settings';

describe('Remote_Oversight_Settings Component', () => {
    beforeEach(() => {
        localStorage.clear();
    });

    it('renders header and network endpoint information', () => {
        render(<Remote_Oversight_Settings />);

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

        // Click Simulate Companion Scan & Authorize Device
        const scanButton = await screen.findByRole('button', { name: /Simulate Companion Scan & Authorize Device/i });
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
        render(<Remote_Oversight_Settings />);

        const pairedContainer = screen.getByTestId('paired-devices-container');
        expect(within(pairedContainer).getByText(/Android Smartphone \(Pixel 8\)/i)).toBeInTheDocument();

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
        render(<Remote_Oversight_Settings />);

        const pairedContainer = screen.getByTestId('paired-devices-container');
        expect(within(pairedContainer).getByText(/Android Smartphone \(Pixel 8\)/i)).toBeInTheDocument();

        const revokeButton = within(pairedContainer).getByTitle('Revoke Device Access');
        await user.click(revokeButton);

        expect(within(pairedContainer).queryByTitle('Revoke Device Access')).not.toBeInTheDocument();
        expect(within(pairedContainer).getByText(/No mobile companion devices paired yet/i)).toBeInTheDocument();

        const auditContainer = screen.getByTestId('audit-log-container');
        expect(within(auditContainer).getByText(/REVOKED/i)).toBeInTheDocument();
    });

    it('handles fallback defaults when user inputs are cleared', async () => {
        const user = userEvent.setup();
        render(<Remote_Oversight_Settings />);

        await user.click(screen.getByRole('button', { name: /Display Pairing QR Code/i }));

        const userInput = screen.getByLabelText(/User \/ Operator Name/i);
        const deviceInput = screen.getByLabelText(/Companion Device Name/i);

        await user.clear(userInput);
        await user.clear(deviceInput);

        await user.click(screen.getByRole('button', { name: /Generate QR Code/i }));
        const scanButton = await screen.findByRole('button', { name: /Simulate Companion Scan & Authorize Device/i });
        await user.click(scanButton);

        const pairedContainer = screen.getByTestId('paired-devices-container');
        expect(await within(pairedContainer).findByText(/Android Companion Device/i)).toBeInTheDocument();
        expect(within(pairedContainer).getAllByText(/Sovereign Operator/i).length).toBeGreaterThan(0);
    });
});

// Metadata: [Remote_Oversight_Settings.test]

