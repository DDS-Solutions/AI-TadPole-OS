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

import { APP_ROUTES, get_route_by_path } from './routes';
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

describe('get_route_by_path route resolution and normalization', () => {
    it('resolves root "/" to "/dashboard"', () => {
        const route = get_route_by_path('/');
        expect(route.path).toBe('/dashboard');
    });

    it('resolves exact registered routes correctly', () => {
        const settings_route = get_route_by_path('/settings');
        expect(settings_route.path).toBe('/settings');

        const org_route = get_route_by_path('/org-chart');
        expect(org_route.path).toBe('/org-chart');
    });

    it('strips query parameters when resolving routes', () => {
        const route_with_query = get_route_by_path('/settings?tab=vault&view=grid');
        expect(route_with_query.path).toBe('/settings');
    });

    it('strips hash fragments when resolving routes', () => {
        const route_with_hash = get_route_by_path('/missions#cluster-alpha');
        expect(route_with_hash.path).toBe('/missions');
    });

    it('strips trailing slashes from path', () => {
        const route_trailing = get_route_by_path('/engine/');
        expect(route_trailing.path).toBe('/engine');
    });

    it('handles query parameters with trailing slash', () => {
        const route = get_route_by_path('/models/?filter=local');
        expect(route.path).toBe('/models');
    });

    it('falls back to default dashboard route for empty string or unknown routes', () => {
        const empty_route = get_route_by_path('');
        expect(empty_route.path).toBe('/dashboard');

        const unknown_route = get_route_by_path('/non-existent-route-xyz');
        expect(unknown_route.path).toBe('/dashboard');
    });

    it('resolves route aliases like /bench to /benchmarks', () => {
        const bench_route = get_route_by_path('/bench');
        expect(bench_route.path).toBe('/benchmarks');

        const bench_with_query = get_route_by_path('/bench?tab=history');
        expect(bench_with_query.path).toBe('/benchmarks');
    });
});

