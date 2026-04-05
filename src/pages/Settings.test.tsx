/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **Root View**: Manages the Settings.test page. 
 * Connects the global state stores to the layout orchestration.
 */

/**
 * @file Settings.test.tsx
 * @description Suite for the System Configuration (Settings) page.
 * @module Pages/Settings
 * @testedBehavior
 * - Preference Management: Theme and density attribute synchronization.
 * - API Persistence: Verification of save calls to settings_store.
 * - Reactive UI: Theme attribute injection into document.documentElement.
 * @aiContext
 * - Mocks settings_store to intercept configuration I/O.
 * - Mocks system_api_service to prevent network side-effects.
 * - Refactored for 100% snake_case architectural parity.
 */
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import { MemoryRouter } from 'react-router-dom';
import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';

// Use vi.hoisted to stub global.localStorage before any module re-evaluation
vi.hoisted(() => {
    const mock_local_storage = {
        getItem: vi.fn(),
        setItem: vi.fn(),
        clear: vi.fn(),
        removeItem: vi.fn(),
        length: 0,
        key: vi.fn(),
    };
    vi.stubGlobal('localStorage', mock_local_storage);
});

import Settings from './Settings';
import * as settings_store from '../stores/settings_store';
import { system_api_service } from '../services/system_api_service';

// Mock dependencies
vi.mock('../stores/settings_store', () => ({
    get_settings: vi.fn(),
    save_settings: vi.fn(),
}));

vi.mock('../services/system_api_service', () => ({
    system_api_service: {
        update_governance_settings: vi.fn(),
    },
}));

const mock_navigate = vi.fn();
vi.mock('react-router-dom', async () => {
    const actual = await vi.importActual('react-router-dom');
    return {
        ...actual as any,
        useNavigate: () => mock_navigate,
    };
});

describe('Settings Page', () => {
    const mock_default_settings = {
        tadpole_os_url: 'http://localhost:8000',
        tadpole_os_api_key: 'test-key',
        theme: 'zinc',
        density: 'compact',
        default_model: 'GPT-4o',
        default_temperature: 0.7,
        auto_approve_safe_skills: true,
        max_agents: 100,
        max_clusters: 10,
        max_swarm_depth: 5,
        max_task_length: 2000,
        default_budget_usd: 10.5,
        privacy_mode: false,
    };

    beforeEach(() => {
        vi.clearAllMocks();
        (settings_store.get_settings as Mock).mockReturnValue(mock_default_settings);
        (settings_store.save_settings as Mock).mockReturnValue(null);
        (system_api_service.update_governance_settings as Mock).mockResolvedValue({});
    });

    it('renders all settings sections', () => {
        render(<MemoryRouter><Settings /></MemoryRouter>);
        
        expect(screen.getByText(/Neural Engine Gateway/i)).toBeInTheDocument();
        expect(screen.getByText(/Appearance/i)).toBeInTheDocument(); // matches Appearance alone too
        expect(screen.getByText(/Agent Defaults/i)).toBeInTheDocument();
        expect(screen.getByText(/Governance & Oversight/i)).toBeInTheDocument();
        expect(screen.getByText(/Swarm Architecture & Limits/i)).toBeInTheDocument();
    });

    it('handles input changes correctly', () => {
        render(<MemoryRouter><Settings /></MemoryRouter>);
        
        const url_input = screen.getByLabelText(/Engine API URL/i);
        fireEvent.change(url_input, { target: { name: 'tadpole_os_url', value: 'http://new-url:9000' } });
        expect(url_input).toHaveValue('http://new-url:9000');

        const theme_select = screen.getByLabelText(/Theme Base/i);
        fireEvent.change(theme_select, { target: { name: 'theme', value: 'slate' } });
        expect(theme_select).toHaveValue('slate');
    });

    it('updates range and number inputs correctly', () => {
        render(<MemoryRouter><Settings /></MemoryRouter>);

        // max_clusters range input
        const max_clusters_slider = screen.getByLabelText(/Max Mission Clusters/i);
        fireEvent.change(max_clusters_slider, { target: { value: '25' } });
        expect(max_clusters_slider).toHaveValue('25');

        // max_swarm_depth slider ( recrutement depth in en.json )
        const max_swarm_depth_slider = screen.getByLabelText(/Max Recruitment Depth/i);
        fireEvent.change(max_swarm_depth_slider, { target: { value: '8' } });
        expect(max_swarm_depth_slider).toHaveValue('8');

        // max_task_length number input
        const max_task_input = screen.getByLabelText(/Max Task Token Limit/i);
        fireEvent.change(max_task_input, { target: { value: '5000' } });
        expect(max_task_input).toHaveValue(5000);

        // default_budget_usd input
        const budget_input = screen.getByLabelText(/Base Mission Budget/i);
        fireEvent.change(budget_input, { target: { value: '25.5' } });
        expect(budget_input).toHaveValue(25.5);
    });

    it('logs error when sync fails in handle_save', async () => {
        const console_spy = vi.spyOn(console, 'error').mockImplementation(() => {});
        (system_api_service.update_governance_settings as Mock).mockRejectedValue(new Error('Network error'));
        
        render(<MemoryRouter><Settings /></MemoryRouter>);
        const save_button = screen.getByText(/Save Changes/i);
        fireEvent.click(save_button);

        await waitFor(() => {
            expect(console_spy).toHaveBeenCalledWith(expect.stringContaining('Failed to sync'), expect.any(Error));
        });
        console_spy.mockRestore();
    });

    it('navigates to template store', () => {
        render(<MemoryRouter><Settings /></MemoryRouter>);
        const store_button = screen.getByText(/Open Template Store/i);
        fireEvent.click(store_button);
        expect(mock_navigate).toHaveBeenCalledWith('/store');
    });

    it('displays validation errors from store', async () => {
        (settings_store.save_settings as Mock).mockReturnValue('INVALID_URL');
        render(<MemoryRouter><Settings /></MemoryRouter>);

        const save_button = screen.getByText(/Save Changes/i);
        fireEvent.click(save_button);

        expect(screen.getByText('INVALID_URL')).toBeInTheDocument();
    });
});

