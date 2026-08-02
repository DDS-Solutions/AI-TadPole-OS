/**
 * @docs ARCHITECTURE:UI-Components
 * @docs OPERATIONS_MANUAL:Governance
 * 
 * ### AI Assist Note
 * **Socratic Gate Intervention Bar**: Top-level visual alert bar that renders when
 * an autonomous agent hits a Socratic Gate (critical unknown or high-risk decision).
 * Features real-time risk indicators, context questions, and interactive approval controls.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Stale gate event state or unhandled submission rejection.
 * - **Telemetry Link**: Search `[SocraticGateBar]` in UI logs.
 */

import React, { useState } from 'react';
import { AlertTriangle, ShieldAlert, CheckCircle2, XCircle, HelpCircle } from 'lucide-react';
import { motion } from 'framer-motion';
import clsx from 'clsx';
import { i18n } from '../../i18n';

export interface SocraticGatePrompt {
    id: string;
    mission_id: string;
    agent_name: string;
    question: string;
    priority: 'P0' | 'P1' | 'P2';
    options?: string[];
    timestamp: string;
}

interface SocraticGateBarProps {
    prompt: SocraticGatePrompt | null;
    onRespond?: (promptId: string, response: string) => void;
    on_respond?: (prompt_id: string, response: string) => void;
    onDismiss?: () => void;
    on_dismiss?: () => void;
}

export const SocraticGateBar: React.FC<SocraticGateBarProps> = ({
    prompt,
    onRespond,
    on_respond,
    onDismiss,
    on_dismiss
}) => {
    const [customResponse, setCustomResponse] = useState('');

    if (!prompt) return null;

    const isP0 = prompt.priority === 'P0';
    const respond_fn = onRespond || on_respond;
    const dismiss_fn = onDismiss || on_dismiss;

    const handleSubmit = (text: string) => {
        if (!text.trim() || !respond_fn) return;
        // Telemetry event log: [SocraticGateBar]
        respond_fn(prompt.id, text.trim());
        setCustomResponse('');
    };

    return (
        <motion.div
            initial={{ opacity: 0, y: -20 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -20 }}
            className={clsx(
                'w-full px-4 py-3 border-b flex flex-col md:flex-row items-center justify-between gap-4 shadow-lg backdrop-blur-md transition-colors z-40',
                isP0
                    ? 'bg-rose-950/80 border-rose-800/60 text-rose-100'
                    : 'bg-amber-950/80 border-amber-800/60 text-amber-100'
            )}
        >
            {/* Priority & Badge Section */}
            <div className="flex items-center gap-3 min-w-max">
                <motion.div
                    animate={{ scale: [1, 1.15, 1] }}
                    transition={{ repeat: Infinity, duration: 2 }}
                >
                    {isP0 ? (
                        <ShieldAlert className="w-5 h-5 text-rose-400" />
                    ) : (
                        <AlertTriangle className="w-5 h-5 text-amber-400" />
                    )}
                </motion.div>
                
                <span className={clsx(
                    'px-2 py-0.5 text-xs font-mono font-bold rounded uppercase tracking-wider',
                    isP0 ? 'bg-rose-900/80 text-rose-300 border border-rose-700' : 'bg-amber-900/80 text-amber-300 border border-amber-700'
                )}>
                    🛑 Socratic Gate [{prompt.priority}]
                </span>

                <span className="text-xs font-medium opacity-80 font-mono">
                    Agent: <strong className="text-white">{prompt.agent_name}</strong>
                </span>
            </div>

            {/* Question Content */}
            <div className="flex-1 text-sm font-medium flex items-center gap-2">
                <HelpCircle className="w-4 h-4 opacity-70 flex-shrink-0" />
                <span className="leading-tight">{prompt.question}</span>
            </div>

            {/* Response Action Controls */}
            <div className="flex items-center gap-2 w-full md:w-auto justify-end">
                {prompt.options && prompt.options.length > 0 ? (
                    <div className="flex items-center gap-2 flex-wrap">
                        {prompt.options.map((opt, idx) => (
                            <button
                                key={idx}
                                onClick={() => handleSubmit(opt)}
                                className="px-3 py-1 text-xs font-medium rounded-md bg-white/10 hover:bg-white/20 border border-white/20 transition-colors flex items-center gap-1.5 cursor-pointer"
                            >
                                <CheckCircle2 className="w-3.5 h-3.5 text-emerald-400" />
                                {opt}
                            </button>
                        ))}
                    </div>
                ) : (
                    <div className="flex items-center gap-2 w-full md:w-auto">
                        <input
                            type="text"
                            aria-label="Socratic Gate Response"
                            value={customResponse}
                            onChange={(e) => setCustomResponse(e.target.value)}
                            onKeyDown={(e) => e.key === 'Enter' && handleSubmit(customResponse)}
                            placeholder={i18n.t('oversight.socratic_placeholder') || 'Provide direction...'}
                            className="px-3 py-1 text-xs rounded bg-black/40 border border-white/20 text-white placeholder-white/40 focus:outline-none focus:border-emerald-500 w-full md:w-64 font-mono"
                        />
                        <button
                            onClick={() => handleSubmit(customResponse)}
                            disabled={!customResponse.trim()}
                            className="px-3 py-1 text-xs font-medium rounded bg-emerald-600 hover:bg-emerald-500 disabled:opacity-50 text-white transition-colors cursor-pointer"
                        >
                            Submit
                        </button>
                    </div>
                )}

                {dismiss_fn && (
                    <button
                        onClick={dismiss_fn}
                        className="p-1 rounded hover:bg-white/10 text-white/60 hover:text-white transition-colors cursor-pointer"
                        title="Dismiss"
                        aria-label="Dismiss alert"
                    >
                        <XCircle className="w-4 h-4" />
                    </button>
                )}
            </div>
        </motion.div>
    );
};

export const Socratic_Gate_Bar = SocraticGateBar;
// Metadata: [Socratic_Gate_Bar]
