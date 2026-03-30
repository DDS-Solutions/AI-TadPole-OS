/**
 * @file Settings.test.tsx
 * @description Suite for the System Configuration (Settings) page.
 * @module Pages/Settings
 * @testedBehavior
 * - Preference Management: Theme and density attribute synchronization.
 * - API Persistence: Verification of save calls to settingsStore.
 * - Reactive UI: Theme attribute injection into document.documentElement.
 * @aiContext
 * - Mocks settingsStore to intercept configuration I/O.
 */
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import Settings from './Settings';
import * as settingsStore from '../stores/settingsStore';

// Mock dependencies
vi.mock('../stores/settingsStore', () => ({
    getSettings: vi.fn(),
    saveSettings: vi.fn(),
}));

const mockNavigate = vi.fn();
vi.mock('react-router-dom', async () => {
    const actual = await vi.importActual('react-router-dom');
    return {
        ...actual as any,
        useNavigate: () => mockNavigate,
    };
});

// Mock global fetch
const mockFetch = vi.fn();
vi.stubGlobal('fetch', mockFetch);

describe('Settings Page', () => {
    const mockDefaultSettings = {
        TadpoleOSUrl: 'http://localhost:8000',
        TadpoleOSApiKey: 'test-key',
        theme: 'zinc',
        density: 'compact',
        defaultModel: 'GPT-4o',
        defaultTemperature: 0.7,
        autoApproveSafeSkills: true,
        maxAgents: 100,
        maxClusters: 10,
        maxSwarmDepth: 5,
        maxTaskLength: 2000,
        defaultBudgetUsd: 10.0
    };

    beforeEach(() => {
        vi.clearAllMocks();
        (settingsStore.getSettings as Mock).mockReturnValue({ ...mockDefaultSettings });
        mockFetch.mockResolvedValue({ ok: true, text: () => Promise.resolve('{}') });
    });

    it('loads settings from the store on mount', async () => {
        render(<MemoryRouter><Settings /></MemoryRouter>);

        expect(settingsStore.getSettings).toHaveBeenCalled();

        await waitFor(() => {
            expect(screen.getByPlaceholderText('http://localhost:8000')).toHaveValue('http://localhost:8000');
            expect(screen.getByPlaceholderText('Enter your NEURAL_TOKEN...')).toHaveValue('test-key');
        });
    });

    it('updates local state when changing form fields', async () => {
        render(<MemoryRouter><Settings /></MemoryRouter>);

        const urlInput = screen.getByLabelText(/Engine API URL/i);
        fireEvent.change(urlInput, { target: { value: 'http://new-url:9000', name: 'TadpoleOSUrl' } });

        expect(urlInput).toHaveValue('http://new-url:9000');

        const themeSelect = screen.getByLabelText(/Theme Base/i);
        fireEvent.change(themeSelect, { target: { value: 'slate', name: 'theme' } });
        expect(themeSelect).toHaveValue('slate');
    });

    it('calls saveSettings and syncs with backend on save', async () => {
        render(<MemoryRouter><Settings /></MemoryRouter>);

        // Change theme and density to non-default values
        const themeSelect = screen.getByLabelText(/Theme Base/i);
        fireEvent.change(themeSelect, { target: { value: 'slate', name: 'theme' } });
        
        const densitySelect = screen.getByLabelText(/Information Density/i);
        fireEvent.change(densitySelect, { target: { value: 'comfortable', name: 'density' } });

        const saveButton = screen.getByText(/Save Changes/i);
        fireEvent.click(saveButton);

        expect(settingsStore.saveSettings).toHaveBeenCalledWith(expect.objectContaining({
            theme: 'slate',
            density: 'comfortable'
        }));

        // Verify global side-effects after state settle
        await waitFor(() => {
            expect(document.documentElement.getAttribute('data-theme')).toBe('slate');
            expect(document.documentElement.getAttribute('data-density')).toBe('comfortable');
            expect(screen.getByText('Saved!')).toBeInTheDocument();
        });

        // Verify fetch call to oversight settings
        expect(mockFetch).toHaveBeenCalledWith(
            expect.stringContaining('http://localhost:8000/v1/oversight/settings'),
            expect.objectContaining({
                method: 'PUT',
                body: expect.stringContaining('"autoApproveSafeSkills":true')
            })
        );
    });

    it('updates numeric settings via handleNumericChange', async () => {
        render(<MemoryRouter><Settings /></MemoryRouter>);

        // maxAgents slider
        const maxAgentsSlider = screen.getByLabelText(/Max Active Agents/i);
        fireEvent.change(maxAgentsSlider, { target: { value: '150' } });
        expect(maxAgentsSlider).toHaveValue('150');

        // maxClusters slider
        const maxClustersSlider = screen.getByLabelText(/Max Mission Clusters/i);
        fireEvent.change(maxClustersSlider, { target: { value: '25' } });
        expect(maxClustersSlider).toHaveValue('25');

        // maxSwarmDepth slider
        const maxSwarmDepthSlider = screen.getByLabelText(/Max Recruitment Depth/i);
        fireEvent.change(maxSwarmDepthSlider, { target: { value: '8' } });
        expect(maxSwarmDepthSlider).toHaveValue('8');

        // maxTaskLength number input
        const maxTaskInput = screen.getByLabelText(/Max Task Token Limit/i);
        fireEvent.change(maxTaskInput, { target: { value: '5000' } });
        expect(maxTaskInput).toHaveValue(5000);

        // defaultBudgetUsd input
        const budgetInput = screen.getByLabelText(/Base Mission Budget/i);
        fireEvent.change(budgetInput, { target: { value: '25.5' } });
        expect(budgetInput).toHaveValue(25.5);
    });

    it('logs error when fetch fails in handleSave', async () => {
        const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
        mockFetch.mockResolvedValueOnce({
            ok: false,
            status: 500,
            statusText: 'Internal Server Error',
            text: () => Promise.resolve('sync failed')
        });
        
        render(<MemoryRouter><Settings /></MemoryRouter>);
        const saveButton = screen.getByText(/Save Changes/i);
        fireEvent.click(saveButton);

        await waitFor(() => {
            expect(consoleSpy).toHaveBeenCalledWith(expect.stringContaining('Failed to sync'), expect.any(Error));
        });
        consoleSpy.mockRestore();
    });

    it('navigates to template store', () => {
        render(<MemoryRouter><Settings /></MemoryRouter>);
        const storeButton = screen.getByText(/Open Template Store/i);
        fireEvent.click(storeButton);
        expect(mockNavigate).toHaveBeenCalledWith('/store');
    });

    it('displays validation errors from store', async () => {
        (settingsStore.saveSettings as Mock).mockReturnValue('INVALID_URL');
        render(<MemoryRouter><Settings /></MemoryRouter>);

        const saveButton = screen.getByText(/Save Changes/i);
        fireEvent.click(saveButton);

        expect(screen.getByText('Fix Errors')).toBeInTheDocument();
        expect(screen.getByText('INVALID_URL')).toBeInTheDocument();
    });
});
