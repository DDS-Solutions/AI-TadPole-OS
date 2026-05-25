/**
 * @docs ARCHITECTURE:Quality:Verification
 * 
 * ### AI Assist Note
 * **@docs ARCHITECTURE:TestSuites**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[routes_navigation_test]` in observability traces.
 */

/**
 * @docs ARCHITECTURE:TestSuites
 *
 * Verifies every registered application route is reachable from the primary
 * navigation shell. This catches route/menu drift before a page becomes
 * effectively hidden.
 */

import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { APP_ROUTES } from './routes';
import { Sidebar } from '../components/layout/Sidebar';

describe('Route navigation connectivity', () => {
    it('exposes every APP_ROUTES path in the sidebar navigation', () => {
        render(
            <MemoryRouter>
                <Sidebar nav_item_class={() => 'nav-item'} />
            </MemoryRouter>
        );

        const sidebar_paths = new Set(
            screen.getAllByRole('link')
                .map(link => link.getAttribute('href'))
                .filter((href): href is string => Boolean(href))
        );

        const missing_paths = APP_ROUTES
            .map(route => route.path)
            .filter(path => !sidebar_paths.has(path));

        expect(missing_paths).toEqual([]);
    });
});


// Metadata: [routes_navigation_test]
