/**
 * @docs ARCHITECTURE:Quality:Verification
 *
 * ### AI Context Alignment
 * - **Subsystem**: Invariant Verification Suite / inv_010_telemetry_isolation.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 *
 * INV-010: Telemetry Graph & Kernel Safety Isolation.
 *
 * Asserts that:
 * 1. Telemetry_Graph.tsx isolates DAG layout recalculation to structural topology changes only.
 * 2. Telemetry_Node component enforces CSS layout containment.
 * 3. server-rs Cargo.toml enforces await_holding_lock = 'deny'.
 * 4. src-tauri main.rs cleans up sidecar processes on RunEvent::Exit.
 * 5. deploy-bunker-1.ps1 supports -WhatIf via SupportsShouldProcess.
 */

import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

describe('INV-010: Telemetry Graph & Kernel Safety Isolation', () => {
    it('Telemetry_Graph.tsx isolates layout effect to structure_hash only', () => {
        const source = readFileSync(
            resolve('src/components/Telemetry_Graph.tsx'),
            'utf-8'
        );

        // Layout effect must depend solely on structure_hash to prevent cascading re-renders on span data ticks
        expect(source).toContain('useEffect(() => {');
        expect(source).toContain('}, [structure_hash]);');
        expect(source).toContain('get_layouted_elements(nodes_ref.current, edges_ref.current)');
    });

    it('Telemetry_Node enforces CSS layout containment for 60fps rendering', () => {
        const source = readFileSync(
            resolve('src/components/Telemetry_Graph.tsx'),
            'utf-8'
        );

        // Custom node container must specify CSS containment and data-agent-state attribute
        expect(source).toContain("style={{ contain: 'layout style paint' }}");
        expect(source).toContain('data-agent-state=');
        expect(source).toContain("willChange: 'opacity, transform'");
    });

    it('server-rs enforces await_holding_lock = deny in clippy configuration', () => {
        const source = readFileSync(
            resolve('server-rs/Cargo.toml'),
            'utf-8'
        );

        expect(source).toContain('await_holding_lock = "deny"');
    });

    it('src-tauri terminates sidecars cleanly on RunEvent::Exit', () => {
        const source = readFileSync(
            resolve('src-tauri/src/main.rs'),
            'utf-8'
        );

        expect(source).toContain('tauri::RunEvent::Exit');
        expect(source).toContain('cleanup_existing_sidecars');
    });

    it('deploy-bunker-1.ps1 supports -WhatIf via SupportsShouldProcess and Set-StrictMode', () => {
        const source = readFileSync(
            resolve('scripts/deploy-bunker-1.ps1'),
            'utf-8'
        );

        expect(source).toContain('SupportsShouldProcess = $true');
        expect(source).toContain('Set-StrictMode -Version Latest');
        expect(source).toContain('$PSCmdlet.ShouldProcess');
    });
});
