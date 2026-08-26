/**
 * @docs ARCHITECTURE:Testing
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Security / Defense_Matrix_Section.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Defense_Matrix_Section } from './Defense_Matrix_Section';

describe('Defense_Matrix_Section Component', () => {
    it('renders memory pressure and capability metrics', () => {
        render(
            <Defense_Matrix_Section
                system_defense={{
                    memory_pressure: 0.45,
                    sandbox_violations: 0,
                    active_firewalls: 4
                } as any}
            />
        );

        expect(screen.getByText(/Defense Matrix|Defense/i)).toBeDefined();
        expect(screen.getByText(/45.0%/)).toBeDefined();
        expect(screen.getByText('NOMINAL')).toBeDefined();
    });

    it('displays high pressure warning when memory exceeds 80%', () => {
        render(
            <Defense_Matrix_Section
                system_defense={{
                    memory_pressure: 0.90,
                    sandbox_violations: 2,
                    active_firewalls: 4
                } as any}
            />
        );

        expect(screen.getByText(/90.0%/)).toBeDefined();
        expect(screen.getByText('PRESSURE_HIGH')).toBeDefined();
    });
});
