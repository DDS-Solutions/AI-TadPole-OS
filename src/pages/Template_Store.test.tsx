/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **Root View**: Manages the Template Store.test page. 
 * Connects the global state stores to the layout orchestration.
 */

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
import '@testing-library/jest-dom/vitest';
import Template_Store from './Template_Store';
import { use_settings_store } from '../stores/settings_store';

// Mock the settings store
vi.mock('../stores/settings_store', () => ({
    use_settings_store: vi.fn(),
    get_settings: vi.fn()
}));

import { get_settings } from '../stores/settings_store';

// Mock fetch for the component
const original_fetch = global.fetch;

describe('Template_Store Page', () => {
    const mock_registry_response = {
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

    const mock_swarm_config = {
        name: 'Finance AI',
        agents: [{ role: 'Auditor' }]
    };

    beforeEach(() => {
        // Reset settings store mock
        const mockSettings = { tadpole_os_url: 'http://localhost:8080', tadpole_os_api_key: 'test-key' };
        (use_settings_store as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
            settings: mockSettings
        });
        (get_settings as unknown as ReturnType<typeof vi.fn>).mockReturnValue(mockSettings);

        // Mock window.alert and dispatchEvent
        vi.spyOn(window, 'alert').mockImplementation(() => {});
        vi.spyOn(window, 'dispatchEvent');

        // Setup fetch mock
        global.fetch = vi.fn().mockImplementation(async (url: string) => {
            let res_body = {};
            if (url.includes('registry.json')) res_body = mock_registry_response;
            else if (url.includes('swarm.json')) res_body = mock_swarm_config;
            else if (url.includes('/engine/templates/install')) res_body = { status: 'success' };
            else return { ok: false, status: 404 };

            return {
                ok: true,
                status: 200,
                text: async () => JSON.stringify(res_body),
                json: async () => res_body
            };
        });
    });

    afterEach(() => {
        global.fetch = original_fetch;
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

        const search_input = screen.getByPlaceholderText(/Search templates/i);
        
        await act(async () => {
            fireEvent.change(search_input, { target: { value: 'Finance' } });
        });

        expect(screen.getByText('Finance AI Agents')).toBeInTheDocument();
        expect(screen.queryByText('Legal Assistant')).not.toBeInTheDocument();
    });

    it('filters templates by industry and size', async () => {
        render(<Template_Store />);
        expect(await screen.findByText('Finance AI Agents')).toBeInTheDocument();

        // Click Legal filter
        const industry_filters = screen.getByTestId('industry-filters');
        const legal_filter_button = within(industry_filters).getByRole('button', { name: /^Legal$/ });
        await act(async () => {
            fireEvent.click(legal_filter_button);
        });

        expect(screen.queryByText('Finance AI Agents')).not.toBeInTheDocument();
        expect(screen.getByText('Legal Assistant')).toBeInTheDocument();

        // Reset industry
        await act(async () => {
            fireEvent.click(within(industry_filters).getByRole('button', { name: /^All$/ }));
        });
        
        // Ensure both back
        expect(screen.getByText('Finance AI Agents')).toBeInTheDocument();

        // Click 50 seats size filter
        const size_filters = screen.getByTestId('size-filters');
        const seats_button = within(size_filters).getByRole('button', { name: /^50 Employees$/i });
        
        await act(async () => {
            fireEvent.click(seats_button);
        });

        expect(screen.getByText('Finance AI Agents')).toBeInTheDocument();
        expect(screen.queryByText('Legal Assistant')).not.toBeInTheDocument();
    });

    it('opens preview modal, fetches config, and installs template', async () => {
        render(<Template_Store />);
        expect(await screen.findByText('Finance AI Agents')).toBeInTheDocument();

        // Click preview on the first template
        const preview_buttons = screen.getAllByText(/Preview Swarm/i);
        
        await act(async () => {
            fireEvent.click(preview_buttons[0]);
        });

        // Wait for modal to load swarm.json
        expect(await screen.findByText('Swarm Configuration (swarm.json)')).toBeInTheDocument();
        
        // Assert the mock swarm config is displayed
        expect(screen.getByText(/"Auditor"/i)).toBeInTheDocument();

        // Click install inside modal
        const install_button = screen.getByText(/Deploy Swarm/i);
        
        await act(async () => {
            fireEvent.click(install_button);
        });

        // Verify POST request to install endpoint
        expect(global.fetch).toHaveBeenCalledWith(
            expect.stringContaining('/engine/templates/install'),
            expect.objectContaining({
                method: 'POST',
                body: JSON.stringify({
                    repository_url: 'https://github.com/DDS-Solutions/Tadpole-OS-Industry-Templates.git',
                    path: 'finance/fintech-nodes'
                })
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

