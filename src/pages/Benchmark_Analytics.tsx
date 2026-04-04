import React, { useState, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import {
    BarChart3,
    History,
    TrendingUp,
    AlertCircle,
    CheckCircle2,
    Activity,
    ArrowRightLeft,
    ChevronRight,
    Zap,
    Loader2
} from 'lucide-react';
import { tadpole_os_service } from '../services/tadpoleos_service';
import { event_bus } from '../services/event_bus';
import { Tooltip } from '../components/ui';
import { i18n } from '../i18n';

interface BenchmarkResult {
    id: string;
    name: string;
    category: string;
    test_id: string;
    mean_ms: number;
    p95_ms?: number;
    p99_ms?: number;
    target_value?: string;
    status: string;
    metadata?: string;
    created_at: string;
}

const Benchmark_Analytics: React.FC = () => {
    const [benchmarks, setBenchmarks] = useState<BenchmarkResult[]>([]);
    const [loading, setLoading] = useState(true);
    const [runningId, setRunningId] = useState<string | null>(null);
    const [selectedTests, setSelectedTests] = useState<string[]>([]);

    useEffect(() => {
        fetchBenchmarks();
    }, []);

    const fetchBenchmarks = async () => {
        try {
            const data = await tadpole_os_service.get_benchmarks();
            setBenchmarks(data);
        } catch (error) {
            console.error('Failed to fetch benchmarks:', error);
        } finally {
            setLoading(false);
        }
    };

    const toggleSelection = (id: string) => {
        if (selectedTests.includes(id)) {
            setSelectedTests(selectedTests.filter(t => t !== id));
        } else if (selectedTests.length < 2) {
            setSelectedTests([...selectedTests, id]);
        }
    };

    const handleRunBenchmark = async (testId: string) => {
        setRunningId(testId);
        event_bus.emit_log({
            source: 'System',
            text: i18n.t('benchmark.event_triggering', { id: testId }),
            severity: 'info'
        });

        try {
            await tadpole_os_service.run_benchmark(testId);
            event_bus.emit_log({
                source: 'System',
                text: i18n.t('benchmark.event_success', { id: testId }),
                severity: 'success'
            });
            await fetchBenchmarks();
        } catch (error) {
            console.error('Benchmark execution failed:', error);
            event_bus.emit_log({
                source: 'System',
                text: i18n.t('benchmark.event_failed', { id: testId, error: (error as Error).message || String(error) }),
                severity: 'error'
            });
        } finally {
            setRunningId(null);
        }
    };

    const getStatusIcon = (status: string) => {
        switch (status) {
            case 'PASS': return <CheckCircle2 className="text-emerald-500" size={16} />;
            case 'FAIL': return <AlertCircle className="text-rose-500" size={16} />;
            default: return <Activity className="text-amber-500" size={16} />;
        }
    };

    const calculateDelta = (v1: number, v2: number) => {
        const delta = v1 - v2;
        const percentage = ((delta / v2) * 100).toFixed(1);
        const isImprovement = delta < 0; // Lower latency is better
        return {
            value: Math.abs(delta).toFixed(2),
            percentage: Math.abs(Number(percentage)),
            isImprovement
        };
    };

    const comparisonData = selectedTests.length === 2 ? {
        t1: benchmarks.find(b => b.id === selectedTests[0]),
        t2: benchmarks.find(b => b.id === selectedTests[1]),
    } : null;

    return (
        <div className="p-8 space-y-8 min-h-screen bg-zinc-950 text-zinc-100">
            <header className="flex justify-between items-end">
                <Tooltip content={i18n.t('benchmark.tooltip_main')} position="right">
                    <div>
                        <h1 className="text-3xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-white to-zinc-500 cursor-help">
                            {i18n.t('benchmark.title')}
                        </h1>
                        <div className="mt-1">
                        </div>
                    </div>
                </Tooltip>
                <div className="flex gap-4">
                    <Tooltip content={i18n.t('benchmark.tooltip_runner')} position="bottom">
                        <button
                            onClick={() => handleRunBenchmark('BM-RUN-01')}
                            disabled={runningId !== null}
                            className="px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white font-bold rounded-lg text-[10px] transition-all flex items-center gap-2 shadow-lg shadow-blue-500/20 disabled:opacity-50 uppercase tracking-widest"
                        >
                            {runningId === 'BM-RUN-01' ? <Loader2 size={12} className="animate-spin" /> : <Zap size={12} />}
                            {runningId === 'BM-RUN-01' ? i18n.t('benchmark.btn_executing') : i18n.t('benchmark.btn_run_runner')}
                        </button>
                    </Tooltip>
                    <Tooltip content={i18n.t('benchmark.tooltip_db')} position="bottom">
                        <button
                            onClick={() => handleRunBenchmark('BM-DB-01')}
                            disabled={runningId !== null}
                            className="px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white font-bold rounded-lg text-[10px] transition-all flex items-center gap-2 shadow-lg shadow-blue-500/20 disabled:opacity-50 uppercase tracking-widest"
                        >
                            {runningId === 'BM-DB-01' ? <Loader2 size={12} className="animate-spin" /> : <BarChart3 size={12} />}
                            {runningId === 'BM-DB-01' ? i18n.t('benchmark.btn_executing') : i18n.t('benchmark.btn_run_db')}
                        </button>
                    </Tooltip>
                    <Tooltip content={i18n.t('benchmark.tooltip_rl')} position="bottom">
                        <button
                            onClick={() => handleRunBenchmark('BM-RL-01')}
                            disabled={runningId !== null}
                            className="px-4 py-2 bg-cyan-600 hover:bg-cyan-500 text-white font-bold rounded-lg text-[10px] transition-all flex items-center gap-2 shadow-lg shadow-cyan-500/20 disabled:opacity-50 uppercase tracking-widest"
                        >
                            {runningId === 'BM-RL-01' ? <Loader2 size={12} className="animate-spin" /> : <ArrowRightLeft size={12} />}
                            {runningId === 'BM-RL-01' ? i18n.t('benchmark.btn_executing') : i18n.t('benchmark.btn_run_rl')}
                        </button>
                    </Tooltip>
                    <div className="px-4 py-2 bg-emerald-500/10 border border-emerald-500/20 rounded-lg flex items-center gap-2">
                        <TrendingUp size={16} className="text-emerald-500" />
                        <span className="text-xs font-semibold text-emerald-500 uppercase tracking-tighter">{i18n.t('benchmark.status_nominal')}</span>
                    </div>
                </div>
            </header>

            {/* Comparison Tool */}
            <AnimatePresence>
                {selectedTests.length === 2 && comparisonData?.t1 && comparisonData?.t2 && (
                    <motion.div
                        initial={{ opacity: 0, y: 20 }}
                        animate={{ opacity: 1, y: 0 }}
                        exit={{ opacity: 0, y: -20 }}
                        className="grid grid-cols-1 md:grid-cols-3 gap-6 p-6 rounded-2xl bg-zinc-900/50 border border-zinc-800 backdrop-blur-xl relative overflow-hidden"
                    >
                        <div className="absolute top-0 right-0 p-8 opacity-5">
                            <ArrowRightLeft size={120} />
                        </div>

                        <div className="col-span-full mb-4 flex items-center justify-between border-b border-zinc-800 pb-4">
                            <Tooltip content={i18n.t('benchmark.tooltip_delta')} position="bottom">
                                <h2 className="text-lg font-semibold flex items-center gap-2 cursor-help">
                                    <BarChart3 className="text-blue-400" size={20} />
                                    {i18n.t('benchmark.label_delta_analysis')}
                                </h2>
                            </Tooltip>
                            <button
                                onClick={() => setSelectedTests([])}
                                className="text-xs text-zinc-500 hover:text-white transition-colors"
                            >
                                {i18n.t('benchmark.btn_clear')}
                            </button>
                        </div>

                        <div className="space-y-4">
                            <div className="text-xs uppercase tracking-widest text-zinc-500 font-bold">{i18n.t('benchmark.label_baseline')}</div>
                            <div className="p-4 rounded-xl bg-zinc-950 border border-zinc-800">
                                <div className="text-sm font-semibold truncate">{comparisonData.t2.name}</div>
                                <div className="text-2xl font-mono mt-1">{comparisonData.t2.mean_ms.toFixed(2)}ms</div>
                            </div>
                        </div>

                        <div className="flex flex-col items-center justify-center space-y-2">
                            <div className="text-xs uppercase tracking-widest text-zinc-500 font-bold">{i18n.t('benchmark.label_variance')}</div>
                            {(() => {
                                const delta = calculateDelta(comparisonData.t1.mean_ms, comparisonData.t2.mean_ms);
                                return (
                                    <div className={`text-3xl font-mono ${delta.isImprovement ? 'text-emerald-400' : 'text-rose-400'}`}>
                                        {delta.isImprovement ? '-' : '+'}{delta.percentage}%
                                    </div>
                                );
                            })()}
                            <div className="text-[10px] text-zinc-600 uppercase font-mono tracking-tighter text-center">
                                {i18n.t('benchmark.label_latency_delta')}
                            </div>
                        </div>

                        <div className="space-y-4">
                            <div className="text-xs uppercase tracking-widest text-zinc-500 font-bold">{i18n.t('benchmark.label_current_target')}</div>
                            <div className="p-4 rounded-xl bg-blue-500/10 border border-blue-500/20">
                                <div className="text-sm font-semibold truncate">{comparisonData.t1.name}</div>
                                <div className="text-2xl font-mono mt-1">{comparisonData.t1.mean_ms.toFixed(2)}ms</div>
                            </div>
                        </div>
                    </motion.div>
                )}
            </AnimatePresence>

            {/* Benchmark List */}
            <div className="rounded-2xl bg-zinc-900/30 border border-zinc-800/50 overflow-hidden">
                <div className="p-6 border-b border-zinc-800 flex items-center justify-between bg-zinc-900/20">
                    <Tooltip content={i18n.t('benchmark.tooltip_compare')} position="left">
                        <h2 className="text-lg font-semibold flex items-center gap-2 cursor-help">
                            <History size={20} className="text-zinc-400" />
                            {i18n.t('benchmark.label_historical_runs')}
                        </h2>
                    </Tooltip>
                    <div className="text-[10px] uppercase tracking-widest text-zinc-500 font-mono">
                        {i18n.t('benchmark.label_select_compare')}
                    </div>
                </div>

                <div className="overflow-x-auto">
                    <table className="w-full text-left">
                        <thead>
                            <tr className="text-[10px] uppercase tracking-widest text-zinc-500 font-bold border-b border-zinc-800">
                                <th className="px-6 py-4">{i18n.t('benchmark.header_status')}</th>
                                <th className="px-6 py-4">{i18n.t('benchmark.header_test_id')}</th>
                                <th className="px-6 py-4">{i18n.t('benchmark.header_mean')}</th>
                                <th className="px-6 py-4">{i18n.t('benchmark.header_p95_p99')}</th>
                                <th className="px-6 py-4">{i18n.t('benchmark.header_target')}</th>
                                <th className="px-6 py-4">{i18n.t('benchmark.header_timestamp')}</th>
                                <th className="px-6 py-4 w-10">{i18n.t('benchmark.header_select')}</th>
                            </tr>
                        </thead>
                        <tbody className="divide-y divide-zinc-800/50">
                            {loading ? (
                                <tr>
                                    <td colSpan={7} className="px-6 py-12 text-center text-zinc-500 animate-pulse font-mono uppercase tracking-widest text-xs">
                                        {i18n.t('benchmark.loading')}
                                    </td>
                                </tr>
                            ) : benchmarks.length === 0 ? (
                                <tr>
                                    <td colSpan={7} className="px-6 py-12 text-center text-zinc-500 font-mono uppercase tracking-widest text-xs">
                                        {i18n.t('benchmark.empty')}
                                    </td>
                                </tr>
                            ) : benchmarks.map((bench) => (
                                <motion.tr
                                    key={bench.id.toString()}
                                    whileHover={{ backgroundColor: 'rgba(255,255,255,0.02)' }}
                                    className={`group cursor-pointer transition-colors ${selectedTests.includes(bench.id.toString()) ? 'bg-blue-500/5' : ''}`}
                                    onClick={() => toggleSelection(bench.id.toString())}
                                >
                                    <td className="px-6 py-4">
                                        {getStatusIcon(bench.status)}
                                    </td>
                                    <td className="px-6 py-4">
                                        <div className="font-semibold text-sm">{bench.name}</div>
                                        <div className="text-[10px] font-mono text-zinc-500 uppercase tracking-tighter flex items-center gap-1 mt-0.5">
                                            <span className="px-1.5 py-0.5 rounded bg-zinc-800">{bench.category}</span>
                                            <ChevronRight size={10} className="text-zinc-700" />
                                            <span>{bench.test_id}</span>
                                        </div>
                                    </td>
                                    <td className="px-6 py-4 font-mono text-sm tracking-tighter">
                                        {bench.mean_ms.toFixed(2)}ms
                                    </td>
                                    <td className="px-6 py-4 font-mono text-[10px] text-zinc-400">
                                        {bench.p95_ms?.toFixed(2) || '—'} / {bench.p99_ms?.toFixed(2) || '—'}
                                    </td>
                                    <td className="px-6 py-4">
                                        <div className="text-[10px] text-zinc-500 font-mono italic max-w-[200px] truncate" title={bench.target_value}>
                                            {bench.target_value || i18n.t('benchmark.label_no_target')}
                                        </div>
                                    </td>
                                    <td className="px-6 py-4 text-xs text-zinc-500">
                                        {new Date(bench.created_at).toLocaleString()}
                                    </td>
                                    <td className="px-6 py-4">
                                        <div className={`w-4 h-4 rounded border transition-all ${selectedTests.includes(bench.id.toString())
                                            ? 'border-blue-500 bg-blue-500 shadow-[0_0_8px_rgba(59,130,246,0.5)]'
                                            : 'border-zinc-700 bg-zinc-950 group-hover:border-zinc-500'
                                            }`} />
                                    </td>
                                </motion.tr>
                            ))}
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    );
};

export default Benchmark_Analytics;


