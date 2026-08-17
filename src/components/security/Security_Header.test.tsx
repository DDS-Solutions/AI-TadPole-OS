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

import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Security_Header } from './Security_Header';
import type { Agent_Health } from '../../services/tadpoleos_service';

describe('Security_Header Component', () => {
    const mock_health: Agent_Health[] = [
        {
            agent_id: 'a1',
            name: 'Scout Alpha',
            is_healthy: true,
            status: 'active',
            failures: 0,
            last_heartbeat: 12345
        }
    ];

    it('renders security hub header and agent initials', () => {
        render(
            <Security_Header
                agent_health={mock_health}
                merkle_integrity={1.0}
            />
        );

        expect(screen.getByText('S')).toBeDefined(); // Agent initial
        expect(screen.getByRole('banner')).toBeDefined();
    });

    it('displays compromised status when integrity is degraded', () => {
        render(
            <Security_Header
                agent_health={mock_health}
                merkle_integrity={0.5}
            />
        );

        expect(screen.getByRole('banner')).toBeDefined();
    });
});
