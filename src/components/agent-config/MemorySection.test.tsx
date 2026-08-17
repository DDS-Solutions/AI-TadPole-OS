/**
 * @docs ARCHITECTURE:Testing
 *
 * ### AI Assist Note
 * Regression coverage for the adjacent production module and its public contracts.
 *
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Contract, rendering, state transition, or error-handling regression.
 * - **Trace Scope**: Vitest assertions and test-local mocks.
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { MemorySection } from './MemorySection';

describe('MemorySection Component', () => {
    const mock_memories = [
        { id: 'mem-1', content: 'Agent remembered deployment workflow.' }
    ];

    it('renders memory list and allows input editing', () => {
        const on_input_mock = vi.fn();
        render(
            <MemorySection
                memories={mock_memories}
                connectorConfigs={[]}
                isLoading={false}
                memoryInput="New neural memory"
                themeColor="#10b981"
                onMemoryInputChange={on_input_mock}
                onSaveMemory={vi.fn()}
                onDeleteMemory={vi.fn()}
                onRefresh={vi.fn()}
                onAddConnector={vi.fn()}
                onRemoveConnector={vi.fn()}
            />
        );

        expect(screen.getByText('Agent remembered deployment workflow.')).toBeDefined();
        expect(screen.getByDisplayValue('New neural memory')).toBeDefined();
    });
});
