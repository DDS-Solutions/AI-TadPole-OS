/**
 * @file agentService.test.ts
 * @description Suite for agent-specific data transformation and normalization logic.
 * @module Services/AgentService
 * @testedBehavior
 * - Data Integrity: Normalization of inconsistent backend ID types (number vs string).
 * - Payload Safety: Ensuring numeric strings and pure strings are handled predictably.
 * @aiContext
 * - Primarily tests the pure function normalizeAgent to prevent runtime ID mismatch errors.
 */
import { describe, it, expect } from 'vitest';
import { normalizeAgent } from './agentService';
import type { RawAgent } from './agentService';

describe('normalizeAgent', () => {
    it('should always convert numeric IDs to strings', () => {
        const rawAgent = {
            id: 123, // numeric ID from hypothetical backend bug
            name: 'Test Agent',
            role: 'Tester',
            status: 'idle'
        } as unknown as RawAgent;

        const normalized = normalizeAgent(rawAgent);
        expect(normalized.id).toBe('123');
        expect(typeof normalized.id).toBe('string');
    });

    it('should keep string IDs as strings', () => {
        const rawAgent: RawAgent = {
            id: 'agent-456',
            name: 'String Agent',
            role: 'Dev',
            status: 'active'
        };

        const normalized = normalizeAgent(rawAgent);
        expect(normalized.id).toBe('agent-456');
    });

    it('should handle numeric strings correctly', () => {
        const rawAgent: RawAgent = {
            id: '789',
            name: 'Numeric String Agent',
            role: 'Analyst',
            status: 'idle'
        };

        const normalized = normalizeAgent(rawAgent);
        expect(normalized.id).toBe('789');
    });
});
