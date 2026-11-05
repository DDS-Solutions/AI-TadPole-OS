/**
 * @docs ARCHITECTURE:Testing
 *
 * ### AI Assist Note
 * - Keep fixtures isolated and assert the public behavior of the extracted boundaries.
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Scheduler / Cron_Config_Mini_Panel.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Cron_Config_Mini_Panel } from './Cron_Config_Mini_Panel';
import { i18n } from '../../i18n';

describe('Cron_Config_Mini_Panel Component', () => {
    it('renders Schedule Helper title and default interval controls', () => {
        const on_cron_change = vi.fn();
        render(<Cron_Config_Mini_Panel current_cron="*/15 * * * *" on_cron_change={on_cron_change} />);

        expect(screen.getByText(i18n.t('scheduled_jobs.schedule_config'))).toBeDefined();
        expect(screen.getByLabelText('Frequency')).toBeDefined();
        expect(screen.getByLabelText('Every')).toBeDefined();
    });

    it('changes interval frequency unit to hours and synthesizes cron', () => {
        const on_cron_change = vi.fn();
        render(<Cron_Config_Mini_Panel current_cron="*/15 * * * *" on_cron_change={on_cron_change} />);

        const unit_select = screen.getByLabelText('Frequency');
        fireEvent.change(unit_select, { target: { value: 'hours' } });

        expect(on_cron_change).toHaveBeenCalledWith(expect.stringContaining('* * *'));
    });

    it('changes interval step value and synthesizes cron', () => {
        const on_cron_change = vi.fn();
        render(<Cron_Config_Mini_Panel current_cron="*/15 * * * *" on_cron_change={on_cron_change} />);

        const interval_input = screen.getByLabelText('Every');
        fireEvent.change(interval_input, { target: { value: '30' } });

        expect(on_cron_change).toHaveBeenCalledWith('*/30 * * * *');
    });

    it('switches between intervals and calendar tabs', () => {
        const on_cron_change = vi.fn();
        render(<Cron_Config_Mini_Panel current_cron="*/15 * * * *" on_cron_change={on_cron_change} />);

        const cal_tab_btn = screen.getByLabelText('Calendar Date & Time');
        fireEvent.click(cal_tab_btn);

        expect(on_cron_change).toHaveBeenCalled();
        expect(screen.getByLabelText('Date')).toBeDefined();
        expect(screen.getByLabelText('Time')).toBeDefined();

        const interval_tab_btn = screen.getByLabelText('Intervals');
        fireEvent.click(interval_tab_btn);

        expect(screen.getByLabelText('Frequency')).toBeDefined();
    });

    it('toggles timezone between UTC and LOCAL and synthesizes date cron', () => {
        const on_cron_change = vi.fn();
        render(<Cron_Config_Mini_Panel current_cron="*/15 * * * *" on_cron_change={on_cron_change} />);

        // Switch to calendar tab first
        fireEvent.click(screen.getByLabelText('Calendar Date & Time'));
        on_cron_change.mockClear();

        // Toggle timezone
        const tz_toggle = screen.getByLabelText(/Toggle timezone mode/i);
        fireEvent.click(tz_toggle);

        expect(on_cron_change).toHaveBeenCalled();
    });

    it('updates calendar date and time inputs', () => {
        const on_cron_change = vi.fn();
        render(<Cron_Config_Mini_Panel current_cron="*/15 * * * *" on_cron_change={on_cron_change} />);

        fireEvent.click(screen.getByLabelText('Calendar Date & Time'));
        on_cron_change.mockClear();

        const date_input = screen.getByLabelText('Date');
        fireEvent.change(date_input, { target: { value: '2026-11-20' } });

        expect(on_cron_change).toHaveBeenCalledWith('0 12 20 11 *');

        const time_input = screen.getByLabelText('Time');
        fireEvent.change(time_input, { target: { value: '14:45' } });

        expect(on_cron_change).toHaveBeenCalledWith('45 14 20 11 *');
    });
});
