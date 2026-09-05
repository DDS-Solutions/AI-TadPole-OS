/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Template_Store / Mcp_Secrets_Wizard_Modal.test
 * - **Primary Entrypoints**: none (test harness)
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: `Mcp_Secrets_Wizard_Modal.test.tsx`
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, act } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import { Mcp_Secrets_Wizard_Modal } from './Mcp_Secrets_Wizard_Modal';
import { system_api_service } from '../../services/system_api_service';

vi.mock('../../services/system_api_service', () => ({
    system_api_service: {
        engine: {
            update_environment: vi.fn().mockResolvedValue({ status: 'success', updated_keys: [] })
        }
    }
}));

describe('Mcp_Secrets_Wizard_Modal Component', () => {
    const mockPlaceholders = [
        { server: 'stripe-mcp', variable: 'STRIPE_API_KEY', description: 'Stripe Secret Key' },
        { server: 'stripe-mcp', variable: 'STRIPE_WEBHOOK_SECRET' },
        { server: 'hubspot-mcp', variable: 'HUBSPOT_ACCESS_TOKEN' }
    ];

    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('does not render when isOpen is false or placeholders are empty', () => {
        const { rerender } = render(
            <Mcp_Secrets_Wizard_Modal
                isOpen={false}
                placeholders={mockPlaceholders}
                onClose={vi.fn()}
            />
        );
        expect(screen.queryByText(/Connect External Tools & Services/i)).not.toBeInTheDocument();

        rerender(
            <Mcp_Secrets_Wizard_Modal
                isOpen={true}
                placeholders={[]}
                onClose={vi.fn()}
            />
        );
        expect(screen.queryByText(/Connect External Tools & Services/i)).not.toBeInTheDocument();
    });

    it('renders server groups and variables when open', () => {
        render(
            <Mcp_Secrets_Wizard_Modal
                isOpen={true}
                placeholders={mockPlaceholders}
                onClose={vi.fn()}
            />
        );

        expect(screen.getByText('stripe-mcp')).toBeInTheDocument();
        expect(screen.getByText('hubspot-mcp')).toBeInTheDocument();
        expect(screen.getByText('STRIPE_API_KEY')).toBeInTheDocument();
        expect(screen.getByText('STRIPE_WEBHOOK_SECRET')).toBeInTheDocument();
        expect(screen.getByText('HUBSPOT_ACCESS_TOKEN')).toBeInTheDocument();
        expect(screen.getByText('Stripe Secret Key')).toBeInTheDocument();
    });

    it('allows toggling show/hide password visibility', async () => {
        render(
            <Mcp_Secrets_Wizard_Modal
                isOpen={true}
                placeholders={mockPlaceholders}
                onClose={vi.fn()}
            />
        );

        const stripeInput = screen.getByPlaceholderText('Enter STRIPE_API_KEY...');
        expect(stripeInput).toHaveAttribute('type', 'password');

        const toggleBtn = screen.getAllByLabelText('Reveal secret')[0];
        await act(async () => {
            fireEvent.click(toggleBtn);
        });

        expect(stripeInput).toHaveAttribute('type', 'text');
    });

    it('saves entered environment variables and invokes callback', async () => {
        const onClose = vi.fn();
        const onSaveSuccess = vi.fn();

        render(
            <Mcp_Secrets_Wizard_Modal
                isOpen={true}
                placeholders={mockPlaceholders}
                onClose={onClose}
                onSaveSuccess={onSaveSuccess}
            />
        );

        const stripeInput = screen.getByPlaceholderText('Enter STRIPE_API_KEY...');
        const hubspotInput = screen.getByPlaceholderText('Enter HUBSPOT_ACCESS_TOKEN...');

        await act(async () => {
            fireEvent.change(stripeInput, { target: { value: 'sk_live_12345' } });
            fireEvent.change(hubspotInput, { target: { value: 'pat_live_67890' } });
        });

        const saveButton = screen.getByText(/Save & Activate Connectors/i);
        await act(async () => {
            fireEvent.click(saveButton);
        });

        expect(system_api_service.engine.update_environment).toHaveBeenCalledWith({
            STRIPE_API_KEY: 'sk_live_12345',
            HUBSPOT_ACCESS_TOKEN: 'pat_live_67890'
        });
        expect(onSaveSuccess).toHaveBeenCalled();
        expect(onClose).toHaveBeenCalled();
    });

    it('closes on Skip / Configure Later button click', async () => {
        const onClose = vi.fn();
        render(
            <Mcp_Secrets_Wizard_Modal
                isOpen={true}
                placeholders={mockPlaceholders}
                onClose={onClose}
            />
        );

        const skipBtn = screen.getByRole('button', { name: /Configure Later in Settings/i });
        await act(async () => {
            fireEvent.click(skipBtn);
        });

        expect(onClose).toHaveBeenCalled();
    });
});
