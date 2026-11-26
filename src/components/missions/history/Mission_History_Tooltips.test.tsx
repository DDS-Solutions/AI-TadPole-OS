/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Missions / History / Mission_History_Tooltips.test
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { render, screen } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import React from 'react';
import { Mission_Catalog_Sidebar } from './Mission_Catalog_Sidebar';
import { Metrics_Rollup_Pane } from './Metrics_Rollup_Pane';
import { Merkle_Audit_Viewer } from './Merkle_Audit_Viewer';
import { CAS_Diff_Inspector } from './CAS_Diff_Inspector';
import type { MissionSummaryItem, MissionDossier, MerkleStepRecord, CasDiffRecord } from '../../../types/mission_dossier';

// Mock Tooltip component to verify content props without relying on browser mouse events
vi.mock('../../ui', () => ({
    Tooltip: ({ children, content }: { children: React.ReactNode; content?: string }) => (
        <div data-testid="tooltip-wrapper" data-tooltip-content={content}>
            {children}
        </div>
    ),
}));

describe('Mission Flight Recorder Tooltips Verification', () => {
    it('verifies tooltips are configured in Mission_Catalog_Sidebar', () => {
        const mock_missions: MissionSummaryItem[] = [
            {
                id: 'mission-001',
                title: 'Test Sovereign Mission',
                status: 'completed',
                agent_id: 'alpha-1',
                created_at: new Date().toISOString(),
                updated_at: new Date().toISOString(),
                budget_usd: 10.0,
                cost_usd: 0.05,
                log_count: 12,
                diff_count: 2,
                is_pinned: false,
            },
        ];

        render(
            <Mission_Catalog_Sidebar
                missions={mock_missions}
                selected_id="mission-001"
                is_loading={false}
                total={1}
                page={1}
                per_page={25}
                search_query=""
                status_filter="all"
                on_select_mission={vi.fn()}
                on_search_change={vi.fn()}
                on_status_change={vi.fn()}
                on_page_change={vi.fn()}
                on_toggle_pin={vi.fn()}
                on_refresh={vi.fn()}
            />
        );

        const wrappers = screen.getAllByTestId('tooltip-wrapper');
        const tooltip_contents = wrappers.map((w) => w.getAttribute('data-tooltip-content'));

        expect(tooltip_contents).toContain('Refresh mission catalog from database');
        expect(tooltip_contents).toContain('Pin mission: Protect from Reaper deletion');
        expect(tooltip_contents).toContain('Discrete turn steps in Merkle audit trail');
        expect(tooltip_contents).toContain('Unique files with pre-mutation CAS snapshots');
        expect(tooltip_contents).toContain('Total LLM execution cost in USD');
        expect(tooltip_contents).toContain('Previous page');
        expect(tooltip_contents).toContain('Next page');
    });

    it('verifies tooltips are configured in Metrics_Rollup_Pane', () => {
        const mock_dossier: MissionDossier = {
            mission: {
                id: 'mission-001',
                title: 'Test Sovereign Mission',
                status: 'completed',
                agent_id: 'alpha-1',
                created_at: new Date().toISOString(),
                updated_at: new Date().toISOString(),
                cost_usd: 0.045,
                log_count: 5,
                diff_count: 1,
                is_pinned: true,
                budget_usd: 10.0,
            },
            logs: [
                {
                    id: 'step-1',
                    mission_id: 'mission-001',
                    step_index: 1,
                    agent_id: 'alpha-1',
                    source: 'agent',
                    text: 'Initialization step',
                    severity: 'info',
                    timestamp: new Date().toISOString(),
                    metadata: null,
                    prev_hash: null,
                    is_verified: true,
                    hash: 'abc1234567890abcdef',
                },
            ],
            diffs: [
                {
                    id: 1,
                    file_path: 'src/main.rs',
                    version_num: 1,
                    hash: 'cas1234567890abcdef',
                    pre_mutation_content: 'fn main() {}',
                    current_content: 'fn main() { println!("Hello"); }',
                    diff_summary: { additions: 1, deletions: 0, changes: 1 },
                    size_bytes: 40,
                    created_at: new Date().toISOString(),
                },
            ],
            memory_palace: [],
            trust_graph: { entities: [], relations: [] },
            economics: { transactions: [], budget_usd: 10.0, total_cost_usd: 0.045 },
            debrief: {
                verdict: 'VERIFIED_SUCCESS',
                drift_score: 0.02,
                agent99_notes: 'Nominal execution',
                recommendations: [],
            },
        };

        render(<Metrics_Rollup_Pane dossier={mock_dossier} />);

        const wrappers = screen.getAllByTestId('tooltip-wrapper');
        const tooltip_contents = wrappers.map((w) => w.getAttribute('data-tooltip-content'));

        expect(tooltip_contents).toContain('ALCOA+ cryptographic audit integrity and execution status');
        expect(tooltip_contents).toContain('All Merkle log hashes cryptographically verified');
        expect(tooltip_contents).toContain('Real-time LLM token execution cost vs allocated ceiling');
        expect(tooltip_contents).toContain('Total discrete execution steps logged in Merkle ledger');
        expect(tooltip_contents).toContain('Files mutated during mission with pre-state snapshots');
        expect(tooltip_contents).toContain('1 additions, 0 deletions across file mutations');
        expect(tooltip_contents).toContain('Execution strictly aligned with sovereign intent (< 0.150)');
    });

    it('verifies tooltips are configured in Merkle_Audit_Viewer', () => {
        const mock_logs: MerkleStepRecord[] = [
            {
                id: 'log-1',
                mission_id: 'mission-001',
                step_index: 1,
                agent_id: 'alpha-1',
                source: 'agent',
                text: 'Plan generated',
                severity: 'info',
                timestamp: new Date().toISOString(),
                metadata: null,
                is_verified: true,
                hash: '1234567890abcdef1234567890abcdef12345678',
                prev_hash: '0000000000000000000000000000000000000000',
            },
        ];

        render(<Merkle_Audit_Viewer logs={mock_logs} />);

        const wrappers = screen.getAllByTestId('tooltip-wrapper');
        const tooltip_contents = wrappers.map((w) => w.getAttribute('data-tooltip-content'));

        expect(tooltip_contents).toContain('ALCOA+ Cryptographic Merkle chain verified intact');
        expect(tooltip_contents).toContain('Previous Block Hash: 0000000000000000000000000000000000000000');
        expect(tooltip_contents).toContain('Block Hash: 1234567890abcdef1234567890abcdef12345678');
        expect(tooltip_contents).toContain('Copy full SHA-256 hash');
    });

    it('verifies tooltips are configured in CAS_Diff_Inspector', () => {
        const mock_diffs: CasDiffRecord[] = [
            {
                id: 1,
                file_path: 'src/main.rs',
                version_num: 2,
                hash: 'a1b2c3d4e5f678901234567890abcdef',
                pre_mutation_content: '// initial\nfn foo() {}',
                current_content: '// updated\nfn foo() { bar(); }',
                diff_summary: { additions: 2, deletions: 1, changes: 3 },
                size_bytes: 120,
                created_at: new Date().toISOString(),
            },
        ];

        render(<CAS_Diff_Inspector diffs={mock_diffs} />);

        const wrappers = screen.getAllByTestId('tooltip-wrapper');
        const tooltip_contents = wrappers.map((w) => w.getAttribute('data-tooltip-content'));

        expect(tooltip_contents).toContain('Pre-mutation CAS SHA-256: a1b2c3d4e5f678901234567890abcdef');
        expect(tooltip_contents).toContain('Side-by-side comparative diff');
        expect(tooltip_contents).toContain('Unified patch diff');
    });
});
