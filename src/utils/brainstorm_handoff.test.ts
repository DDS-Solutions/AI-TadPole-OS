/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Utilities / brainstorm_handoff.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Verifies deterministic system filtering, turn windowing, and character bounding.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { describe, it, expect } from 'vitest';
import { build_brainstorm_context } from './brainstorm_handoff';

describe('build_brainstorm_context', () => {
    it('returns undefined when only system or blank entries exist', () => {
        expect(build_brainstorm_context([])).toBeUndefined();
        expect(build_brainstorm_context([
            { speaker: 'System', text: 'Online', is_system: true },
            { speaker: 'User', text: '   ' },
        ])).toBeUndefined();
    });

    it('formats conversational turns and excludes system banners', () => {
        const result = build_brainstorm_context([
            { speaker: 'System', text: 'Voice online', is_system: true },
            { speaker: 'User', text: 'Let us design   the billing refactor' },
            { speaker: 'Agent of Nine', text: 'Agreed, split into 3 phases.' },
        ]);

        expect(result).toContain('### Prior Brainstorm Context (Chat/Plan Session)');
        expect(result).not.toContain('Voice online');
        expect(result).toContain('User: Let us design the billing refactor');
        expect(result).toContain('Agent of Nine: Agreed, split into 3 phases.');
    });

    it('keeps only the most recent max_turns and caps total length', () => {
        const entries = Array.from({ length: 6 }, (_, i) => ({
            speaker: 'User',
            text: `Turn ${i + 1} ${'x'.repeat(60)}`,
        }));

        const bounded = build_brainstorm_context(entries, 3, 200);
        expect(bounded).toBeDefined();
        expect(bounded).not.toContain('Turn 1');
        expect(bounded).not.toContain('Turn 2');
        expect(bounded).not.toContain('Turn 3');
        expect(bounded).toContain('Turn 6');
        expect(bounded!.length).toBeLessThanOrEqual(200);
    });
});
