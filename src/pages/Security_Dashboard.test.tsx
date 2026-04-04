/**
 * @file Security_Dashboard.test.tsx
 * @description Suite for the Oversight & Compliance (Security) dashboard.
 * @module Pages/Security_Dashboard
 * @testedBehavior
 * - Compliance Monitoring: Rendering of spent vs efficiency security quotas.
 * - Audit Trail: Verification of skill authorization records (approved/rejected).
 * - Node Health: Real-time monitoring of agent health and throttling states.
 * @aiContext
 * - Mocks tadpole_os_service security and audit endpoints.
 * - Stubs global.setInterval to prevent timer-based memory leaks during tests.
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
    }
}));

describe('Security_Dashboard Page', () => {
    const mockQuotas = {
        totalSpent: 15.50,
        efficiency: 45.2,
        systemDefense: {
            merkleIntegrity: 1.0,
            memoryPressure: 0.2,
            sandboxStatus: 'ACTIVE',
            sandboxType: 'NSJAIL'
        }
    };

    const mockAuditTrail = {
        data: [
            { id: 'aud-1', decidedAt: new Date().toISOString(), agent_id: 'agent-alpha', skill: 'file_read', decision: 'approved', isVerified: true },
            { id: 'aud-2', decidedAt: null, agent_id: 'agent-beta', skill: 'network_access', decision: 'rejected', isVerified: true }
        ]
    };

    const mockAgentHealth = {
        agents: [
            { agent_id: 'agent-alpha', name: 'Alpha Agent', isHealthy: true, status: 'active', failureCount: 0, isThrottled: false },
            { agent_id: 'agent-beta', name: 'Beta Agent', isHealthy: false, status: 'idle', failureCount: 3, isThrottled: true }
        ]
    };

    beforeEach(() => {
        vi.clearAllMocks();
        vi.stubGlobal('setInterval', vi.fn());
        (tadpole_os_service.get_security_quotas as any).mockResolvedValue(mockQuotas);
        (tadpole_os_service.get_audit_trail as any).mockResolvedValue(mockAuditTrail);
        (tadpole_os_service.get_agent_health as any).mockResolvedValue(mockAgentHealth);
    });

    it('renders loading state initially', () => {
        (tadpole_os_service.get_security_quotas as any).mockReturnValue(new Promise(() => {}));
        render(<Security_Dashboard />);
        expect(screen.getByText(/Decrypting Security Ledger.../i)).toBeInTheDocument();
    });

    it('renders dashboard with security data', async () => {
        render(<Security_Dashboard />);

        await waitFor(() => {
            expect(screen.getByText(/Oversight & Compliance Dashboard/i)).toBeInTheDocument();
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
        const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
        (tadpole_os_service.get_security_quotas as any).mockRejectedValue(new Error('API Down'));
        
        render(<Security_Dashboard />);

        await waitFor(() => {
            expect(consoleErrorSpy).toHaveBeenCalled();
        });
        consoleErrorSpy.mockRestore();
    });
});
