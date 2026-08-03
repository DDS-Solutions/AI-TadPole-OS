/**
 * @docs ARCHITECTURE:Quality:Verification
 * 
 * ### AI Assist Note
 * **Verification and quality assurance for the Tadpole OS engine.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[anomalyParser_test]` in observability traces.
 */

import { describe, it, expect } from 'vitest';
import { parseAnomaly } from '../src/components/intelligence/knowledge_graph/utils/anomalyParser';

describe('anomalyParser', () => {
    it('successfully parses a valid unused symbol anomaly', () => {
        const raw = 'Unused symbol (0 incoming references): myUnusedFunc in src/components/MyComponent.tsx';
        const parsed = parseAnomaly(raw, 0);

        expect(parsed.type).toBe('UNUSED_SYMBOL');
        expect(parsed.name).toBe('myUnusedFunc');
        expect(parsed.rawPath).toBe('src/components/MyComponent.tsx');
        expect(parsed.original).toBe(raw);
        expect(parsed.stableKey).toBe('unused-myUnusedFunc-src/components/MyComponent.tsx');
    });

    it('gracefully handles an anomaly that does not match the expected format', () => {
        const raw = 'Something else happened in some other format';
        const parsed = parseAnomaly(raw, 42);

        expect(parsed.type).toBe('UNKNOWN');
        expect(parsed.name).toBe(raw);
        expect(parsed.rawPath).toBeNull();
        expect(parsed.original).toBe(raw);
        expect(parsed.stableKey).toBe('unknown-42-Something else happened in some other format');
    });

    it('correctly slices long unknown anomalies for the stableKey', () => {
        const longRaw = 'A'.repeat(100);
        const parsed = parseAnomaly(longRaw, 1);

        expect(parsed.type).toBe('UNKNOWN');
        expect(parsed.stableKey).toBe(`unknown-1-${'A'.repeat(64)}`);
    });
});

// Metadata: [anomalyParser_test]
