/**
 * @file SecurityDashboard.test.tsx
 * @description Suite for the Oversight & Compliance (Security) dashboard.
 * @module Pages/SecurityDashboard
 * @testedBehavior
 * - Compliance Monitoring: Rendering of spent vs efficiency security quotas.
 * - Audit Trail: Verification of skill authorization records (approved/rejected).
 * - Node Health: Real-time monitoring of agent health and throttling states.
 * @aiContext
 * - Mocks TadpoleOSService security and audit endpoints.
 * - Stubs global.setInterval to prevent timer-based memory leaks during tests.
 */
import { render, screen, waitFor, within } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import SecurityDashboard from './SecurityDashboard';
import { TadpoleOSService } from '../services/tadpoleosService';

// Mock TadpoleOSService
vi.mock('../services/tadpoleosService', () => ({
    TadpoleOSService: {
        getSecurityQuotas: vi.fn(),
        getAuditTrail: vi.fn(),
        getAgentHealth: vi.fn(),
    }
}));

describe('SecurityDashboard Page', () => {
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
            { id: 'aud-1', decidedAt: new Date().toISOString(), agentId: 'agent-alpha', skill: 'file_read', decision: 'approved', isVerified: true },
            { id: 'aud-2', decidedAt: null, agentId: 'agent-beta', skill: 'network_access', decision: 'rejected', isVerified: true }
        ]
    };

    const mockAgentHealth = {
        agents: [
            { agentId: 'agent-alpha', name: 'Alpha Agent', isHealthy: true, status: 'active', failureCount: 0, isThrottled: false },
            { agentId: 'agent-beta', name: 'Beta Agent', isHealthy: false, status: 'idle', failureCount: 3, isThrottled: true }
        ]
    };

    beforeEach(() => {
        vi.clearAllMocks();
        vi.stubGlobal('setInterval', vi.fn());
        (TadpoleOSService.getSecurityQuotas as any).mockResolvedValue(mockQuotas);
        (TadpoleOSService.getAuditTrail as any).mockResolvedValue(mockAuditTrail);
        (TadpoleOSService.getAgentHealth as any).mockResolvedValue(mockAgentHealth);
    });

    it('renders loading state initially', () => {
        (TadpoleOSService.getSecurityQuotas as any).mockReturnValue(new Promise(() => {}));
        render(<SecurityDashboard />);
        expect(screen.getByText(/Decrypting Security Ledger.../i)).toBeInTheDocument();
    });

    it('renders dashboard with security data', async () => {
        render(<SecurityDashboard />);

        await waitFor(() => {
            expect(screen.getByText(/Oversight & Compliance Dashboard/i)).toBeInTheDocument();
        });

        expect(screen.getByText('$15.50')).toBeInTheDocument();
        
        // Check health list
        expect(await screen.findByText('Alpha Agent')).toBeInTheDocument();
        expect(await screen.findByText('Beta Agent')).toBeInTheDocument();
    });

    it('displays audit trail entries correctly', async () => {
        render(<SecurityDashboard />);

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
        (TadpoleOSService.getSecurityQuotas as any).mockRejectedValue(new Error('API Down'));
        
        render(<SecurityDashboard />);

        await waitFor(() => {
            expect(consoleErrorSpy).toHaveBeenCalled();
        });
        consoleErrorSpy.mockRestore();
    });
});
