/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Pages / Detached_Swarm_Pulse.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import React from 'react';
import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import Detached_Swarm_Pulse from './Detached_Swarm_Pulse';

// Mock Swarm_Visualizer to avoid canvas/WebSocket initialization
vi.mock('../components/Swarm_Visualizer', () => ({
    Swarm_Visualizer: ({ is_detached }: { is_detached?: boolean }) => (
        <div data-testid="mock-swarm-visualizer" data-detached={is_detached ? 'true' : 'false'}>
            Mocked Swarm Visualizer
        </div>
    )
}));

// Mock i18n translations
vi.mock('../i18n', () => ({
    i18n: {
        t: (key: string) => key,
    }
}));

describe('Detached_Swarm_Pulse Page', () => {
    it('renders the screen-reader only header and mocks visualizer with detached layout', () => {
        render(<Detached_Swarm_Pulse />);

        // Verify the screen-reader-only header exists and uses key-based localization mock
        const header = screen.getByRole('heading', { level: 1, name: 'swarm_visualizer.detached_telemetry_h1' });
        expect(header).toBeInTheDocument();
        expect(header).toHaveClass('sr-only');

        // Verify that the Swarm_Visualizer mounts and receives is_detached={true}
        const visualizer = screen.getByTestId('mock-swarm-visualizer');
        expect(visualizer).toBeInTheDocument();
        expect(visualizer).toHaveAttribute('data-detached', 'true');
    });

    it('injects JSON-LD script metadata tag into the document', () => {
        const { container } = render(<Detached_Swarm_Pulse />);

        const scriptTag = container.querySelector('script[type="application/ld+json"]');
        expect(scriptTag).toBeInTheDocument();
        
        const metadata = JSON.parse(scriptTag!.textContent || '{}');
        expect(metadata).toEqual({
            "@context": "https://schema.org",
            "@type": "SoftwareApplication",
            "name": "Tadpole OS Swarm Pulse",
            "description": "swarm_visualizer.detached_description",
            "author": { "@type": "Organization", "name": "Sovereign Engineering" },
            "applicationCategory": "Telemetry Tool",
            "operatingSystem": "Tadpole OS"
        });
    });
});
