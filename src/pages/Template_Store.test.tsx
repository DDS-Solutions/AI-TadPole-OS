/**
 * @file Template_Store.test.tsx
 * @description Suite for the Swarm Template Store (Marketplace) page.
 * @module Pages/Template_Store
 * @testedBehavior
 * - Registry Discovery: Fetching and filtering industry-specific swarm templates.
 * - Pre-view Logic: Modal-based preview of swarm configuration (swarm.json).
 * - Installation: Verification of template deployment to the local engine.
 * @aiContext
 * - Mocks global.fetch to intercept registry and configuration requests.
 * - Spies on window.dispatchEvent to verify successful installation signals.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, fireEvent, act, within } from '@testing-library/react';
import Template_Store from './Template_Store';
import { use_settings_store } from '../stores/settings_store';

// Mock the settings store
vi.mock('../stores/settings_store', () => ({
    use_settings_store: vi.fn()
}));

// Mock fetch for the component
const originalFetch = global.fetch;

describe('Template_Store Page', () => {
    const mockRegistryResponse = {
        templates: [
            {
                id: 'tmpl-1',
                name: 'Finance AI Agents',
                description: 'A suite of financial agents',
                industry: 'Finance',
                company_size: 50,
                tags: ['finance', 'fintech'],
                path: 'finance/fintech-nodes'
            },
            {
                id: 'tmpl-2',
                name: 'Legal Assistant',
                description: 'Review legal documents',
                industry: 'Legal',
                company_size: null,
                tags: ['legal'],
                path: 'legal/document-reviewer'
            }
        ]
    };

    const mockSwarmConfig = {
        name: 'Finance AI',
        agents: [{ role: 'Auditor' }]
    };

    beforeEach(() => {
        // Reset settings store mock
        (use_settings_store as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
            settings: { TadpoleOSUrl: 'http://localhost:8080', TadpoleOSApiKey: 'test-key' }
        });

        // Mock window.alert and dispatchEvent
        vi.spyOn(window, 'alert').mockImplementation(() => {});
        vi.spyOn(window, 'dispatchEvent');

        // Setup fetch mock
        global.fetch = vi.fn().mockImplementation(async (url: string) => {
            if (url.includes('registry.json')) {
                return {
                    ok: true,
                    json: async () => mockRegistryResponse
                };
            }
            if (url.includes('swarm.json')) {
                return {
                    ok: true,
                    json: async () => mockSwarmConfig
                };
            }
            if (url.includes('/engine/templates/install')) {
                return {
                    ok: true,
                    json: async () => ({ status: 'success' })
                };
            }
            return { ok: false, status: 404 };
        });
    });

    afterEach(() => {
        global.fetch = originalFetch;
        vi.restoreAllMocks();
    });

    it('renders the store and fetches the registry', async () => {
        render(<Template_Store />);
        expect(await screen.findByText('Swarm Template Store')).toBeInTheDocument();
        expect(await screen.findByText('Finance AI Agents')).toBeInTheDocument();
        expect(screen.getByText('Legal Assistant')).toBeInTheDocument();

        // Check assigned industries (multiple due to badges and filters)
        expect(screen.getAllByText('Finance').length).toBeGreaterThan(0);
        expect(screen.getAllByText('Legal').length).toBeGreaterThan(0);
        expect(screen.getByText('50 Seats')).toBeInTheDocument();
    });

    it('filters templates by search query', async () => {
        render(<Template_Store />);
        expect(await screen.findByText('Finance AI Agents')).toBeInTheDocument();

        const searchInput = screen.getByPlaceholderText(/Search templates/i);
        
        await act(async () => {
            fireEvent.change(searchInput, { target: { value: 'Finance' } });
        });

        expect(screen.getByText('Finance AI Agents')).toBeInTheDocument();
        expect(screen.queryByText('Legal Assistant')).not.toBeInTheDocument();
    });

    it('filters templates by industry and size', async () => {
        render(<Template_Store />);
        expect(await screen.findByText('Finance AI Agents')).toBeInTheDocument();

        // Click Legal filter
        const industryFilters = screen.getByTestId('industry-filters');
        const legalFilterButton = within(industryFilters).getByRole('button', { name: /^Legal$/ });
        await act(async () => {
            fireEvent.click(legalFilterButton);
        });

        expect(screen.queryByText('Finance AI Agents')).not.toBeInTheDocument();
        expect(screen.getByText('Legal Assistant')).toBeInTheDocument();

        // Reset industry
        await act(async () => {
            fireEvent.click(within(industryFilters).getByRole('button', { name: /^All$/ }));
        });
        
        // Ensure both back
        expect(screen.getByText('Finance AI Agents')).toBeInTheDocument();

        // Click 50 seats size filter
        const sizeFilters = screen.getByTestId('size-filters');
        const seatsButton = within(sizeFilters).getByRole('button', { name: /^50 Employees$/i });
        
        await act(async () => {
            fireEvent.click(seatsButton);
        });

        expect(screen.getByText('Finance AI Agents')).toBeInTheDocument();
        expect(screen.queryByText('Legal Assistant')).not.toBeInTheDocument();
    });

    it('opens preview modal, fetches config, and installs template', async () => {
        render(<Template_Store />);
        expect(await screen.findByText('Finance AI Agents')).toBeInTheDocument();

        // Click preview on the first template
        const previewButtons = screen.getAllByText(/Pre View Swarm Configuration/i);
        
        await act(async () => {
            fireEvent.click(previewButtons[0]);
        });

        // Wait for modal to load swarm.json
        expect(await screen.findByText('Swarm Configuration (swarm.json)')).toBeInTheDocument();
        
        // Assert the mock swarm config is displayed
        expect(screen.getByText(/"Auditor"/i)).toBeInTheDocument();

        // Click install inside modal
        const installButton = screen.getByText(/Install Configuration/i);
        
        await act(async () => {
            fireEvent.click(installButton);
        });

        // Verify POST request to install endpoint
        expect(global.fetch).toHaveBeenCalledWith(
            'http://localhost:8080/engine/templates/install',
            expect.objectContaining({
                method: 'POST',
                body: expect.stringContaining('finance/fintech-nodes')
            })
        );

        // Dispatches event and marks as installed
        expect(window.dispatchEvent).toHaveBeenCalled();
        expect(screen.getByText(/Installed/i)).toBeInTheDocument();
        
        // Modal is closed after install
        expect(screen.queryByText('Swarm Configuration (swarm.json)')).not.toBeInTheDocument();
    });

    it('displays error message if fetching registry fails', async () => {
        global.fetch = vi.fn().mockImplementation(async () => {
            throw new Error('Network Error');
        });

        render(<Template_Store />);
        expect(await screen.findByText('Network Error')).toBeInTheDocument();
    });
});
