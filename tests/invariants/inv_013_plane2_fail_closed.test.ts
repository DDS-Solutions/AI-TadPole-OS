/**
 * @docs ARCHITECTURE:Quality:Verification
 *
 * ### AI Context Alignment
 * - **Subsystem**: Invariant Verification Suite / inv_013_plane2_fail_closed.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 *
 * INV-013: Plane 2 SMB Digital Twin Fail-Closed Isolation Guard.
 */

import { describe, it, expect } from 'vitest';
import * as fs from 'fs';
import * as path from 'path';

describe('INV-013: Plane 2 SMB Digital Twin Fail-Closed Isolation Guard', () => {
    const root_dir = path.resolve(__dirname, '../../');
    const runner_mod_path = path.join(root_dir, 'server-rs/src/agent/runner/mod.rs');
    const tool_security_path = path.join(root_dir, 'server-rs/src/agent/runner/tools/security.rs');
    const toolbelt_path = path.join(root_dir, 'server-rs/src/agent/runner/synthesis/toolbelt.rs');
    const mission_route_path = path.join(root_dir, 'server-rs/src/routes/agent/missions.rs');
    const task_route_path = path.join(root_dir, 'server-rs/src/routes/agent/tasks.rs');

    it('asserts that PLANE2_DEPARTMENT_MARKER is canonical and immutable', () => {
        const content = fs.readFileSync(runner_mod_path, 'utf-8');
        expect(content).toContain('pub(crate) const PLANE2_DEPARTMENT_MARKER: &str = "Plane 2: SMB-Digital-Twin";');
    });

    it('verifies that is_plane2_department enforces case-insensitive whitespace-trimmed matching', () => {
        const content = fs.readFileSync(runner_mod_path, 'utf-8');
        expect(content).toContain('fn is_plane2_department(department: &str) -> bool');
        expect(content).toContain('.trim()');
        expect(content).toContain('.eq_ignore_ascii_case(PLANE2_DEPARTMENT_MARKER)');
    });

    it('verifies that the runner pre-flight check blocks Plane 2 dispatch', () => {
        const content = fs.readFileSync(runner_mod_path, 'utf-8');
        expect(content).toContain('AgentIdentity::is_plane2_department(&agent.identity.department)');
    });

    it('verifies that Zero Trust tool security strictly blocks Plane 2 tool calls', () => {
        const content = fs.readFileSync(tool_security_path, 'utf-8');
        expect(content).toContain('if ctx.identity.is_plane2()');
        expect(content).toContain('ToolExecutionError::SecurityBlocked');
    });

    it('verifies that toolbelt synthesis conceals mutating and code-editing tools from Plane 2', () => {
        const content = fs.readFileSync(toolbelt_path, 'utf-8');
        expect(content).toContain('if ctx.identity.is_plane2()');
    });

    it('verifies that HTTP routes reject Plane 2 tasks and missions before resource acquisition', () => {
        const mission_content = fs.readFileSync(mission_route_path, 'utf-8');
        expect(mission_content).toContain('AgentIdentity::is_plane2_department(&agent.identity.department)');

        const task_content = fs.readFileSync(task_route_path, 'utf-8');
        expect(task_content).toContain('AgentIdentity::is_plane2_department(&agent.identity.department)');
    });
});
