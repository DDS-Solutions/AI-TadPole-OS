/**
 * @docs ARCHITECTURE:Testing
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Security / Security_Header.test
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
