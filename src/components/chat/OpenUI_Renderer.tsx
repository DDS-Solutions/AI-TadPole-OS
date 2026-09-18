/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Chat / OpenUI_Renderer
 * - **Primary Entrypoints**: `OpenUI_Renderer`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Pure render component — no side effects, no network calls.
 * - `[Structural]` DSL is data-only, never eval'd — safe from injection.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: `OpenUI_Renderer.test.tsx`
 */

import React, { useCallback, useMemo, useState } from 'react';
import { motion } from 'framer-motion';
import { TrendingUp, TrendingDown, Minus, Table2, BarChart3, LayoutGrid } from 'lucide-react';
import type {
    OpenUI_DSL,
    OpenUI_KPI_Card,
    OpenUI_Bar_Chart,
    OpenUI_Table,
    OpenUI_Layout,
} from '../../types';

// ── KPI Card ─────────────────────────────────────────────────

const KPI_Card: React.FC<{ data: OpenUI_KPI_Card }> = React.memo(({ data }) => {
    const delta_color = (data.delta ?? 0) > 0
        ? 'text-emerald-400'
        : (data.delta ?? 0) < 0
            ? 'text-rose-400'
            : 'text-zinc-500';

    const DeltaIcon = (data.delta ?? 0) > 0
        ? TrendingUp
        : (data.delta ?? 0) < 0
            ? TrendingDown
            : Minus;

    return (
        <motion.div
            initial={{ opacity: 0, scale: 0.95 }}
            animate={{ opacity: 1, scale: 1 }}
            className="flex flex-col gap-1 p-3 rounded-xl bg-zinc-900/60 border border-zinc-700/40 backdrop-blur-sm min-w-[140px]"
        >
            <span className="text-[9px] font-bold uppercase tracking-[0.15em] text-zinc-500">
                {data.title}
            </span>
            <div className="flex items-baseline gap-1.5">
                <span className="text-lg font-bold text-zinc-100 tabular-nums">
                    {typeof data.value === 'number' ? data.value.toLocaleString() : data.value}
                </span>
                {data.unit && (
                    <span className="text-[10px] text-zinc-500 font-mono">{data.unit}</span>
                )}
            </div>
            {data.delta !== undefined && (
                <div className={`flex items-center gap-1 text-[10px] font-mono ${delta_color}`}>
                    <DeltaIcon size={10} />
                    <span>{data.delta > 0 ? '+' : ''}{data.delta}%</span>
                    {data.delta_label && (
                        <span className="text-zinc-600 ml-1">{data.delta_label}</span>
                    )}
                </div>
            )}
        </motion.div>
    );
});
KPI_Card.displayName = 'KPI_Card';

// ── Bar Chart ────────────────────────────────────────────────

const DEFAULT_COLORS = [
    '#6366f1', '#22d3ee', '#f59e0b', '#ef4444', '#10b981', '#8b5cf6',
];

const Bar_Chart: React.FC<{ data: OpenUI_Bar_Chart }> = React.memo(({ data }) => {
    const max_value = useMemo(() => {
        let max = 0;
        for (const ds of data.datasets) {
            for (const v of ds.data) {
                if (v > max) max = v;
            }
        }
        return max || 1;
    }, [data.datasets]);

    return (
        <motion.div
            initial={{ opacity: 0, y: 8 }}
            animate={{ opacity: 1, y: 0 }}
            className="flex flex-col gap-2 p-3 rounded-xl bg-zinc-900/60 border border-zinc-700/40 backdrop-blur-sm"
        >
            <div className="flex items-center gap-2">
                <BarChart3 size={12} className="text-indigo-400" />
                <span className="text-[10px] font-bold uppercase tracking-[0.12em] text-zinc-400">
                    {data.title}
                </span>
            </div>

            {/* Legend */}
            {data.datasets.length > 1 && (
                <div className="flex flex-wrap gap-3 text-[9px] text-zinc-500">
                    {data.datasets.map((ds, i) => (
                        <div key={ds.label} className="flex items-center gap-1">
                            <div
                                className="w-2 h-2 rounded-full"
                                style={{ backgroundColor: ds.color || DEFAULT_COLORS[i % DEFAULT_COLORS.length] }}
                            />
                            <span>{ds.label}</span>
                        </div>
                    ))}
                </div>
            )}

            {/* Bars */}
            <div className="flex items-end gap-1 h-24">
                {data.labels.map((label, li) => (
                    <div key={label} className="flex-1 flex flex-col items-center gap-0.5">
                        <div className="flex items-end gap-px w-full h-20">
                            {data.datasets.map((ds, di) => {
                                const val = ds.data[li] ?? 0;
                                const pct = (val / max_value) * 100;
                                return (
                                    <motion.div
                                        key={ds.label}
                                        initial={{ height: 0 }}
                                        animate={{ height: `${pct}%` }}
                                        transition={{ duration: 0.5, delay: li * 0.05 }}
                                        className="flex-1 rounded-t-sm"
                                        style={{
                                            backgroundColor: ds.color || DEFAULT_COLORS[di % DEFAULT_COLORS.length],
                                            minHeight: val > 0 ? 2 : 0,
                                        }}
                                        title={`${ds.label}: ${val}`}
                                    />
                                );
                            })}
                        </div>
                        <span className="text-[8px] text-zinc-600 truncate max-w-full">{label}</span>
                    </div>
                ))}
            </div>
        </motion.div>
    );
});
Bar_Chart.displayName = 'Bar_Chart';

// ── Sortable Table ───────────────────────────────────────────

const Data_Table: React.FC<{ data: OpenUI_Table }> = React.memo(({ data }) => {
    const [sort_key, set_sort_key] = useState<string | null>(null);
    const [sort_asc, set_sort_asc] = useState(true);

    const sorted_rows = useMemo(() => {
        if (!sort_key) return data.rows;
        return [...data.rows].sort((a, b) => {
            const va = a[sort_key];
            const vb = b[sort_key];
            if (typeof va === 'number' && typeof vb === 'number') {
                return sort_asc ? va - vb : vb - va;
            }
            return sort_asc
                ? String(va).localeCompare(String(vb))
                : String(vb).localeCompare(String(va));
        });
    }, [data.rows, sort_key, sort_asc]);

    const handle_sort = useCallback((key: string) => {
        if (!data.sortable) return;
        if (sort_key === key) {
            set_sort_asc((prev) => !prev);
        } else {
            set_sort_key(key);
            set_sort_asc(true);
        }
    }, [data.sortable, sort_key]);

    const table_header = useMemo(() => (
        <thead>
            <tr className="border-b border-zinc-700/50">
                {data.columns.map((col) => (
                    <th
                        key={col.key}
                        className={`py-1.5 px-2 font-bold text-zinc-400 uppercase tracking-wider text-[9px] ${
                            data.sortable ? 'cursor-pointer hover:text-zinc-200 transition-colors select-none' : ''
                        } ${
                            col.align === 'right' ? 'text-right'
                            : col.align === 'center' ? 'text-center'
                            : 'text-left'
                        }`}
                        onClick={() => handle_sort(col.key)}
                    >
                        <span className="flex items-center gap-1">
                            {col.label}
                            {sort_key === col.key && (
                                <span className="text-indigo-400">{sort_asc ? '↑' : '↓'}</span>
                            )}
                        </span>
                    </th>
                ))}
            </tr>
        </thead>
    ), [data.columns, data.sortable, handle_sort, sort_key, sort_asc]);

    const table_body = useMemo(() => (
        <tbody>
            {sorted_rows.map((row, ri) => {
                const row_key = (row.id ?? row.key ?? `row-${ri}`) as React.Key;
                return (
                    <tr
                        key={row_key}
                        className="border-b border-zinc-800/30 hover:bg-zinc-800/30 transition-colors"
                    >
                        {data.columns.map((col) => (
                            <td
                                key={col.key}
                                className={`py-1.5 px-2 text-zinc-300 tabular-nums ${
                                    col.align === 'right' ? 'text-right'
                                    : col.align === 'center' ? 'text-center'
                                    : 'text-left'
                                }`}
                            >
                                {typeof row[col.key] === 'boolean'
                                    ? (row[col.key] ? '✓' : '✗')
                                    : String(row[col.key] ?? '')}
                            </td>
                        ))}
                    </tr>
                );
            })}
        </tbody>
    ), [data.columns, sorted_rows]);

    return (
        <motion.div
            initial={{ opacity: 0, y: 8 }}
            animate={{ opacity: 1, y: 0 }}
            className="flex flex-col gap-2 p-3 rounded-xl bg-zinc-900/60 border border-zinc-700/40 backdrop-blur-sm overflow-x-auto"
        >
            {data.title && (
                <div className="flex items-center gap-2">
                    <Table2 size={12} className="text-cyan-400" />
                    <span className="text-[10px] font-bold uppercase tracking-[0.12em] text-zinc-400">
                        {data.title}
                    </span>
                </div>
            )}

            <table className="w-full text-[11px] border-collapse">
                {table_header}
                {table_body}
            </table>
        </motion.div>
    );
});
Data_Table.displayName = 'Data_Table';

// ── Layout Container ─────────────────────────────────────────

const Layout_Container: React.FC<{ data: OpenUI_Layout }> = React.memo(({ data }) => (
    <div
        className={`flex gap-2 ${
            data.direction === 'column' ? 'flex-col' : 'flex-row flex-wrap'
        }`}
    >
        {data.children.map((child, i) => (
            <OpenUI_Renderer key={i} dsl={child} />
        ))}
    </div>
));
Layout_Container.displayName = 'Layout_Container';

// ── Main Renderer ────────────────────────────────────────────

interface OpenUI_Renderer_Props {
    dsl: OpenUI_DSL;
}

/**
 * OpenUI_Renderer — Renders a strongly-typed DSL payload into interactive
 * React components inside chat messages. Supports KPI cards, bar charts,
 * sortable tables, and recursive layouts.
 */
export const OpenUI_Renderer: React.FC<OpenUI_Renderer_Props> = React.memo(({ dsl }) => {
    switch (dsl.kind) {
        case 'kpi_card':
            return <KPI_Card data={dsl} />;
        case 'bar_chart':
            return <Bar_Chart data={dsl} />;
        case 'table':
            return <Data_Table data={dsl} />;
        case 'layout':
            return <Layout_Container data={dsl} />;
        default: {
            // Exhaustive check — if a new kind is added, TypeScript catches it
            const _exhaustive: never = dsl;
            return (
                <div className="p-2 text-[10px] text-rose-400 border border-rose-800/30 rounded-lg bg-rose-950/20">
                    <LayoutGrid size={10} className="inline mr-1" />
                    Unknown OpenUI kind: {JSON.stringify((_exhaustive as Record<string, unknown>)?.kind)}
                </div>
            );
        }
    }
});

OpenUI_Renderer.displayName = 'OpenUI_Renderer';
