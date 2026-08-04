/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Verification of the Neural Map page components.**
 * Verifies that the Neural Map page and the custom KnowledgeGraph layout render properly.
 * Mocks the complex force graph canvas layer to isolate page container testing.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Rendering failure if mock dependencies are incomplete or React Router context is missing.
 * - **Telemetry Link**: Search `[Neural_Map.test]` in tracing logs.
 */

import React from 'react';
import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import Neural_Map from './Neural_Map';

// Mock the KnowledgeGraph component to isolate the page test from complex canvas rendering
vi.mock('../components/intelligence/KnowledgeGraph', () => ({
    KnowledgeGraph: () => <div data-testid="mock-knowledge-graph">Mocked Knowledge Graph</div>
}));

describe('Neural_Map Page', () => {
    it('renders the header, title, and description correctly', () => {
        render(
            <MemoryRouter>
                <Neural_Map />
            </MemoryRouter>
        );

        // Verify title is rendered
        expect(screen.getByText(/Neural Map & Knowledge Graph/i)).toBeInTheDocument();

        // Verify descriptive paragraph is rendered
        expect(screen.getByText(/Interactive topology graph illustrating the functional hierarchy/i)).toBeInTheDocument();
        expect(screen.getByText(/Select nodes to trace downstream dependencies/i)).toBeInTheDocument();
    });

    it('mounts the mocked KnowledgeGraph component inside the layout', () => {
        render(
            <MemoryRouter>
                <Neural_Map />
            </MemoryRouter>
        );

        // Verify that the KnowledgeGraph placeholder is present in the document
        expect(screen.getByTestId('mock-knowledge-graph')).toBeInTheDocument();
        expect(screen.getByText('Mocked Knowledge Graph')).toBeInTheDocument();
    });
});

// Metadata: [Neural_Map_test]
