/**
 * @docs ARCHITECTURE:Quality:Verification
 *
 * ### AI Context Alignment
 * - **Subsystem**: Invariant Verification Suite / inv_005_openui_depth.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 *
 * INV-005: OpenUI Render Depth — Recursive component tree caps at depth 10.
 *
 * Asserts that the MAX_RENDER_DEPTH constant in OpenUI_Renderer.tsx
 * is set to 10 (or less), preventing infinite recursion from malicious
 * or malformed OpenUI payloads.
 */

import { describe, it, expect } from 'vitest';

describe('INV-005: OpenUI recursive depth cap', () => {
    it('MAX_RENDER_DEPTH is set to 10 in source code', async () => {
        const { readFileSync } = await import('node:fs');
        const { resolve } = await import('node:path');
        const source = readFileSync(
            resolve('src/components/chat/OpenUI_Renderer.tsx'),
            'utf-8'
        );

        // Match the constant declaration
        const match = source.match(/const\s+MAX_RENDER_DEPTH\s*=\s*(\d+)/);
        expect(match).not.toBeNull();

        const depth = parseInt(match![1], 10);
        expect(depth).toBeLessThanOrEqual(10);
        expect(depth).toBeGreaterThan(0);
    });

    it('depth guard branch exists in the renderer function', async () => {
        const { readFileSync } = await import('node:fs');
        const { resolve } = await import('node:path');
        const source = readFileSync(
            resolve('src/components/chat/OpenUI_Renderer.tsx'),
            'utf-8'
        );

        // Verify the depth check exists: `if (depth > MAX_RENDER_DEPTH)`
        expect(source).toContain('depth > MAX_RENDER_DEPTH');
    });
});
