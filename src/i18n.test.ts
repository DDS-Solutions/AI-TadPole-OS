/**
 * @docs ARCHITECTURE:Infrastructure
 *
 * ### AI Context Alignment
 * - **Subsystem**: System Core / i18n.test
 * - **Primary Entrypoints**: none (test harness)
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic translation resolution and strict fallback contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: `src/i18n.test.ts`
 */

import { describe, it, expect, vi } from 'vitest';

// Unmock to test the real i18n engine instead of the global setup mock
vi.unmock('./i18n');

import { i18n } from './i18n';

describe('i18n localization engine', () => {
    describe('key resolution & namespace routing', () => {
        it('resolves direct keys across namespaces', () => {
            expect(i18n.t('import.title', { type: 'SKILL' })).toBe('Import SKILL');
            expect(i18n.t('import.target_hub')).toBe('Target Capability Hub');
            expect(i18n.t('common.close_modal')).toBe('Close Modal');
            expect(i18n.t('common.delete')).toBe('Delete');
        });

        it('routes legacy prefixes to their target domains via namespaceMap', () => {
            // command -> system.command
            expect(i18n.t('command.ops_center')).toBe('Operations Dashboard');
            expect(i18n.t('command.search_placeholder')).toContain('Type a command');

            // agent_card -> agent.agent_card
            expect(i18n.t('agent_card.vision')).toBe('Vision Capabilities');
            expect(i18n.t('agent_card.tools')).toBe('Tool Calling Supported');

            // voice -> mission.voice
            expect(i18n.t('voice.close')).toBe('Close Voice Hub');
            expect(i18n.t('voice.start')).toBe('Start Voice Uplink');

            // ops -> system.ops
            expect(i18n.t('ops.placeholder_name')).toBe('UNNAMED_NODE');
        });

        it('preserves generic loading at root without clobbering by security', () => {
            expect(i18n.t('loading')).toBe('Loading Module...');
            expect(i18n.t('security.loading')).toBe('Initializing Governance Telemetry...');
        });

        it('resolves newly added missing keys', () => {
            expect(i18n.t('template_store.asset_agents')).toBe('Agents');
            expect(i18n.t('template_store.close_modal')).toBe('Close modal');
            expect(i18n.t('template_store.mcp_wizard_title')).toBe('Configure Connector Secrets');
            expect(i18n.t('skills.remediation_steps')).toBe('Remediation / Recommendation');
            expect(i18n.t('provider.field_rpm_label')).toBe('Requests Per Minute (RPM)');
            expect(i18n.t('workspaces.tooltip_legacy_silo', { name: 'Sentinel' })).toBe('Sentinel is not assigned to any active mission cluster');
            expect(i18n.t('system.connection_error')).toBe('Neural Link Connection Error • Offline');
            expect(i18n.t('telemetry_graph.aria_filter_mission')).toBe('Filter by Mission ID');
        });
    });

    describe('fallback and defaultValue handling', () => {
        it('falls back to defaultValue when key is not found', () => {
            const fallback = i18n.t('non.existent.translation.key', { defaultValue: 'Fallback Message' });
            expect(fallback).toBe('Fallback Message');
        });

        it('interpolates parameters into defaultValue when key is missing', () => {
            const fallback = i18n.t('non.existent.key', {
                name: 'Alpha',
                defaultValue: 'Node {{name}} standing by.'
            });
            expect(fallback).toBe('Node Alpha standing by.');
        });

        it('returns raw key if not found and no defaultValue is provided', () => {
            expect(i18n.t('some.missing.key')).toBe('some.missing.key');
        });
    });

    describe('parameter interpolation', () => {
        it('interpolates standard {{key}} placeholders', () => {
            expect(i18n.t('ops.event_agent_init', { name: 'Architect' })).toBe('Initialized agent Architect');
        });

        it('interpolates {{param:key}} placeholders', () => {
            expect(i18n.t('agent_card.tooltip_full_id', { name: 'Node-01' })).toBe('Full Identifier: Node-01');
        });
    });

    describe('returnObjects option', () => {
        it('returns raw object when returnObjects is true', () => {
            const obj = i18n.t('status', { returnObjects: true });
            expect(typeof obj).toBe('object');
            expect(obj).toHaveProperty('idle');
            expect(obj).toHaveProperty('active');
        });
    });
});
