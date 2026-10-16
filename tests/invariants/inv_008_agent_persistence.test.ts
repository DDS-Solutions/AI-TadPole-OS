/**
 * @docs ARCHITECTURE:Persistence
 *
 * ### AI Context Alignment
 * - **Subsystem**: Invariant Verification Suite / inv_008_agent_persistence.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 *
 * INV-008: Agent Persistence & Credential Polarization Invariant Suite
 *
 * Asserts structural & behavioral guarantees in server-rs/src/agent/persistence/:
 *   1. Authoritative CAS versioning with RETURNING version and checked_add overflow guard
 *   2. save_agent_db_in_tx commit-isolation (version not mutated pre-commit)
 *   3. Production schema alignment for claim_agent (NOT NULL metadata/skills/workflows/tools)
 *   4. Multi-slot credential polarization (model, planning_slot, execution_slot, connector URIs)
 *   5. Security validate_path enforcement on RESOURCE_ROOT fallback in infra_config
 *   6. Collision-proof hash-based manifest ID generation and status token validation
 *   7. Child-first cascade deletion order for durable workflows
 */

import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

describe('INV-008: Agent Persistence & Credential Polarization Invariants', () => {
    const agentDbSource = readFileSync(
        resolve('server-rs/src/agent/persistence/agent_db.rs'),
        'utf-8'
    );
    const infraConfigSource = readFileSync(
        resolve('server-rs/src/agent/persistence/infra_config.rs'),
        'utf-8'
    );
    const syncManifestsSource = readFileSync(
        resolve('server-rs/src/agent/persistence/sync_manifests.rs'),
        'utf-8'
    );

    it('asserts authoritative CAS versioning with RETURNING version and checked_add in execute_save_agent', () => {
        expect(agentDbSource).toContain('RETURNING version');
        expect(agentDbSource).toContain('.checked_add(1)');
        expect(agentDbSource).toContain('Agent \'{}\' version overflow');
        // Must not contain unchecked addition
        expect(agentDbSource).not.toContain('let next_version = agent.version + 1;');
    });

    it('asserts save_agent_db_in_tx preserves in-memory version until commit succeeds', () => {
        expect(agentDbSource).toContain('pub async fn save_agent_db_in_tx(');
        expect(agentDbSource).toContain('-> Result<u32, AppError>');
        // Must not mutate agent.version inside save_agent_db_in_tx
        const txFnBody = agentDbSource.substring(
            agentDbSource.indexOf('pub async fn save_agent_db_in_tx'),
            agentDbSource.indexOf('pub async fn save_agents_json')
        );
        expect(txFnBody).not.toContain('agent.version = next_version;');
    });

    it('verifies claim_agent inserts all NOT NULL columns aligned with migration schema', () => {
        expect(agentDbSource).toContain('INSERT INTO agents (id, name, role, department, description, status, created_at, heartbeat_at, category, metadata, skills, workflows, mcp_tools, version)');
        expect(agentDbSource).toContain('VALUES (?1, ?2, \'Specialist\', \'Core\', \'Auto-created continuity agent identity\', \'busy\', ?3, ?3, \'user\', \'{}\', \'[]\', \'[]\', \'[]\', 1)');
    });

    it('enforces multi-slot credential polarization in save_agents_json', () => {
        expect(agentDbSource).toContain('agent.models.model.api_key = None;');
        expect(agentDbSource).toContain('agent.models.planning_slot');
        expect(agentDbSource).toContain('agent.models.execution_slot');
        expect(agentDbSource).toContain('slot.api_key = None;');
        expect(agentDbSource).toContain('slot.custom_headers = None;');
        expect(agentDbSource).toContain('***@'); // basic auth userinfo sanitization
        // Must not contain test skip no-op
        expect(agentDbSource).not.toContain('if cfg!(test) {\n        return Ok(());\n    }');
    });

    it('ensures RESOURCE_ROOT fallback in infra_config passes through security validate_path', () => {
        expect(infraConfigSource).toContain('crate::utils::security::validate_path(res_path, PROVIDERS_FILE)');
        expect(infraConfigSource).toContain('crate::utils::security::validate_path(res_path, MODELS_FILE)');
    });

    it('ensures sync manifests use collision-free hashed IDs and validate status vocabulary', () => {
        expect(syncManifestsSource).toContain('sha2::{Digest, Sha256}');
        expect(syncManifestsSource).toContain('VALID_SYNC_STATUSES');
        expect(syncManifestsSource).toContain('Invalid sync status');
    });

    it('ensures delete_agent_cascade removes durable_workflows child-first before mission_history', () => {
        const workflowsIdx = agentDbSource.indexOf('DELETE FROM durable_workflows');
        const missionHistoryIdx = agentDbSource.indexOf('DELETE FROM mission_history');
        expect(workflowsIdx).toBeGreaterThan(0);
        expect(missionHistoryIdx).toBeGreaterThan(0);
        expect(workflowsIdx).toBeLessThan(missionHistoryIdx);
    });
});
