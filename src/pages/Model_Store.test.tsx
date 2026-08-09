/**
 * @docs ARCHITECTURE:UI-Pages
 * 
 * ### AI Assist Note
 * **@file Model_Store.test.tsx**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Model_Store_test]` in observability traces.
 */

/**
 * @file Model_Store.test.tsx
 * @description Unit tests for the Model_Store component ensuring route features, catalog fetching, and model pulling work 100%.
 */

import { render, screen, fireEvent, act } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import '@testing-library/jest-dom/vitest';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import Model_Store from './Model_Store';
import { tadpole_os_service } from '../services/tadpoleos_service';

vi.mock('../services/tadpoleos_service', () => ({
    tadpole_os_service: {
        get_model_catalog: vi.fn(),
        get_nodes: vi.fn(),
        pull_model: vi.fn(),
    }
}));

vi.mock('../components/ui', () => ({
    Tw_Empty_State: ({ title, description }: any) => (
        <div data-testid="empty-state">
            <h2>{title}</h2>
            <p>{description}</p>
        </div>
    ),
    Tooltip: ({ children, content }: any) => <span title={content}>{children}</span>,
}));

vi.mock('../i18n', () => ({
    i18n: {
        t: (key: string, options?: any) => {
            if (options && typeof options === 'object') {
                let interpolated = key;
                Object.keys(options).forEach(opt_key => {
                    interpolated = interpolated.replace(new RegExp(`{{${opt_key}}}`, 'g'), String(options[opt_key]));
                });
                return interpolated;
            }
            return key;
        },
    },
}));

const mock_catalog = [
    {
        id: 'llama3:8b',
        name: 'Llama 3 (8B)',
        provider: 'ollama',
        description: 'Meta\'s most capable 8B model.',
        size: '4.7GB',
        vram: '8GB',
        tags: ['General', 'Logic']
    },
    {
        id: 'phi3:latest',
        name: 'Phi-3 Mini',
        provider: 'ollama',
        description: 'Microsoft\'s SLM.',
        size: '2.3GB',
        vram: '4GB',
        tags: ['Fast', 'Efficiency']
    }
];

const mock_nodes = [
    { id: 'node-1', name: 'Bunker 1', status: 'online', address: 'localhost:8001' },
    { id: 'node-2', name: 'Bunker 2', status: 'online', address: 'localhost:8002' }
];

describe('Model_Store Component', () => {
    beforeEach(() => {
        vi.clearAllMocks();
        vi.mocked(tadpole_os_service.get_model_catalog).mockResolvedValue(mock_catalog);
        vi.mocked(tadpole_os_service.get_nodes).mockResolvedValue(mock_nodes);
    });

    it('renders the loading state initially', async () => {
        let resolve_catalog: any;
        const catalog_promise = new Promise(resolve => { resolve_catalog = resolve; });
        vi.mocked(tadpole_os_service.get_model_catalog).mockReturnValue(catalog_promise as any);

        render(
            <MemoryRouter>
                <Model_Store />
            </MemoryRouter>
        );
        expect(screen.getByText('model_store.loading_catalog')).toBeInTheDocument();

        await act(async () => {
            resolve_catalog(mock_catalog);
        });
    });

    it('renders catalog and nodes after successful fetch', async () => {
        render(
            <MemoryRouter>
                <Model_Store />
            </MemoryRouter>
        );

        expect(await screen.findByText('Llama 3 (8B)')).toBeInTheDocument();
        expect(screen.getByText('Phi-3 Mini')).toBeInTheDocument();
        expect(screen.getByText('model_store.active_nodes')).toBeInTheDocument();
        expect(screen.getByText('2')).toBeInTheDocument(); // 2 active nodes
    });

    it('renders error message when fetch fails', async () => {
        vi.mocked(tadpole_os_service.get_model_catalog).mockRejectedValue(new Error('Fetch failed'));

        render(
            <MemoryRouter>
                <Model_Store />
            </MemoryRouter>
        );

        expect(await screen.findByText('model_store.fetch_failed')).toBeInTheDocument();
    });

    it('filters catalog based on search input', async () => {
        render(
            <MemoryRouter>
                <Model_Store />
            </MemoryRouter>
        );

        // Wait for load
        expect(await screen.findByText('Llama 3 (8B)')).toBeInTheDocument();

        const search_input = screen.getByPlaceholderText('model_store.search_placeholder');
        
        // Search for 'Phi'
        fireEvent.change(search_input, { target: { value: 'Phi' } });

        expect(screen.queryByText('Llama 3 (8B)')).not.toBeInTheDocument();
        expect(screen.getByText('Phi-3 Mini')).toBeInTheDocument();
    });

    it('filters catalog based on category tab selections', async () => {
        render(
            <MemoryRouter>
                <Model_Store />
            </MemoryRouter>
        );

        // Wait for load
        expect(await screen.findByText('Llama 3 (8B)')).toBeInTheDocument();

        // Select 'general' tab
        const general_tab = screen.getByRole('button', { name: 'general' });
        fireEvent.click(general_tab);

        expect(screen.getByText('Llama 3 (8B)')).toBeInTheDocument();
        expect(screen.queryByText('Phi-3 Mini')).not.toBeInTheDocument();
    });

    it('handles model pulling when a node is selected', async () => {
        vi.mocked(tadpole_os_service.pull_model).mockResolvedValue({ status: 'success' });
        render(
            <MemoryRouter>
                <Model_Store />
            </MemoryRouter>
        );

        expect(await screen.findByText('Llama 3 (8B)')).toBeInTheDocument();

        // Click deploy on node-1 button for Llama 3
        const deploy_buttons = screen.getAllByRole('button');
        const llama3_node1_btn = deploy_buttons.find(btn => btn.textContent?.includes('node-1'));
        expect(llama3_node1_btn).toBeDefined();

        await act(async () => {
            fireEvent.click(llama3_node1_btn!);
        });

        expect(tadpole_os_service.pull_model).toHaveBeenCalledWith('llama3:8b', 'node-1');
    });

    it('renders empty state if no models match search', async () => {
        render(
            <MemoryRouter>
                <Model_Store />
            </MemoryRouter>
        );

        expect(await screen.findByText('Llama 3 (8B)')).toBeInTheDocument();

        const search_input = screen.getByPlaceholderText('model_store.search_placeholder');
        fireEvent.change(search_input, { target: { value: 'nonexistent-model' } });

        expect(screen.getByTestId('empty-state')).toBeInTheDocument();
        expect(screen.getByText('model_store.no_models_found')).toBeInTheDocument();
    });
});

// Metadata: [Model_Store_test]
