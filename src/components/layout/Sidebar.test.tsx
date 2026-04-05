/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Manages the interface for Sidebar.test. 
 * Part of the Tadpole-OS Modular UI ecosystem.
 */

/**
 * @file Sidebar.test.tsx
 * @description Integration tests for the primary navigation sidebar.
 * @module Components/Layout/Sidebar
 * @testedBehavior
 * - Navigation Rendering: Ensures all core operational links are present.
 * - Sub-navigation Integration: Verifies Asset_Nav and Intelligence_Nav are correctly placed.
 * - Branding/Compliance: Validates logo and node certification display.
 * @aiContext
 * - Uses MemoryRouter to support NavLink components.
 * - Mocks sub-navigation components to isolate sidebar behavior.
 */
import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { Sidebar } from './Sidebar';

// Mock components used in Sidebar
vi.mock('../Intelligence_Nav', () => ({
    Intelligence_Nav: () => <div data-testid="intelligence-nav" />
}));

vi.mock('../Asset_Nav', () => ({
    Asset_Nav: () => <div data-testid="asset-nav" />
}));

vi.mock('../ui', () => ({
    Tooltip: ({ children, content }: any) => <div title={content}>{children}</div>
}));

describe('Sidebar Component', () => {
    const mockNavItemClass = ({ is_active }: { is_active: boolean }) => 
        is_active ? 'active-class' : 'inactive-class';

    it('renders navigation links and core sections', () => {
        render(
            <MemoryRouter>
                <Sidebar nav_item_class={mockNavItemClass} />
            </MemoryRouter>
        );

        expect(screen.getByText('Tadpole OS')).toBeInTheDocument();
        expect(screen.getByText('Core Ops')).toBeInTheDocument();
        expect(screen.getByText('Operations')).toBeInTheDocument();
        expect(screen.getByText('Hierarchy')).toBeInTheDocument();
        expect(screen.getByText('Missions')).toBeInTheDocument();
        expect(screen.getByText('Scheduled Jobs')).toBeInTheDocument();
        expect(screen.getByText('Oversight')).toBeInTheDocument();
        expect(screen.getByText('System Docs')).toBeInTheDocument();
        expect(screen.getByText('Settings')).toBeInTheDocument();
    });

    it('renders sub-navigation components', () => {
        render(
            <MemoryRouter>
                <Sidebar nav_item_class={mockNavItemClass} />
            </MemoryRouter>
        );

        expect(screen.getByTestId('intelligence-nav')).toBeInTheDocument();
        expect(screen.getByTestId('asset-nav')).toBeInTheDocument();
    });

    it('displays the certification badge', () => {
        render(
            <MemoryRouter>
                <Sidebar nav_item_class={mockNavItemClass} />
            </MemoryRouter>
        );

        expect(screen.getByText('Node Certified')).toBeInTheDocument();
        expect(screen.getByText('Sovereign Cluster v2.4')).toBeInTheDocument();
    });
});

