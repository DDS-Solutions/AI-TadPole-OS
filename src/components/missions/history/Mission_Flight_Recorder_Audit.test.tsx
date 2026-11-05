/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Missions / History / Mission_Flight_Recorder_Audit.test
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import React from 'react';
import { Metrics_Rollup_Pane } from './Metrics_Rollup_Pane';
import { Agent99_Debrief_Viewer } from './Agent99_Debrief_Viewer';
import { Mission_Catalog_Sidebar } from './Mission_Catalog_Sidebar';
import { Merkle_Audit_Viewer } from './Merkle_Audit_Viewer';
import { compute_aligned_diff } from './CAS_Diff_Inspector';
import { Mission_Flight_Dossier } from './Mission_Flight_Dossier';
import type { MissionDossier, MissionSummaryItem, MerkleStepRecord } from '../../../types/mission_dossier';

vi.mock('../../ui', () => ({
    Tooltip: ({ children, content }: { children: React.ReactNode; content?: string }) => (
        <div data-testid="tooltip-wrapper" data-tooltip-content={content}>
            {children}
        </div>
    ),
}));

describe('Mission Flight Recorder Audit Remediations (S-10)', () => {
    const base_mission: MissionSummaryItem = {
        id: 'mission-audit-01',
        title: 'Audit Remediation Mission',
        status: 'completed',
        agent_id: 'agent-101',
        created_at: '2026-10-28T12:00:00Z',
        updated_at: '2026-10-28T12:30:00Z',
        cost_usd: 0.1234,
        budget_usd: 5.0,
        log_count: 3,
        diff_count: 0,
        is_pinned: false,
    };

    it('R-02: displays Unverified / Legacy badge when logs have NULL hashes', () => {
        const legacy_dossier: MissionDossier = {
            mission: base_mission,
            logs: [
                {
                    id: 'log-1',
                    mission_id: 'mission-audit-01',
                    agent_id: 'agent-101',
                    source: 'System',
                    text: 'Legacy entry',
                    severity: 'info',
                    timestamp: '2026-10-28T12:00:01Z',
                    metadata: null,
                    hash: null,
                    prev_hash: null,
                    is_verified: false,
                },
            ],
            diffs: [],
            memory_palace: [],
            trust_graph: { entities: [], relations: [] },
            economics: { total_cost_usd: 0.1234, budget_usd: 5.0, transactions: [] },
            debrief: {
                agent99_notes: null,
                drift_score: null,
                verdict: 'COMPLETED_UNVERIFIED',
                recommendations: [],
            },
        };

        render(<Metrics_Rollup_Pane dossier={legacy_dossier} />);
        expect(screen.getByText('Unverified Legacy')).toBeDefined();
    });

    it('R-02: displays Tamper Flagged badge when hash mismatch is present', () => {
        const tamper_dossier: MissionDossier = {
            mission: base_mission,
            logs: [
                {
                    id: 'log-1',
                    mission_id: 'mission-audit-01',
                    agent_id: 'agent-101',
                    source: 'System',
                    text: 'Tampered entry',
                    severity: 'info',
                    timestamp: '2026-10-28T12:00:01Z',
                    metadata: null,
                    hash: 'fake-hash-value',
                    prev_hash: null,
                    is_verified: false,
                },
            ],
            diffs: [],
            memory_palace: [],
            trust_graph: { entities: [], relations: [] },
            economics: { total_cost_usd: 0.1234, budget_usd: 5.0, transactions: [] },
            debrief: {
                agent99_notes: null,
                drift_score: null,
                verdict: 'COMPLETED_WITH_TAMPER_FLAG',
                recommendations: [],
            },
        };

        render(<Metrics_Rollup_Pane dossier={tamper_dossier} />);
        expect(screen.getByText('Tamper Flagged')).toBeDefined();
    });

    it('R-01 & SEC-05: Agent 99 Debrief displays truthful empty state and evaluated drift', () => {
        const debrief_no_auditor = {
            agent99_notes: null,
            drift_score: 0.92,
            verdict: 'COMPLETED_UNVERIFIED',
            recommendations: ['Check system logs'],
        };

        render(<Agent99_Debrief_Viewer debrief={debrief_no_auditor} mission_id="mission-audit-01" />);

        // Must NOT render synthetic "Agent 99 audited execution logs: Task concluded with nominal status..."
        expect(screen.queryByText(/Merkle audit trail is continuous from genesis/)).toBeNull();
        expect(screen.getByText('No Agent 99 debrief transcript recorded for this flight.')).toBeDefined();

        // Must display alert when drift > 0.85 rather than unconditional "remained within strict tolerances"
        expect(screen.getByText(/Vector drift exceeded sovereign threshold/)).toBeDefined();
    });

    it('R-08: Mission catalog renders CAS badge when diff_count is 0', () => {
        const missions: MissionSummaryItem[] = [
            {
                ...base_mission,
                diff_count: 0,
            },
        ];

        render(
            <Mission_Catalog_Sidebar
                missions={missions}
                selected_id="mission-audit-01"
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

        // Diff count '0' should be rendered in the document
        expect(screen.getByText('0')).toBeDefined();
    });

    it('R-02: Merkle_Audit_Viewer correctly distinguishes Unverified Legacy from Tamper Flag', () => {
        const test_logs: MerkleStepRecord[] = [
            {
                id: 'log-legacy',
                mission_id: 'mission-audit-01',
                step_index: 1,
                agent_id: 'agent-101',
                source: 'System',
                text: 'Old unhashed log entry',
                severity: 'info',
                timestamp: '2026-10-28T12:00:00Z',
                metadata: null,
                hash: null,
                prev_hash: null,
                is_verified: false,
            },
            {
                id: 'log-tampered',
                mission_id: 'mission-audit-01',
                step_index: 2,
                agent_id: 'agent-101',
                source: 'System',
                text: 'Tampered hash entry',
                severity: 'error',
                timestamp: '2026-10-28T12:00:05Z',
                metadata: null,
                hash: 'mismatched-hash-xyz',
                prev_hash: null,
                is_verified: false,
            },
        ];

        render(<Merkle_Audit_Viewer logs={test_logs} />);
        expect(screen.getByText('Unverified Legacy')).toBeDefined();
        expect(screen.getByText('Tamper Flag')).toBeDefined();
    });

    it('R-07: Merkle_Audit_Viewer preserves chronological step numbering when filtered', () => {
        const test_logs: MerkleStepRecord[] = [
            {
                id: 'step-1',
                mission_id: 'mission-audit-01',
                step_index: 1,
                agent_id: 'agent-101',
                source: 'System',
                text: 'Initialization info',
                severity: 'info',
                timestamp: '2026-10-28T12:00:00Z',
                metadata: null,
                hash: 'hash-1',
                prev_hash: null,
                is_verified: true,
            },
            {
                id: 'step-2',
                mission_id: 'mission-audit-01',
                step_index: 2,
                agent_id: 'agent-101',
                source: 'System',
                text: 'Critical error at step two',
                severity: 'error',
                timestamp: '2026-10-28T12:00:05Z',
                metadata: null,
                hash: 'hash-2',
                prev_hash: 'hash-1',
                is_verified: true,
            },
        ];

        render(<Merkle_Audit_Viewer logs={test_logs} />);

        // Filter to errors only
        const error_filter_button = screen.getByRole('button', { name: /^error$/i });
        fireEvent.click(error_filter_button);

        // The only displayed log was original step #2; must NOT be renumbered to #1
        expect(screen.getByText('#2')).toBeDefined();
        expect(screen.queryByText('#1')).toBeNull();
    });

    it('R-03: compute_aligned_diff aligns insertions without cascading false modification shifts', () => {
        const pre = 'lineA\nlineB\nlineC';
        const cur = 'insertedHeader\nlineA\nlineB\nlineC';

        const aligned = compute_aligned_diff(pre, cur);

        // Row 1: insertedHeader (added)
        expect(aligned[0].is_added).toBe(true);
        expect(aligned[0].cur).toBe('insertedHeader');
        expect(aligned[0].pre).toBeNull();

        // Rows 2, 3, 4: lineA, lineB, lineC must be equal, NOT modified
        expect(aligned[1].is_modified).toBe(false);
        expect(aligned[1].pre).toBe('lineA');
        expect(aligned[1].cur).toBe('lineA');

        expect(aligned[2].is_modified).toBe(false);
        expect(aligned[2].pre).toBe('lineB');
        expect(aligned[2].cur).toBe('lineB');

        expect(aligned[3].is_modified).toBe(false);
        expect(aligned[3].pre).toBe('lineC');
        expect(aligned[3].cur).toBe('lineC');
    });

    it('R-06: Harmonized drift thresholds treat 0.60 as ELEVATED and 0.92 as ANOMALY', () => {
        const elevated_dossier: MissionDossier = {
            mission: base_mission,
            logs: [],
            diffs: [],
            memory_palace: [],
            trust_graph: { entities: [], relations: [] },
            economics: { total_cost_usd: 0.1, budget_usd: 5.0, transactions: [] },
            debrief: {
                agent99_notes: 'Moderate deviation',
                drift_score: 0.60,
                verdict: 'STATUS_NOMINAL',
                recommendations: [],
            },
        };

        const { rerender } = render(<Metrics_Rollup_Pane dossier={elevated_dossier} />);
        expect(screen.getByText('ELEVATED')).toBeDefined();

        const anomaly_dossier: MissionDossier = {
            ...elevated_dossier,
            debrief: {
                ...elevated_dossier.debrief,
                drift_score: 0.92,
            },
        };

        rerender(<Metrics_Rollup_Pane dossier={anomaly_dossier} />);
        expect(screen.getByText('ANOMALY')).toBeDefined();
    });

    it('R-09: Mission catalog and dossier tabs declare accessible ARIA roles', () => {
        const missions: MissionSummaryItem[] = [base_mission];

        const { unmount } = render(
            <Mission_Catalog_Sidebar
                missions={missions}
                selected_id="mission-audit-01"
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

        // Sidebar items must have button role and aria-pressed
        const item = screen.getByRole('button', { name: /Select mission/i });
        expect(item).toBeDefined();
        expect(item.getAttribute('aria-pressed')).toBe('true');
        unmount();

        const full_dossier: MissionDossier = {
            mission: base_mission,
            logs: [],
            diffs: [],
            memory_palace: [],
            trust_graph: { entities: [], relations: [] },
            economics: { total_cost_usd: 0.1, budget_usd: 5.0, transactions: [] },
            debrief: {
                agent99_notes: null,
                drift_score: 0.05,
                verdict: 'VERIFIED_SUCCESS',
                recommendations: [],
            },
        };

        render(<Mission_Flight_Dossier dossier={full_dossier} on_toggle_pin={vi.fn()} />);

        // Tablist and tabpanel must be present
        expect(screen.getByRole('tablist', { name: 'Mission Dossier Views' })).toBeDefined();
        expect(screen.getByRole('tabpanel')).toBeDefined();
    });
});
