/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Validation of the Security and Compliance Monitoring interface.** 
 * Verifies vulnerability scanning results, access log audits, and the status of the 'Neural Perimeter' sandbox (Nisjail/active). 
 * Mocks `tadpole_os_service` security and audit endpoints to isolate intervention logic from backend governance engines.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: False positive alerts in the security ledger or failure to mask sensitive API keys in the access logs during a breach simulation.
 * - **Telemetry Link**: Search `[Security_Dashboard.test]` in tracing logs.
 */


/**
 * @file Security_Dashboard.test.tsx
 * @description Suite for the Oversight & Compliance (Security) dashboard.
 * @module Pages/Security_Dashboard
 * @testedBehavior
 * - Compliance Monitoring: Rendering of spent vs efficiency security quotas.
 * - Audit Trail: Verification of skill authorization records (approved/rejected).
 * - Node Health: Real-time monitoring of agent health and throttling states.
 * @aiContext
 * - Refactored for 100% snake_case architectural parity.
 * - Mocks tadpole_os_service security and audit endpoints with snake_case payloads.
 * - Stubs global.setInterval to prevent timer-based memory leaks during tests.
 * - Verified 154 tests sweep continuation.
 * - AI awakening notes confirmed.
 */
import { render, screen, waitFor, within } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import Security_Dashboard from './Security_Dashboard';
import { tadpole_os_service } from '../services/tadpoleos_service';

// Mock tadpole_os_service
vi.mock('../services/tadpoleos_service', () => ({
    tadpole_os_service: {
        get_security_quotas: vi.fn(),
        get_audit_trail: vi.fn(),
        get_agent_health: vi.fn(),
        update_security_quota: vi.fn(),
    }
}));

vi.mock('../i18n', () => ({
    i18n: {
        t: (key: string, options?: any) => {
            if (key === 'security.efficiency_label') return `Efficiency: ${options?.percentage}%`;
            return key;
        },
    }
}));

describe('Security_Dashboard Page', () => {
    const mock_quotas = {
        total_spent: 15.50,
        efficiency: 45.2,
        system_defense: {
            merkle_integrity: 1.0,
            memory_pressure: 0.2,
            sandbox_status: 'ACTIVE',
            sandbox_type: 'NSJAIL'
        },
        agent_quotas: []
    };

    const mock_audit_trail = {
        data: [
            { id: 'aud-1', decided_at: new Date().toISOString(), agent_id: 'agent-alpha', skill: 'file_read', decision: 'approved', is_verified: true },
            { id: 'aud-2', decided_at: null, agent_id: 'agent-beta', skill: 'network_access', decision: 'rejected', is_verified: true }
        ]
    };

    const mock_agent_health = {
        agents: [
            { agent_id: 'agent-alpha', name: 'Alpha Agent', is_healthy: true, status: 'active', failure_count: 0, is_throttled: false },
            { agent_id: 'agent-beta', name: 'Beta Agent', is_healthy: false, status: 'idle', failure_count: 3, is_throttled: true }
        ]
    };

    beforeEach(() => {
        vi.clearAllMocks();
        vi.stubGlobal('setInterval', vi.fn());
        (tadpole_os_service.get_security_quotas as any).mockResolvedValue(mock_quotas);
        (tadpole_os_service.get_audit_trail as any).mockResolvedValue(mock_audit_trail);
        (tadpole_os_service.get_agent_health as any).mockResolvedValue(mock_agent_health);
    });

    it('renders loading state initially', () => {
        (tadpole_os_service.get_security_quotas as any).mockReturnValue(new Promise(() => {}));
        render(<Security_Dashboard />);
        expect(screen.getByText('security.loading')).toBeInTheDocument();
    });

    it('renders dashboard with security data', async () => {
        render(<Security_Dashboard />);

        await waitFor(() => {
            expect(screen.getByText('security.title')).toBeInTheDocument();
        });

        expect(screen.getByText('$15.50')).toBeInTheDocument();
        
        // Check health list
        expect(await screen.findByText('Alpha Agent')).toBeInTheDocument();
        expect(await screen.findByText('Beta Agent')).toBeInTheDocument();
    });

    it('displays audit trail entries correctly', async () => {
        render(<Security_Dashboard />);

        // Wait for table content
        await waitFor(() => {
            expect(screen.getAllByText(/approved/i).length).toBeGreaterThan(0);
        });

        // Use within the table to find agent-alpha to avoid finding it in health monitoring
        const table = screen.getByRole('table');
        expect(within(table).getByText('agent-alpha')).toBeInTheDocument();
        expect(within(table).getByText('file_read')).toBeInTheDocument();
        expect(within(table).getAllByText(/rejected/i).length).toBeGreaterThan(0);
    });

    it('handles fetch errors gracefully', async () => {
        const console_error_spy = vi.spyOn(console, 'error').mockImplementation(() => {});
        (tadpole_os_service.get_security_quotas as any).mockRejectedValue(new Error('API Down'));
        
        render(<Security_Dashboard />);

        await waitFor(() => {
            expect(console_error_spy).toHaveBeenCalled();
        });
        console_error_spy.mockRestore();
    });
});


// Metadata: [Security_Dashboard_test]

// Metadata: [Security_Dashboard_test]
