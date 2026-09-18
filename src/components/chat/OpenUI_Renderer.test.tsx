/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Chat / OpenUI_Renderer Tests
 * - **Primary Entrypoints**: none declared
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Tests DSL rendering completeness and interactive sorting.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { describe, it, expect } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { OpenUI_Renderer } from './OpenUI_Renderer';
import type { OpenUI_KPI_Card, OpenUI_Bar_Chart, OpenUI_Table, OpenUI_Layout } from '../../types';

describe('OpenUI_Renderer', () => {
    it('renders a KPI card with value, unit, and delta', () => {
        const kpi: OpenUI_KPI_Card = {
            kind: 'kpi_card',
            title: 'Total Revenue',
            value: 42500,
            unit: 'USD',
            delta: 12.5,
            delta_label: 'vs last month',
        };
        render(<OpenUI_Renderer dsl={kpi} />);

        expect(screen.getByText('Total Revenue')).toBeTruthy();
        expect(screen.getByText('42,500')).toBeTruthy();
        expect(screen.getByText('USD')).toBeTruthy();
        expect(screen.getByText('+12.5%')).toBeTruthy();
        expect(screen.getByText('vs last month')).toBeTruthy();
    });

    it('renders a bar chart with title and labels', () => {
        const chart: OpenUI_Bar_Chart = {
            kind: 'bar_chart',
            title: 'Monthly Revenue',
            labels: ['Jan', 'Feb', 'Mar'],
            datasets: [{
                label: 'Revenue',
                data: [100, 200, 150],
                color: '#6366f1',
            }],
        };
        render(<OpenUI_Renderer dsl={chart} />);

        expect(screen.getByText('Monthly Revenue')).toBeTruthy();
        expect(screen.getByText('Jan')).toBeTruthy();
        expect(screen.getByText('Feb')).toBeTruthy();
        expect(screen.getByText('Mar')).toBeTruthy();
    });

    it('renders a sortable table and sorts on click', () => {
        const table: OpenUI_Table = {
            kind: 'table',
            title: 'Agent Performance',
            sortable: true,
            columns: [
                { key: 'name', label: 'Agent' },
                { key: 'score', label: 'Score', align: 'right' },
            ],
            rows: [
                { name: 'Alpha', score: 95 },
                { name: 'Bravo', score: 87 },
                { name: 'Charlie', score: 91 },
            ],
        };
        render(<OpenUI_Renderer dsl={table} />);

        expect(screen.getByText('Agent Performance')).toBeTruthy();
        expect(screen.getByText('Alpha')).toBeTruthy();
        expect(screen.getByText('87')).toBeTruthy();

        // Click Score header to sort
        const score_header = screen.getByText('Score');
        fireEvent.click(score_header);

        // After sort ascending, first row should be 87 (Bravo)
        const cells = screen.getAllByRole('cell');
        const score_cells = cells.filter((_, i) => i % 2 === 1);
        expect(score_cells[0].textContent).toBe('87');
    });

    it('renders a nested layout with multiple children', () => {
        const layout: OpenUI_Layout = {
            kind: 'layout',
            direction: 'row',
            children: [
                { kind: 'kpi_card', title: 'Metric A', value: 100 },
                { kind: 'kpi_card', title: 'Metric B', value: 200 },
            ],
        };
        render(<OpenUI_Renderer dsl={layout} />);

        expect(screen.getByText('Metric A')).toBeTruthy();
        expect(screen.getByText('Metric B')).toBeTruthy();
    });

    it('renders KPI card with negative delta', () => {
        const kpi: OpenUI_KPI_Card = {
            kind: 'kpi_card',
            title: 'Error Rate',
            value: '3.2%',
            delta: -8,
        };
        render(<OpenUI_Renderer dsl={kpi} />);

        expect(screen.getByText('Error Rate')).toBeTruthy();
        expect(screen.getByText('3.2%')).toBeTruthy();
        expect(screen.getByText('-8%')).toBeTruthy();
    });
});
