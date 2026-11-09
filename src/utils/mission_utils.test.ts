/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Utils / mission_utils.test
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { describe, it, expect } from 'vitest';
import { clean_mission_title } from './mission_utils';

describe('clean_mission_title', () => {
    it('strips injected Socratic envelope HTML comments', () => {
        const input = '<!-- SOCRATIC_GATE_ENVELOPE: PRE-CLEARED -->\n# Fix memory leak in LanceDB vector cache\nFurther instructions';
        expect(clean_mission_title(input)).toBe('Fix memory leak in LanceDB vector cache Further instructions');
    });

    it('handles truncated Socratic gate envelope comments from raw titles', () => {
        const input = '<!-- SOCRATIC_GATE_ENVELOPE: PRE-CLEARED --> ...';
        expect(clean_mission_title(input)).toBe('Untitled Sovereign Mission');
    });

    it('falls back to custom fallback if provided and title is empty/envelope-only', () => {
        const input = '<!-- SOCRATIC_GATE_ENVELOPE: PRE-CLEARED -->';
        expect(clean_mission_title(input, 'Untitled Mission')).toBe('Untitled Mission');
    });

    it('preserves clean titles without altering them', () => {
        expect(clean_mission_title('Autonomous Swarm Mission')).toBe('Autonomous Swarm Mission');
        expect(clean_mission_title('Audit Remediation Mission')).toBe('Audit Remediation Mission');
    });

    it('handles null, undefined, or empty strings gracefully', () => {
        expect(clean_mission_title(null)).toBe('Untitled Sovereign Mission');
        expect(clean_mission_title(undefined)).toBe('Untitled Sovereign Mission');
        expect(clean_mission_title('')).toBe('Untitled Sovereign Mission');
        expect(clean_mission_title('   ')).toBe('Untitled Sovereign Mission');
    });

    it('strips markdown heading characters from titles', () => {
        expect(clean_mission_title('### Refactor Mission Header')).toBe('Refactor Mission Header');
    });
});

