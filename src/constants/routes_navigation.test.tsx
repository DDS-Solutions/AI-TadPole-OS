/**
 * @docs ARCHITECTURE:Quality:Verification
 *
 * ### AI Context Alignment
 * - **Subsystem**: System Core / routes_navigation.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';

vi.unmock('../components/layout/Sidebar');
vi.unmock('../components/Intelligence_Nav');
vi.unmock('../components/Asset_Nav');

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
