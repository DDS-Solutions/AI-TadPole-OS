/**
 * @docs ARCHITECTURE:UI-Components
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Scheduler / Cron_Config_Mini_Panel
 * - **Primary Entrypoints**: `Cron_Config_Mini_Panel`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import React, { useState, useMemo } from 'react';
import { Calendar, Repeat, Globe } from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';
import {
    synthesize_interval_cron,
    synthesize_date_cron,
    type Cron_Interval_Unit,
    type Cron_Timezone
} from '../../utils/cron_builder';

export interface Cron_Config_Mini_Panel_Props {
    current_cron: string;
    on_cron_change: (new_cron: string) => void;
}

export const Cron_Config_Mini_Panel: React.FC<Cron_Config_Mini_Panel_Props> = ({
    on_cron_change
}) => {
    // Mode switcher: 'intervals' vs 'calendar'
    const [active_tab, set_active_tab] = useState<'intervals' | 'calendar'>('intervals');

    // Timezone switcher: 'local' vs 'utc'
    const [timezone, set_timezone] = useState<Cron_Timezone>('utc');

    // Interval State
    const [interval_unit, set_interval_unit] = useState<Cron_Interval_Unit>('minutes');
    const [interval_value, set_interval_value] = useState<number>(15);
    const [interval_time, set_interval_time] = useState<string>('00:00');
    const [interval_day, set_interval_day] = useState<number>(1);

    // Calendar Date & Time State
    const default_date = useMemo(() => {
        const tomorrow = new Date();
        tomorrow.setDate(tomorrow.getDate() + 1);
        return tomorrow.toISOString().split('T')[0];
    }, []);

    const [cal_date, set_cal_date] = useState<string>(default_date);
    const [cal_time, set_cal_time] = useState<string>('12:00');

    const handle_tab_change = (tab: 'intervals' | 'calendar') => {
        set_active_tab(tab);
        if (tab === 'intervals') {
            const cron = synthesize_interval_cron({
                unit: interval_unit,
                interval: interval_value,
                time: interval_time,
                day_of_month: interval_day
            });
            on_cron_change(cron);
        } else {
            const cron = synthesize_date_cron({
                date: cal_date,
                time: cal_time,
                timezone
            });
            on_cron_change(cron);
        }
    };

    const handle_timezone_toggle = () => {
        const next_tz = timezone === 'utc' ? 'local' : 'utc';
        set_timezone(next_tz);
        if (active_tab === 'calendar') {
            const cron = synthesize_date_cron({
                date: cal_date,
                time: cal_time,
                timezone: next_tz
            });
            on_cron_change(cron);
        }
    };

    return (
        <div className="bg-slate-950/70 backdrop-blur-md border border-slate-800/80 rounded-xl p-3 flex flex-col justify-between min-w-0 transition-all hover:border-slate-700/60 shadow-lg">
            {/* Header: Title + Timezone Toggle + Tab Buttons */}
            <div className="flex items-center justify-between gap-2 mb-2">
                <div className="flex items-center gap-1.5 min-w-0">
                    <Tooltip content={i18n.t('scheduled_jobs.schedule_config_tooltip')} position="top">
                        <label className="text-[10px] font-bold font-mono text-zinc-400 uppercase tracking-widest cursor-help truncate">
                            {i18n.t('scheduled_jobs.schedule_config')}
                        </label>
                    </Tooltip>
                </div>

                <div className="flex items-center gap-1 shrink-0">
                    {/* Timezone Toggle */}
                    <button
                        type="button"
                        onClick={handle_timezone_toggle}
                        title={`Toggle timezone (Currently: ${timezone.toUpperCase()})`}
                        aria-label={`Toggle timezone mode: ${timezone.toUpperCase()}`}
                        className="px-2 py-0.5 rounded-lg text-[9px] font-mono font-bold flex items-center gap-1 bg-slate-900 border border-slate-800 text-zinc-400 hover:text-green-400 hover:border-green-500/40 transition-colors"
                    >
                        <Globe size={10} className={timezone === 'utc' ? 'text-green-400' : 'text-amber-400'} />
                        <span>{timezone === 'utc' ? i18n.t('scheduled_jobs.tz_utc') : i18n.t('scheduled_jobs.tz_local')}</span>
                    </button>

                    {/* Dual Tab Switcher */}
                    <div className="flex bg-slate-900 p-0.5 rounded-lg border border-slate-800">
                        <button
                            type="button"
                            onClick={() => handle_tab_change('intervals')}
                            aria-label={i18n.t('scheduled_jobs.tab_intervals')}
                            className={`px-2 py-0.5 text-[9px] font-mono font-bold rounded-md flex items-center gap-1 transition-all ${
                                active_tab === 'intervals'
                                    ? 'bg-green-600 text-white shadow-sm shadow-green-500/20'
                                    : 'text-zinc-500 hover:text-zinc-300'
                            }`}
                        >
                            <Repeat size={10} />
                            <span>{i18n.t('scheduled_jobs.tab_intervals')}</span>
                        </button>
                        <button
                            type="button"
                            onClick={() => handle_tab_change('calendar')}
                            aria-label={i18n.t('scheduled_jobs.tab_calendar')}
                            className={`px-2 py-0.5 text-[9px] font-mono font-bold rounded-md flex items-center gap-1 transition-all ${
                                active_tab === 'calendar'
                                    ? 'bg-green-600 text-white shadow-sm shadow-green-500/20'
                                    : 'text-zinc-500 hover:text-zinc-300'
                            }`}
                        >
                            <Calendar size={10} />
                            <span>{i18n.t('scheduled_jobs.tab_calendar')}</span>
                        </button>
                    </div>
                </div>
            </div>

            {/* Panel Body: Mode-Specific Form Inputs */}
            {active_tab === 'intervals' ? (
                <div className="space-y-2">
                    <div className="grid grid-cols-2 gap-2">
                        {/* Unit Selector */}
                        <div>
                            <span className="block text-[9px] font-mono text-zinc-500 mb-1 uppercase tracking-wider">{i18n.t('scheduled_jobs.frequency_label')}</span>
                            <select
                                aria-label={i18n.t('scheduled_jobs.frequency_label')}
                                value={interval_unit}
                                onChange={(e) => {
                                    const next_unit = e.target.value as Cron_Interval_Unit;
                                    set_interval_unit(next_unit);
                                    const next_cron = synthesize_interval_cron({
                                        unit: next_unit,
                                        interval: interval_value,
                                        time: interval_time,
                                        day_of_month: interval_day
                                    });
                                    on_cron_change(next_cron);
                                }}
                                className="w-full bg-[color:var(--color-background)] border border-[color:var(--color-border)] rounded-lg p-1.5 text-xs font-mono text-zinc-100 focus:border-green-500/50 focus:ring-1 focus:ring-green-500/30 outline-none transition-all"
                            >
                                <option value="minutes">{i18n.t('scheduled_jobs.unit_minutes')}</option>
                                <option value="hours">{i18n.t('scheduled_jobs.unit_hours')}</option>
                                <option value="days">{i18n.t('scheduled_jobs.unit_days')}</option>
                                <option value="months">{i18n.t('scheduled_jobs.unit_months')}</option>
                            </select>
                        </div>

                        {/* Interval Number */}
                        <div>
                            <span className="block text-[9px] font-mono text-zinc-500 mb-1 uppercase tracking-wider">
                                {i18n.t('scheduled_jobs.interval_label')} ({interval_unit})
                            </span>
                            <input
                                aria-label={i18n.t('scheduled_jobs.interval_label')}
                                type="number"
                                min={1}
                                max={interval_unit === 'minutes' ? 59 : interval_unit === 'hours' ? 23 : interval_unit === 'days' ? 31 : 12}
                                value={interval_value}
                                onChange={(e) => {
                                    const val = Math.max(1, parseInt(e.target.value, 10) || 1);
                                    set_interval_value(val);
                                    const next_cron = synthesize_interval_cron({
                                        unit: interval_unit,
                                        interval: val,
                                        time: interval_time,
                                        day_of_month: interval_day
                                    });
                                    on_cron_change(next_cron);
                                }}
                                className="w-full bg-[color:var(--color-background)] border border-[color:var(--color-border)] rounded-lg p-1.5 text-xs font-mono text-zinc-100 focus:border-green-500/50 focus:ring-1 focus:ring-green-500/30 outline-none transition-all"
                            />
                        </div>
                    </div>

                    {/* Sub-inputs if Days/Months or Hours require time specification */}
                    {(interval_unit === 'days' || interval_unit === 'months' || interval_unit === 'hours') && (
                        <div className="grid grid-cols-2 gap-2 pt-1 border-t border-slate-800/40">
                            {interval_unit === 'months' ? (
                                <div>
                                    <span className="block text-[9px] font-mono text-zinc-500 mb-1 uppercase tracking-wider">{i18n.t('scheduled_jobs.day_label')}</span>
                                    <input
                                        aria-label={i18n.t('scheduled_jobs.day_label')}
                                        type="number"
                                        min={1}
                                        max={31}
                                        value={interval_day}
                                        onChange={(e) => {
                                            const day = Math.min(31, Math.max(1, parseInt(e.target.value, 10) || 1));
                                            set_interval_day(day);
                                            const next_cron = synthesize_interval_cron({
                                                unit: interval_unit,
                                                interval: interval_value,
                                                time: interval_time,
                                                day_of_month: day
                                            });
                                            on_cron_change(next_cron);
                                        }}
                                        className="w-full bg-[color:var(--color-background)] border border-[color:var(--color-border)] rounded-lg p-1.5 text-xs font-mono text-zinc-100 focus:border-green-500/50 focus:ring-1 focus:ring-green-500/30 outline-none transition-all"
                                    />
                                </div>
                            ) : null}

                            <div className={interval_unit !== 'months' ? 'col-span-2' : ''}>
                                <span className="block text-[9px] font-mono text-zinc-500 mb-1 uppercase tracking-wider">
                                    {i18n.t('scheduled_jobs.time_label')} ({timezone.toUpperCase()})
                                </span>
                                <input
                                    aria-label={i18n.t('scheduled_jobs.time_label')}
                                    type="time"
                                    value={interval_time}
                                    onChange={(e) => {
                                        const time_val = e.target.value;
                                        set_interval_time(time_val);
                                        const next_cron = synthesize_interval_cron({
                                            unit: interval_unit,
                                            interval: interval_value,
                                            time: time_val,
                                            day_of_month: interval_day
                                        });
                                        on_cron_change(next_cron);
                                    }}
                                    className="w-full bg-[color:var(--color-background)] border border-[color:var(--color-border)] rounded-lg p-1.5 text-xs font-mono text-zinc-100 focus:border-green-500/50 focus:ring-1 focus:ring-green-500/30 outline-none transition-all"
                                />
                            </div>
                        </div>
                    )}
                </div>
            ) : (
                /* Calendar Date & Time Inputs */
                <div className="space-y-2">
                    <div className="grid grid-cols-2 gap-2">
                        {/* Date Picker */}
                        <div>
                            <span className="block text-[9px] font-mono text-zinc-500 mb-1 uppercase tracking-wider">
                                {i18n.t('scheduled_jobs.date_label')}
                            </span>
                            <input
                                aria-label={i18n.t('scheduled_jobs.date_label')}
                                type="date"
                                value={cal_date}
                                onChange={(e) => {
                                    const date_val = e.target.value;
                                    set_cal_date(date_val);
                                    const next_cron = synthesize_date_cron({
                                        date: date_val,
                                        time: cal_time,
                                        timezone
                                    });
                                    on_cron_change(next_cron);
                                }}
                                className="w-full bg-[color:var(--color-background)] border border-[color:var(--color-border)] rounded-lg p-1.5 text-xs font-mono text-zinc-100 focus:border-green-500/50 focus:ring-1 focus:ring-green-500/30 outline-none transition-all"
                            />
                        </div>

                        {/* Time Picker */}
                        <div>
                            <span className="block text-[9px] font-mono text-zinc-500 mb-1 uppercase tracking-wider">
                                {i18n.t('scheduled_jobs.time_label')} ({timezone.toUpperCase()})
                            </span>
                            <input
                                aria-label={i18n.t('scheduled_jobs.time_label')}
                                type="time"
                                value={cal_time}
                                onChange={(e) => {
                                    const time_val = e.target.value;
                                    set_cal_time(time_val);
                                    const next_cron = synthesize_date_cron({
                                        date: cal_date,
                                        time: time_val,
                                        timezone
                                    });
                                    on_cron_change(next_cron);
                                }}
                                className="w-full bg-[color:var(--color-background)] border border-[color:var(--color-border)] rounded-lg p-1.5 text-xs font-mono text-zinc-100 focus:border-green-500/50 focus:ring-1 focus:ring-green-500/30 outline-none transition-all"
                            />
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
};
