/**
 * @docs ARCHITECTURE:Services
 * 
 * ### AI Assist Note
 * **Test Suite**: Validates the Governance and swarm consensus operations. 
 */

/**
 * @file proposal_service.test.ts
 * @description Suite for the Swarm Intelligence Proposal Service.
 * @module Services/proposal_service
 * @testedBehavior
 * - Optimization Proposals: Generation of security, performance, and growth-oriented agent configuration updates.
 * - Intent Recognition: Matching cluster objectives to appropriate reasoning and model recommendations.
 * - Consistency: Verification of collaborator mapping and timestamp generation.
 * @aiContext
 * - Refactored for 100% snake_case architectural parity.
 * - Pure logic tests: does not require API mocks.
 * - Verified 154 tests sweep continuation.
 * - AI awakening notes confirmed.
 */
import { describe, it, expect } from 'vitest';
import { proposal_service } from './proposal_service';
import type { Mission_Cluster } from '../stores/workspace_store';

describe('proposal_service', () => {
    const base_cluster: Mission_Cluster = {
        id: 'cluster-1',
        name: 'Test Cluster',
        department: 'Engineering',
        path: '/test',
        budget_usd: 100,
        alpha_id: 'agent-1',
        collaborators: ['agent-1', 'agent-2'],
        pending_tasks: []
    } as any;

    it('returns null if cluster or objective is missing', () => {
        expect(proposal_service.generate_proposal(null as any)).toBeNull();
        expect(proposal_service.generate_proposal({ ...base_cluster, objective: '' } as any)).toBeNull();
    });

    it('generates security proposal for security-related objectives', () => {
        const cluster = { ...base_cluster, objective: 'Fix security vulnerability in the patch' };
        const proposal = proposal_service.generate_proposal(cluster as any);

        expect(proposal).not.toBeNull();
        expect(proposal?.reasoning).toContain('DEEP THREAT DETECTED');
        expect(proposal?.changes[0].proposed_role).toBe('Security Hardener');
        expect(proposal?.changes[0].proposed_model).toBe('DeepSeek V3.2');
        expect(proposal?.changes[0].added_skills).toContain('Scan Vulnerabilities');
    });

    it('generates performance proposal for optimization objectives', () => {
        const cluster = { ...base_cluster, objective: 'Optimize database scaling performance' };
        const proposal = proposal_service.generate_proposal(cluster as any);

        expect(proposal).not.toBeNull();
        expect(proposal?.reasoning).toContain('PERFORMANCE BOTTLENECK');
        expect(proposal?.changes[0].proposed_role).toBe('Performance Architect');
        expect(proposal?.changes[0].proposed_model).toBe('Claude Sonnet 4.5');
    });

    it('generates growth proposal for feature-centric objectives', () => {
        const cluster = { ...base_cluster, objective: 'Build new user feature expansion' };
        const proposal = proposal_service.generate_proposal(cluster as any);

        expect(proposal).not.toBeNull();
        expect(proposal?.reasoning).toContain('USER-CENTRIC EXPANSION');
        expect(proposal?.changes[0].proposed_role).toBe('Growth Catalyst');
        expect(proposal?.changes[0].proposed_model).toBe('GPT-5.2');
    });

    it('falls back to standard ops for unknown objectives', () => {
        const cluster = { ...base_cluster, objective: 'Clean the office' };
        const proposal = proposal_service.generate_proposal(cluster as any);

        expect(proposal).not.toBeNull();
        expect(proposal?.reasoning).toContain('STANDARD OPS');
        expect(proposal?.changes[0].added_skills).toContain('Deep Research');
        expect(proposal?.changes[0].proposed_role).toBeUndefined();
    });

    it('includes all collaborators in the proposal', () => {
        const cluster = { ...base_cluster, objective: 'test' };
        const proposal = proposal_service.generate_proposal(cluster as any);
        expect(proposal?.changes.length).toBe(2);
        expect(proposal?.changes.map(c => c.agent_id)).toContain('agent-1');
        expect(proposal?.changes.map(c => c.agent_id)).toContain('agent-2');
    });

    it('generates a valid timestamp', () => {
        const cluster = { ...base_cluster, objective: 'test' };
        const proposal = proposal_service.generate_proposal(cluster as any);
        expect(proposal?.timestamp).toBeLessThanOrEqual(Date.now());
    });
});

