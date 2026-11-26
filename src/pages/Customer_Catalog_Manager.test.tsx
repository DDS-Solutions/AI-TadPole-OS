/**
 * @docs ARCHITECTURE:UI
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Pages / Customer_Catalog_Manager.test
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor, act } from '@testing-library/react';
import { Customer_Catalog_Manager } from './Customer_Catalog_Manager';
import * as baseApi from '../services/base_api_service';
import type { BusinessCompany } from '../services/system/business_api';

const mockCompany: BusinessCompany = {
    company_key: 'Mock-2-Service',
    display_name: 'Harbor Plumbing Co',
    company_kind: 'service',
    profile: {
        description: 'Emergency plumbing services',
        hours: 'Mon-Fri 8am-6pm',
        support_email: 'support@harborplumbing.example',
        policy: '30-day workmanship guarantee',
    },
    offering: {
        display_reference: 'SVC-001',
        title: 'Emergency Drain Clearing',
        description: 'Same-day residential drain service',
        price_minor: 14900,
        currency: 'USD',
        source_updated_at: '2026-11-20T00:00:00Z',
    },
} as unknown as BusinessCompany;

describe('Customer_Catalog_Manager', () => {
    beforeEach(() => {
        vi.restoreAllMocks();
    });

    it('isolates embedded company workspace from global /a2a/v1/profile and omits hardware dummy items', async () => {
        const apiSpy = vi.spyOn(baseApi, 'api_request').mockResolvedValue({
            name: 'Global Card',
            model_profile: 'gemma4:e4b',
            info_cards: [],
        });

        render(
            <Customer_Catalog_Manager
                companyKey="Mock-2-Service"
                company={mockCompany}
                embedded={true}
            />
        );

        await waitFor(() => {
            expect(apiSpy).toHaveBeenCalledWith(
                '/a2a/v1/company-agent-card.json?company_key=Mock-2-Service',
                { method: 'GET' }
            );
        });

        // Must show the company's own offering and NOT hardcoded hardware-store dummy items
        expect(screen.getAllByText(/Emergency Drain Clearing/i).length).toBeGreaterThan(0);
        expect(screen.queryByText(/Heavy Duty Claw Hammer/i)).not.toBeInTheDocument();

        // Trigger a user edit in embedded mode — must NOT fire PUT /a2a/v1/profile
        const nameInput = screen.getByDisplayValue('Harbor Plumbing Co');
        vi.useFakeTimers();
        try {
            fireEvent.change(nameInput, { target: { value: 'Harbor Plumbing Updated' } });
            await act(async () => {
                await vi.advanceTimersByTimeAsync(800);
            });
        } finally {
            vi.useRealTimers();
        }

        const putCalls = apiSpy.mock.calls.filter(
            ([path, opts]) => path === '/a2a/v1/profile' && (opts as { method?: string })?.method === 'PUT'
        );
        expect(putCalls).toHaveLength(0);
    });

    it('fetches global agent card when no companyKey is supplied', async () => {
        const apiSpy = vi.spyOn(baseApi, 'api_request').mockResolvedValue({
            name: 'Global Card',
            model_profile: 'gemma4:e4b',
            info_cards: [],
        });

        render(<Customer_Catalog_Manager />);

        await waitFor(() => {
            expect(apiSpy).toHaveBeenCalledWith('/a2a/v1/company-agent-card.json', { method: 'GET' });
        });
    });
});
