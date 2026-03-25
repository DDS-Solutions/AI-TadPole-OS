import { describe, it, expect } from 'vitest';
import { ProposalService } from './ProposalService';
import type { MissionCluster } from '../stores/workspaceStore';

describe('ProposalService', () => {
    const baseCluster: MissionCluster = {
        id: 'cluster-1',
        name: 'Test Cluster',
        department: 'Engineering',
        path: '/test',
        budgetUsd: 100,
        alphaId: 'agent-1',
        collaborators: ['agent-1', 'agent-2'],
        pendingTasks: []
    };

    it('returns null if cluster or objective is missing', () => {
        expect(ProposalService.generateProposal(null as any)).toBeNull();
        expect(ProposalService.generateProposal({ ...baseCluster, objective: '' } as any)).toBeNull();
    });

    it('generates security proposal for security-related objectives', () => {
        const cluster = { ...baseCluster, objective: 'Fix security vulnerability in the patch' };
        const proposal = ProposalService.generateProposal(cluster);

        expect(proposal).not.toBeNull();
        expect(proposal?.reasoning).toContain('DEEP THREAT DETECTED');
        expect(proposal?.changes[0].proposedRole).toBe('Security Hardener');
        expect(proposal?.changes[0].proposedModel).toBe('DeepSeek V3.2');
        expect(proposal?.changes[0].addedSkills).toContain('Scan Vulnerabilities');
    });

    it('generates performance proposal for optimization objectives', () => {
        const cluster = { ...baseCluster, objective: 'Optimize database scaling performance' };
        const proposal = ProposalService.generateProposal(cluster);

        expect(proposal).not.toBeNull();
        expect(proposal?.reasoning).toContain('PERFORMANCE BOTTLENECK');
        expect(proposal?.changes[0].proposedRole).toBe('Performance Architect');
        expect(proposal?.changes[0].proposedModel).toBe('Claude Sonnet 4.5');
    });

    it('generates growth proposal for feature-centric objectives', () => {
        const cluster = { ...baseCluster, objective: 'Build new user feature expansion' };
        const proposal = ProposalService.generateProposal(cluster);

        expect(proposal).not.toBeNull();
        expect(proposal?.reasoning).toContain('USER-CENTRIC EXPANSION');
        expect(proposal?.changes[0].proposedRole).toBe('Growth Catalyst');
        expect(proposal?.changes[0].proposedModel).toBe('GPT-5.2');
    });

    it('falls back to standard ops for unknown objectives', () => {
        const cluster = { ...baseCluster, objective: 'Clean the office' };
        const proposal = ProposalService.generateProposal(cluster);

        expect(proposal).not.toBeNull();
        expect(proposal?.reasoning).toContain('STANDARD OPS');
        expect(proposal?.changes[0].addedSkills).toContain('Deep Research');
        expect(proposal?.changes[0].proposedRole).toBeUndefined();
    });

    it('includes all collaborators in the proposal', () => {
        const cluster = { ...baseCluster, objective: 'test' };
        const proposal = ProposalService.generateProposal(cluster);
        expect(proposal?.changes.length).toBe(2);
        expect(proposal?.changes.map(c => c.agentId)).toContain('agent-1');
        expect(proposal?.changes.map(c => c.agentId)).toContain('agent-2');
    });

    it('generates a valid timestamp', () => {
        const cluster = { ...baseCluster, objective: 'test' };
        const proposal = ProposalService.generateProposal(cluster);
        expect(proposal?.timestamp).toBeLessThanOrEqual(Date.now());
    });
});
