/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Validation of the Neural Template and Blueprint repository.** 
 * Verifies the retrieval, preview, and instantiation of standardized agent and swarm configurations from the remote registry. 
 * Mocks `global.fetch` to intercept registry and configuration requests, and spies on `window.dispatchEvent` for installation signals.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Incompatible schema versions in local templates causing instantiation failures or missing metadata in the blueprint preview pane.
 * - **Telemetry Link**: Search `[Template_Store.test]` in tracing logs.
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
import { MemoryRouter } from 'react-router-dom';
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
            else if (url.includes('knowledge.json')) res_body = [
                {
                    title: "Test Playbook",
                    description: "A playbook description",
                    topic: "test-topic",
                    concept_type: "playbook",
                    resource_uri: "https://example.com/sop",
                    tags: "test, playbook"
                }
            ];
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
        render(
            <MemoryRouter>
                <Template_Store />
            </MemoryRouter>
        );
        expect(await screen.findByText('Swarm Template Store')).toBeInTheDocument();
        expect(await screen.findByText('Finance AI Agents')).toBeInTheDocument();
        expect(screen.getByText('Legal Assistant')).toBeInTheDocument();

        // Check assigned industries (multiple due to badges and filters)
        expect(screen.getAllByText('Finance').length).toBeGreaterThan(0);
        expect(screen.getAllByText('Legal').length).toBeGreaterThan(0);
        expect(screen.getByText('50 Seats')).toBeInTheDocument();
    });

    it('filters templates by search query', async () => {
        render(
            <MemoryRouter>
                <Template_Store />
            </MemoryRouter>
        );
        expect(await screen.findByText('Finance AI Agents')).toBeInTheDocument();

        const search_input = screen.getByPlaceholderText(/Search templates/i);
        
        await act(async () => {
            fireEvent.change(search_input, { target: { value: 'Finance' } });
        });

        expect(screen.getByText('Finance AI Agents')).toBeInTheDocument();
        expect(screen.queryByText('Legal Assistant')).not.toBeInTheDocument();
    });

    it('filters templates by industry and size', async () => {
        render(
            <MemoryRouter>
                <Template_Store />
            </MemoryRouter>
        );
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
        render(
            <MemoryRouter>
                <Template_Store />
            </MemoryRouter>
        );
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

        // Assert that the playbook preview is displayed
        expect(await screen.findByText('Playbooks & Institutional Knowledge (OKF)')).toBeInTheDocument();
        expect(screen.getByText('Test Playbook')).toBeInTheDocument();
        expect(screen.getByText('A playbook description')).toBeInTheDocument();

        // Click install inside modal
        const install_button = screen.getByText(/Deploy Swarm/i);
        
        await act(async () => {
            fireEvent.click(install_button);
        });

        // Dispatches event and marks as installed (awaited due to lazy-loading microtask delay)
        expect(await screen.findByText(/Installed/i)).toBeInTheDocument();

        // Verify POST request to install endpoint
        expect(global.fetch).toHaveBeenCalledWith(
            expect.stringContaining('/engine/templates/install'),
            expect.objectContaining({
                method: 'POST',
                body: JSON.stringify({
                    repository_url: 'https://github.com/DDS-Solutions/AI-Tadpole-OS-Industry-Templates.git',
                    path: 'finance/fintech-nodes'
                })
            })
        );

        expect(window.dispatchEvent).toHaveBeenCalled();
        
        // Modal is closed after install
        expect(screen.queryByText('Swarm Configuration (swarm.json)')).not.toBeInTheDocument();
    });

    it('displays error message if fetching registry fails', async () => {
        global.fetch = vi.fn().mockImplementation(async () => {
            throw new Error('Network Error');
        });

        render(
            <MemoryRouter>
                <Template_Store />
            </MemoryRouter>
        );
        expect(await screen.findByText('Network Error')).toBeInTheDocument();
    });

    it('renders repository action buttons and triggers registry scan', async () => {
        render(
            <MemoryRouter>
                <Template_Store />
            </MemoryRouter>
        );

        // Wait for initial load to finish so the scan button is enabled
        expect(await screen.findByText('Finance AI Agents')).toBeInTheDocument();

        // Verify Scan Repo and Make your own templates buttons exist
        const scan_button = await screen.findByRole('button', { name: /Scan Repo/i });
        const explore_button = screen.getByRole('link', { name: /Make your own templates/i });

        expect(scan_button).toBeInTheDocument();
        expect(explore_button).toBeInTheDocument();
        expect(explore_button).toHaveAttribute('href', 'https://dds-solutions.github.io/AI-Tadpole-OS-Industry-Templates/');

        // Clear fetch call counts
        vi.mocked(global.fetch).mockClear();

        // Click Scan Repo button
        await act(async () => {
            fireEvent.click(scan_button);
        });

        // Verify it triggers a fetch to registry.json
        expect(global.fetch).toHaveBeenCalledWith(
            expect.stringContaining('registry.json')
        );
    });

    it('allows importing a downloaded swarm archive file and opening preview modal', async () => {
        render(
            <MemoryRouter>
                <Template_Store />
            </MemoryRouter>
        );

        expect(await screen.findByText('Finance AI Agents')).toBeInTheDocument();

        const downloaded_swarms_button = screen.getByText(/(Downloaded Swarms|btn_downloaded_swarms)/i);
        expect(downloaded_swarms_button).toBeInTheDocument();

        const swarmData = {
            id: 'custom-local-swarm',
            name: 'Local Customs Swarm',
            description: 'A custom downloaded swarm file from web builder',
            industry: 'Customs',
            company_size: 100,
            tags: ['custom', 'local'],
            agents: [{ role: 'Custom Agent' }]
        };
        const jsonContent = JSON.stringify(swarmData);
        const mockFile = new File([jsonContent], 'custom_swarm.json', { type: 'application/json' });
        Object.defineProperty(mockFile, 'text', { value: () => Promise.resolve(jsonContent) });

        const fileInput = downloaded_swarms_button.parentElement?.querySelector('input[type="file"]') as HTMLInputElement;
        expect(fileInput).toBeInTheDocument();

        await act(async () => {
            fireEvent.change(fileInput, { target: { files: [mockFile] } });
            await new Promise(r => setTimeout(r, 300));
        });

        // Verifies the downloaded swarm opens in preview modal with badges
        const swarmElements = await screen.findAllByText('Local Customs Swarm', {}, { timeout: 3000 });
        expect(swarmElements.length).toBeGreaterThan(0);
        expect(screen.getByText('Swarm Configuration (swarm.json)')).toBeInTheDocument();
        expect(screen.getByText(/Custom Agent/i)).toBeInTheDocument();
    });
});


// Metadata: [Template_Store_test]
