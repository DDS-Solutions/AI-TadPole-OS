/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: Domain Model / Agent Hierarchy Tests
 * - **Primary Entrypoints**: `agent_hierarchy.test`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Exhaustively tests hierarchy precedence logic across standard and edge-case swarm sets.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { describe, it, expect } from 'vitest';
import { resolve_alpha, resolve_nexus } from './agent_hierarchy';

describe('agent_hierarchy', () => {
    describe('resolve_alpha', () => {
        it('returns undefined for empty agent lists', () => {
            expect(resolve_alpha([])).toBeUndefined();
        });

        it('resolves CEO role by precedence', () => {
            const agents = [
                { id: '10', name: 'Worker', role: 'Engineer' },
                { id: '20', name: 'Leader', role: 'CEO' },
                { id: '1', name: 'Root', role: 'Observer' },
            ];
            expect(resolve_alpha(agents)?.id).toBe('20');
        });

        it('resolves case-insensitive CEO or Agent of Nine name match', () => {
            const agents = [
                { id: '10', name: 'Agent of Nine', role: 'Overseer' },
                { id: '1', name: 'Sub', role: 'Dev' },
            ];
            expect(resolve_alpha(agents)?.id).toBe('10');
        });

        it('falls back to agent with ID "1" when no CEO or Nine is present', () => {
            const agents = [
                { id: '5', name: 'Worker A', role: 'Engineer' },
                { id: '1', name: 'Lead 1', role: 'Staff' },
                { id: '2', name: 'Worker B', role: 'Engineer' },
            ];
            expect(resolve_alpha(agents)?.id).toBe('1');
        });

        it('falls back to the first agent when neither CEO, Nine, nor ID "1" exists', () => {
            const agents = [
                { id: 'alpha-custom', name: 'Custom Root', role: 'Specialist' },
                { id: 'beta-custom', name: 'Custom Next', role: 'Analyst' },
            ];
            expect(resolve_alpha(agents)?.id).toBe('alpha-custom');
        });
    });

    describe('resolve_nexus', () => {
        it('returns undefined for empty agent lists', () => {
            expect(resolve_nexus([])).toBeUndefined();
        });

        it('resolves COO role distinct from alpha', () => {
            const alpha = { id: '1', name: 'Boss', role: 'CEO' };
            const agents = [
                alpha,
                { id: '2', name: 'Sub Boss', role: 'COO' },
                { id: '3', name: 'Engineer', role: 'Dev' },
            ];
            expect(resolve_nexus(agents, alpha)?.id).toBe('2');
        });

        it('resolves Tadpole Alpha or Tadpole name match', () => {
            const alpha = { id: '1', name: 'Agent of Nine', role: 'CEO' };
            const agents = [
                alpha,
                { id: '10', name: 'Tadpole Alpha', role: 'Operations' },
                { id: '2', name: 'Other', role: 'Worker' },
            ];
            expect(resolve_nexus(agents, alpha)?.id).toBe('10');
        });

        it('falls back to ID "2" distinct from alpha', () => {
            const alpha = { id: '1', name: 'Boss', role: 'CEO' };
            const agents = [
                alpha,
                { id: '2', name: 'Worker Two', role: 'Engineer' },
                { id: '3', name: 'Worker Three', role: 'Engineer' },
            ];
            expect(resolve_nexus(agents, alpha)?.id).toBe('2');
        });

        it('falls back to first available agent distinct from alpha', () => {
            const alpha = { id: 'custom-1', name: 'Solo', role: 'CEO' };
            const agents = [
                alpha,
                { id: 'custom-2', name: 'Next', role: 'Engineer' },
            ];
            expect(resolve_nexus(agents, alpha)?.id).toBe('custom-2');
        });
    });
});
