/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Missions / History / Merkle_Audit_Viewer
 * - **Primary Entrypoints**: `Merkle_Audit_Viewer`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Step timeline, SHA-256 hash badges in JetBrains Mono, click-to-copy, ALCOA+ proof validation.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import React, { useState, useMemo } from 'react';
import { 
    ShieldCheck, 
    ShieldAlert, 
    AlertCircle, 
    Copy, 
    Check, 
    Search, 
    Code, 
    ChevronDown, 
    ChevronUp,
    Terminal,
    Bot,
    User,
    Info,
    AlertTriangle,
    XCircle
} from 'lucide-react';
import type { MerkleStepRecord } from '../../../types/mission_dossier';
import { Tooltip } from '../../ui';

interface MerkleAuditViewerProps {
    logs: MerkleStepRecord[];
}

export const Merkle_Audit_Viewer: React.FC<MerkleAuditViewerProps> = ({ logs }) => {
    const [search_text, set_search_text] = useState('');
    const [severity_filter, set_severity_filter] = useState<string>('all');
    const [source_filter, set_source_filter] = useState<string>('all');
    const [copied_hash, set_copied_hash] = useState<string | null>(null);
    const [expanded_meta, set_expanded_meta] = useState<Record<string, boolean>>({});

    const handle_copy = async (text: string, id: string) => {
        try {
            await navigator.clipboard.writeText(text);
            set_copied_hash(id);
            setTimeout(() => set_copied_hash(null), 2000);
        } catch (err) {
            console.warn('Failed to copy hash to clipboard:', err);
        }
    };

    const toggle_meta = (id: string) => {
        set_expanded_meta((prev) => ({ ...prev, [id]: !prev[id] }));
    };

    // Maintain true chronological step indexes from source flight ledger
    const original_index_map = useMemo(() => {
        const map = new Map<string, number>();
        logs.forEach((log, idx) => {
            map.set(log.id, log.step_index ?? idx + 1);
        });
        return map;
    }, [logs]);

    const filtered_logs = useMemo(() => {
        return logs.filter((log) => {
            if (severity_filter !== 'all' && log.severity.toLowerCase() !== severity_filter) {
                return false;
            }
            if (source_filter !== 'all' && log.source.toLowerCase() !== source_filter) {
                return false;
            }
            if (search_text.trim()) {
                const q = search_text.toLowerCase();
                const text_match = log.text.toLowerCase().includes(q);
                const hash_match = log.hash?.toLowerCase().includes(q) ?? false;
                const agent_match = log.agent_id.toLowerCase().includes(q);
                if (!text_match && !hash_match && !agent_match) return false;
            }
            return true;
        });
    }, [logs, severity_filter, source_filter, search_text]);

    const render_source_icon = (source: string) => {
        const s = source.toLowerCase();
        if (s.includes('agent')) return <Bot size={13} className="text-cyan-400" />;
        if (s.includes('user')) return <User size={13} className="text-purple-400" />;
        return <Terminal size={13} className="text-zinc-400" />;
    };

    const render_severity_pill = (sev: string) => {
        const s = sev.toLowerCase();
        if (s === 'error') {
            return (
                <span className="inline-flex items-center gap-1 text-[10px] font-bold text-rose-400 bg-rose-500/10 px-1.5 py-0.5 rounded border border-rose-500/20">
                    <XCircle size={10} /> ERR
                </span>
            );
        }
        if (s === 'warning') {
            return (
                <span className="inline-flex items-center gap-1 text-[10px] font-bold text-amber-400 bg-amber-500/10 px-1.5 py-0.5 rounded border border-amber-500/20">
                    <AlertTriangle size={10} /> WARN
                </span>
            );
        }
        if (s === 'success') {
            return (
                <span className="inline-flex items-center gap-1 text-[10px] font-bold text-emerald-400 bg-emerald-500/10 px-1.5 py-0.5 rounded border border-emerald-500/20">
                    <ShieldCheck size={10} /> OK
                </span>
            );
        }
        return (
            <span className="inline-flex items-center gap-1 text-[10px] font-mono text-zinc-400 bg-zinc-800/60 px-1.5 py-0.5 rounded">
                <Info size={10} /> INFO
            </span>
        );
    };

    return (
        <div className="flex flex-col h-full bg-zinc-950 border border-zinc-800/80 rounded-xl overflow-hidden">
            {/* Filter & Search Bar */}
            <div className="p-3 border-b border-zinc-800/80 bg-zinc-900/50 flex flex-wrap items-center justify-between gap-3">
                <div className="flex items-center gap-2 flex-1 min-w-[200px]">
                    <div className="relative flex-1">
                        <Search size={14} className="absolute left-3 top-1/2 -translate-y-1/2 text-zinc-500 pointer-events-none" />
                        <input
                            type="text"
                            placeholder="Filter audit trail by text or hash..."
                            value={search_text}
                            onChange={(e) => set_search_text(e.target.value)}
                            className="w-full bg-[color:var(--color-background)] border border-zinc-800 rounded-lg pl-9 pr-3 py-1.5 text-xs text-zinc-200 placeholder-zinc-500 focus:outline-none focus:border-emerald-500/50 focus:ring-1 focus:ring-emerald-500/30 transition-all font-sans"
                        />
                    </div>
                </div>

                <div className="flex items-center gap-2">
                    {/* Severity Filter */}
                    <div className="flex items-center gap-1 bg-zinc-950/80 p-0.5 rounded-lg border border-zinc-800 text-[11px]">
                        {['all', 'info', 'warning', 'error'].map((sev) => (
                            <button
                                key={sev}
                                onClick={() => set_severity_filter(sev)}
                                className={`px-2 py-0.5 rounded font-medium transition-colors uppercase text-[10px] ${
                                    severity_filter === sev
                                        ? 'bg-zinc-800 text-zinc-100 shadow-sm'
                                        : 'text-zinc-500 hover:text-zinc-300'
                                }`}
                            >
                                {sev}
                            </button>
                        ))}
                    </div>

                    {/* Source Filter */}
                    <div className="flex items-center gap-1 bg-zinc-950/80 p-0.5 rounded-lg border border-zinc-800 text-[11px]">
                        {['all', 'agent', 'system', 'user'].map((src) => (
                            <button
                                key={src}
                                onClick={() => set_source_filter(src)}
                                className={`px-2 py-0.5 rounded font-medium transition-colors uppercase text-[10px] ${
                                    source_filter === src
                                        ? 'bg-zinc-800 text-zinc-100 shadow-sm'
                                        : 'text-zinc-500 hover:text-zinc-300'
                                }`}
                            >
                                {src}
                            </button>
                        ))}
                    </div>

                    <span className="text-[11px] font-mono text-zinc-500 pl-2">
                        {filtered_logs.length} / {logs.length} turns
                    </span>
                </div>
            </div>

            {/* Steps Timeline Container */}
            <div className="flex-1 overflow-y-auto custom-scrollbar p-4 flex flex-col gap-3">
                {filtered_logs.length === 0 ? (
                    <div className="flex flex-col items-center justify-center p-12 text-center text-zinc-500">
                        <Terminal size={32} className="text-zinc-600 mb-2" />
                        <span className="text-xs font-semibold text-zinc-400">No log entries matched criteria</span>
                    </div>
                ) : (
                    filtered_logs.map((log, index) => {
                        const has_meta = Boolean(log.metadata && Object.keys(log.metadata).length > 0);
                        const is_meta_open = Boolean(expanded_meta[log.id]);

                        return (
                            <div
                                key={log.id}
                                className={`p-3.5 rounded-xl border flex flex-col gap-2.5 transition-all ${
                                    log.severity === 'error'
                                        ? 'bg-rose-950/10 border-rose-900/40'
                                        : log.severity === 'warning'
                                        ? 'bg-amber-950/10 border-amber-900/40'
                                        : 'bg-zinc-900/40 border-zinc-800/80 hover:border-zinc-700'
                                }`}
                            >
                                {/* Step Header */}
                                <div className="flex items-center justify-between text-xs">
                                    <div className="flex items-center gap-2">
                                        <span className="font-mono text-[10px] font-bold text-zinc-500 px-1.5 py-0.5 rounded bg-zinc-950 border border-zinc-800">
                                            #{original_index_map.get(log.id) ?? index + 1}
                                        </span>
                                        <span className="inline-flex items-center gap-1 font-semibold text-zinc-300">
                                            {render_source_icon(log.source)}
                                            {log.source}
                                        </span>
                                        <span className="text-zinc-500 font-mono text-[10px]">
                                            [{log.agent_id}]
                                        </span>
                                        {render_severity_pill(log.severity)}
                                    </div>

                                    <div className="flex items-center gap-2 font-mono text-[11px] text-zinc-500">
                                        <span>{new Date(log.timestamp).toLocaleTimeString()}</span>
                                        {log.is_verified ? (
                                            <Tooltip content="ALCOA+ Cryptographic Merkle chain verified intact" position="left">
                                                <span 
                                                    className="inline-flex items-center gap-1 text-[10px] font-bold text-emerald-400 bg-emerald-500/10 px-2 py-0.5 rounded-full border border-emerald-500/20 cursor-help"
                                                >
                                                    <ShieldCheck size={11} /> Merkle Valid
                                                </span>
                                            </Tooltip>
                                        ) : log.hash === null ? (
                                            <Tooltip content="Legacy unhashed log entry without cryptographic Merkle proof" position="left">
                                                <span 
                                                    className="inline-flex items-center gap-1 text-[10px] font-bold text-amber-400 bg-amber-500/10 px-2 py-0.5 rounded-full border border-amber-500/20 cursor-help"
                                                >
                                                    <AlertCircle size={11} /> Unverified Legacy
                                                </span>
                                            </Tooltip>
                                        ) : (
                                            <Tooltip content="CRITICAL: Merkle hash-chain mismatch detected" position="left">
                                                <span 
                                                    className="inline-flex items-center gap-1 text-[10px] font-bold text-rose-400 bg-rose-500/10 px-2 py-0.5 rounded-full border border-rose-500/20 cursor-help"
                                                >
                                                    <ShieldAlert size={11} /> Tamper Flag
                                                </span>
                                            </Tooltip>
                                        )}
                                    </div>
                                </div>

                                {/* Log Text Payload */}
                                <div className="text-xs text-zinc-200 leading-relaxed font-sans whitespace-pre-wrap break-words bg-zinc-950/50 p-2.5 rounded-lg border border-zinc-900/80">
                                    {log.text}
                                </div>

                                {/* Cryptographic Merkle Hash Bar */}
                                <div className="flex flex-wrap items-center justify-between gap-2 pt-1 text-[10px] font-mono text-zinc-500 border-t border-zinc-800/40">
                                    <div className="flex items-center gap-3">
                                        {log.prev_hash && (
                                            <Tooltip content={`Previous Block Hash: ${log.prev_hash}`} position="top">
                                                <div className="flex items-center gap-1 cursor-help">
                                                    <span className="text-zinc-600">prev:</span>
                                                    <span className="text-zinc-400 bg-zinc-950 px-1.5 py-0.5 rounded border border-zinc-900 truncate max-w-[140px]">
                                                        {log.prev_hash.slice(0, 16)}…
                                                    </span>
                                                </div>
                                            </Tooltip>
                                        )}
                                        {log.hash && (
                                            <div className="flex items-center gap-1">
                                                <Tooltip content={`Block Hash: ${log.hash}`} position="top">
                                                    <div className="flex items-center gap-1 cursor-help">
                                                        <span className="text-zinc-600">hash:</span>
                                                        <span className="text-cyan-400 font-bold bg-zinc-950 px-1.5 py-0.5 rounded border border-zinc-900 truncate max-w-[140px]">
                                                            {log.hash.slice(0, 16)}…
                                                        </span>
                                                    </div>
                                                </Tooltip>
                                                <Tooltip content={copied_hash === `hash-${log.id}` ? "Copied SHA-256 hash!" : "Copy full SHA-256 hash"} position="top">
                                                    <button
                                                        onClick={() => handle_copy(log.hash!, `hash-${log.id}`)}
                                                        className="p-1 hover:text-zinc-200 transition-colors"
                                                        aria-label="Copy full SHA-256 hash"
                                                    >
                                                        {copied_hash === `hash-${log.id}` ? (
                                                            <Check size={11} className="text-emerald-400" />
                                                        ) : (
                                                            <Copy size={11} />
                                                        )}
                                                    </button>
                                                </Tooltip>
                                            </div>
                                        )}
                                    </div>

                                    {has_meta && (
                                        <button
                                            onClick={() => toggle_meta(log.id)}
                                            className="flex items-center gap-1 text-[11px] text-zinc-400 hover:text-cyan-300 font-mono transition-colors"
                                        >
                                            <Code size={11} />
                                            <span>metadata</span>
                                            {is_meta_open ? <ChevronUp size={11} /> : <ChevronDown size={11} />}
                                        </button>
                                    )}
                                </div>

                                {/* Metadata Collapsible Section */}
                                {has_meta && is_meta_open && (
                                    <div className="p-2.5 rounded-lg bg-slate-950/60 border border-zinc-800 font-mono text-[11px] text-zinc-300 overflow-x-auto max-h-48 custom-scrollbar">
                                        <pre>{JSON.stringify(log.metadata, null, 2)}</pre>
                                    </div>
                                )}
                            </div>
                        );
                    })
                )}
            </div>
        </div>
    );
};
