/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Hierarchy / HierarchyConnectionLines.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { describe, it, expect } from 'vitest';
import { render } from '@testing-library/react';
import { HierarchyConnectionLines } from '../HierarchyConnectionLines';

describe('HierarchyConnectionLines', () => {
    it('returns null when branchPath or xCoords is empty', () => {
        const { container: c1 } = render(
            <HierarchyConnectionLines branchPath="" xCoords={[500]} hasActiveChains={false} />
        );
        expect(c1.firstChild).toBeNull();

        const { container: c2 } = render(
            <HierarchyConnectionLines branchPath="M 500 0 V 20" xCoords={[]} hasActiveChains={false} />
        );
        expect(c2.firstChild).toBeNull();
    });

    it('renders SVG paths and calculates junction dot coordinates correctly', () => {
        const path = 'M 500 0 V 20 M 500 20 Q 500 30 490 30 H 303 Q 293 30 293 40 V 52 M 500 20 Q 500 30 510 30 H 697 Q 707 30 707 40 V 52';
        const coords = [293, 707];

        const { container } = render(
            <HierarchyConnectionLines branchPath={path} xCoords={coords} hasActiveChains={false} />
        );

        const svg = container.querySelector('svg');
        expect(svg).toBeDefined();

        const paths = container.querySelectorAll('path');
        expect(paths.length).toBe(1);
        expect(paths[0].getAttribute('d')).toBe(path);

        const circles = container.querySelectorAll('circle');
        expect(circles.length).toBe(2);
        expect(circles[0].getAttribute('cx')).toBe('293');
        expect(circles[0].getAttribute('cy')).toBe('30');
        expect(circles[1].getAttribute('cx')).toBe('707');
        expect(circles[1].getAttribute('cy')).toBe('30');
    });

    it('places center stem circle at cy=20 and renders active neural animation when active', () => {
        const path = 'M 500 0 V 20 M 500 20 V 52';
        const coords = [500];

        const { container } = render(
            <HierarchyConnectionLines branchPath={path} xCoords={coords} hasActiveChains={true} />
        );

        const paths = container.querySelectorAll('path');
        expect(paths.length).toBe(2); // Base path + animated gradient path

        const circles = container.querySelectorAll('circle');
        expect(circles.length).toBe(1);
        expect(circles[0].getAttribute('cx')).toBe('500');
        expect(circles[0].getAttribute('cy')).toBe('20');
    });
});
