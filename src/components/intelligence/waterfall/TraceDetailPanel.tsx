/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Intelligence / TraceDetailPanel
 * - **Primary Entrypoints**: `Trace_Detail_Panel`, `LocalErrorBoundary`, `TraceDetailPanelProps`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: `[LocalErrorBoundary]`
 */

import React, { useMemo, useState, useEffect } from 'react';
import { Terminal, ExternalLink, X, Clock, Activity, CheckCircle2, AlertTriangle, Copy, Check } from 'lucide-react';
import clsx from 'clsx';
import { type Trace_Node, get_span_error_details } from '../../../types';
import { TickerContext } from './WaterfallTicker';

// PII & Secrets Redaction (SEC-002 / SEC-004)
const SENSITIVE_KEYS = /token|api_key|secret|password|authorization|private_key|credential/i;
const SENSITIVE_VALUES_REGEX = /bearer\s+[a-zA-Z0-9_\-.]+|ey[a-zA-Z0-9_\-.]+\.ey[a-zA-Z0-9_\-.]+\.[a-zA-Z0-9_\-.]+|ghp_[a-zA-Z0-9]+|sk_live_[a-zA-Z0-9]+/i;

const redact_attributes = (attributes: Record<string, string | number | boolean>) => {
    const redacted: Record<string, string | number | boolean> = {};
    for (const [key, value] of Object.entries(attributes || {})) {
        const is_sensitive_key = SENSITIVE_KEYS.test(key);
        const is_sensitive_value = typeof value === 'string' && SENSITIVE_VALUES_REGEX.test(value);
        if (is_sensitive_key || is_sensitive_value) {
            redacted[key] = '[REDACTED]';
        } else {
            redacted[key] = value;
        }
    }
    return redacted;
};

// Failsafe Error Boundary (Reliability / Clock Skew)
export class LocalErrorBoundary extends React.Component<{ children: React.ReactNode; fallbackMessage?: string }, { hasError: boolean }> {
    constructor(props: { children: React.ReactNode; fallbackMessage?: string }) {
        super(props);
        this.state = { hasError: false };
    }

    static getDerivedStateFromError() {
        return { hasError: true };
    }

    componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
        console.error("[LocalErrorBoundary] Observability component crashed:", error, errorInfo);
    }

    render() {
        if (this.state.hasError) {
            return (
                <div className="p-4 bg-red-950/20 border border-red-500/20 text-red-400 font-mono text-xs rounded-lg m-4">
                    ⚠️ {this.props.fallbackMessage || "Observability Link Degraded: Trace rendering failure."}
                </div>
            );
        }
        return this.props.children;
    }
}

export interface TraceDetailPanelProps {
    span: Trace_Node;
    agent_name: string;
    is_detached: boolean;
    on_close: () => void;
    on_detach?: () => void;
}

export const Trace_Detail_Panel: React.FC<TraceDetailPanelProps> = ({
    span,
    agent_name,
    is_detached,
    on_close,
    on_detach
}) => {
    const is_running = !span.end_time;
    const [local_now, set_local_now] = useState(() => Date.now());
    const [has_copied_error, set_has_copied_error] = useState(false);
    const ticker = React.useContext(TickerContext);

    useEffect(() => {
        if (!is_running || !ticker) return;
        return ticker.subscribe(set_local_now);
    }, [is_running, ticker]);

    const duration = is_running ? (local_now - span.start_time) : (span.end_time! - span.start_time);
    const sanitized_attributes = useMemo(() => redact_attributes(span.attributes || {}), [span.attributes]);
    const error_details = useMemo(() => get_span_error_details(span), [span]);
    const is_error = span.status === 'error' || Boolean(error_details.error || error_details.reason || (error_details.http_status && error_details.http_status >= 400));

    const handle_copy_error = async () => {
        const error_payload = {
            span_id: span.id,
            trace_id: span.trace_id,
            span_name: span.name,
            agent_id: span.agent_id,
            agent_name,
            status: span.status,
            http_status: error_details.http_status,
            error: error_details.error,
            reason: error_details.reason,
            stack: error_details.stack,
            attributes: sanitized_attributes
        };
        try {
            await navigator.clipboard.writeText(JSON.stringify(error_payload, null, 2));
            set_has_copied_error(true);
            setTimeout(() => set_has_copied_error(false), 2000);
        } catch {
            set_has_copied_error(true);
            setTimeout(() => set_has_copied_error(false), 2000);
        }
    };

    return (
        <div className={clsx(
            "flex flex-col h-full bg-zinc-950 font-sans border-[color:var(--color-border)] select-none",
            is_detached ? "w-full p-6" : "w-80 border-l border-zinc-800 bg-zinc-900/60 backdrop-blur-md p-4 animate-in slide-in-from-right duration-200"
        )}>
            {/* Header */}
            <div className="flex items-center justify-between border-b border-white/5 pb-3 mb-4">
                <h4 className="text-xs font-bold text-zinc-200 uppercase tracking-wider truncate flex items-center gap-1.5">
                    <Terminal size={12} strokeWidth={1.5} className="text-cyan-500" />
                    {span.name}
                </h4>
                <div className="flex items-center gap-2">
                    {on_detach && !is_detached && (
                        <button
                            onClick={on_detach}
                            className="p-1 hover:bg-zinc-800 rounded-md text-zinc-500 hover:text-zinc-200 transition-colors"
                            title="Detach Details Window"
                        >
                            <ExternalLink size={14} strokeWidth={1.5} />
                        </button>
                    )}
                    <button
                        onClick={on_close}
                        className="p-1 hover:bg-zinc-800 rounded-md text-zinc-500 hover:text-zinc-200 transition-colors"
                        title="Close details"
                    >
                        <X size={14} strokeWidth={1.5} />
                    </button>
                </div>
            </div>

            {/* Content Area */}
            <div className="flex-1 overflow-y-auto space-y-4 pr-1 custom-scrollbar">
                {/* Prominent Error Banner */}
                {is_error && (
                    <div 
                        data-testid="trace-error-banner"
                        className="bg-red-950/40 border border-red-500/40 rounded-xl p-3 text-red-200 space-y-2 shadow-[0_0_15px_rgba(239,68,68,0.15)] animate-in fade-in duration-200"
                    >
                        <div className="flex items-center justify-between">
                            <div className="flex items-center gap-1.5 text-red-400 font-bold text-xs uppercase tracking-wider">
                                <AlertTriangle size={14} strokeWidth={2} className="text-red-400 shrink-0" />
                                <span>Execution Fault {error_details.http_status ? `(HTTP ${error_details.http_status})` : ''}</span>
                            </div>
                            <button
                                type="button"
                                onClick={handle_copy_error}
                                aria-label="Copy error details"
                                className="flex items-center gap-1 text-[9px] font-mono px-2 py-1 rounded bg-red-900/40 hover:bg-red-800/60 border border-red-700/50 text-red-200 transition-all cursor-pointer active:scale-95"
                            >
                                {has_copied_error ? (
                                    <>
                                        <Check size={10} className="text-emerald-400" />
                                        <span className="text-emerald-300">Copied</span>
                                    </>
                                ) : (
                                    <>
                                        <Copy size={10} />
                                        <span>Copy Details</span>
                                    </>
                                )}
                            </button>
                        </div>

                        <p className="text-[11px] font-mono font-semibold text-red-200 break-words leading-relaxed">
                            {error_details.error || (error_details.http_status ? `HTTP ${error_details.http_status} Server Fault` : 'Span terminated with error status')}
                        </p>

                        {error_details.reason && (
                            <p className="text-[9px] font-mono text-red-300/90 bg-red-950/80 p-2 rounded border border-red-900/50 leading-normal">
                                {error_details.reason}
                            </p>
                        )}

                        {error_details.stack && (
                            <div className="space-y-1">
                                <span className="text-[8px] font-mono font-bold text-red-400/80 uppercase tracking-widest block">Stack Trace</span>
                                <pre className="text-[8px] font-mono text-red-200/90 bg-zinc-950 p-2 rounded border border-red-900/40 overflow-x-auto max-h-32 whitespace-pre-wrap">
                                    {error_details.stack}
                                </pre>
                            </div>
                        )}
                    </div>
                )}

                {/* Status and Duration */}
                <div className="grid grid-cols-2 gap-3">
                    <div className="bg-zinc-900 border border-white/5 rounded-lg p-2.5">
                        <span className="text-[8px] font-bold text-zinc-500 uppercase tracking-widest block mb-1">Status</span>
                        <div className="flex items-center gap-1.5">
                            {span.status === 'success' && <CheckCircle2 size={12} strokeWidth={1.5} className="text-emerald-500" />}
                            {span.status === 'error' && <AlertTriangle size={12} strokeWidth={1.5} className="text-red-500" />}
                            {span.status === 'running' && <Activity size={12} strokeWidth={1.5} className="text-cyan-500 animate-pulse" />}
                            <span className={clsx(
                                "text-[10px] font-bold uppercase tracking-wider",
                                span.status === 'success' && "text-emerald-400",
                                span.status === 'error' && "text-red-400",
                                span.status === 'running' && "text-cyan-400"
                            )}>
                                {span.status}
                            </span>
                        </div>
                    </div>

                    <div className="bg-zinc-900 border border-white/5 rounded-lg p-2.5">
                        <span className="text-[8px] font-bold text-zinc-500 uppercase tracking-widest block mb-1">Elapsed Time</span>
                        <div className="flex items-center gap-1.5 font-mono text-[10px] text-zinc-300 font-bold">
                            <Clock size={12} strokeWidth={1.5} className="text-zinc-500" />
                            {Math.max(0, duration)}ms
                        </div>
                    </div>
                </div>

                {/* Agent Assignment */}
                <div className="bg-zinc-900 border border-white/5 rounded-lg p-3">
                    <span className="text-[8px] font-bold text-zinc-500 uppercase tracking-widest block mb-2">Executing Agent</span>
                    <div className="flex items-center gap-3">
                        <div className="w-8 h-8 rounded-lg bg-zinc-800 border border-zinc-700 flex items-center justify-center font-bold text-xs text-cyan-400">
                            {agent_name[0] || '?'}
                        </div>
                        <div className="flex flex-col">
                            <span className="text-xs font-bold text-zinc-200">{agent_name}</span>
                            <span className="text-[9px] font-mono text-zinc-600 uppercase tracking-tighter block mt-0.5">ID: {span.agent_id}</span>
                        </div>
                    </div>
                </div>

                {/* Attributes Details */}
                <div className="space-y-2">
                    <span className="text-[8px] font-bold text-zinc-500 uppercase tracking-widest block">Span Attributes</span>
                    {Object.keys(sanitized_attributes).length === 0 ? (
                        <span className="text-[10px] text-zinc-600 italic font-mono block pl-1">No metadata attributes recorded.</span>
                    ) : (
                        <div className="space-y-2">
                            {Object.entries(sanitized_attributes).map(([key, value]) => (
                                <div key={key} className="bg-zinc-900/50 border border-white/5 rounded-lg p-2.5 font-mono">
                                    <span className="text-[8px] text-zinc-500 font-bold block mb-1 truncate" title={key}>{key}</span>
                                    <pre className="text-[10px] text-zinc-300 font-mono whitespace-pre-wrap break-all overflow-x-auto bg-zinc-950/30 p-1.5 rounded border border-white/5">
                                        {typeof value === 'object' ? JSON.stringify(value, null, 2) : String(value)}
                                    </pre>
                                </div>
                            ))}
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
};

// Metadata: [TraceDetailPanel]
// Telemetry: [Neural_Waterfall]
