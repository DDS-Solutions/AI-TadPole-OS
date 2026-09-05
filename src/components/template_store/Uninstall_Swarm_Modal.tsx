/**
 * @docs ARCHITECTURE:UI-Components
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Template_Store / Uninstall_Swarm_Modal
 * - **Primary Entrypoints**: `Uninstall_Swarm_Modal`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 * - `[Structural]` Dialog conforms to WAI-ARIA modal dialog semantics with keyboard navigation and focus restoration.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: `Uninstall_Swarm_Modal.test.tsx`
 */

import { useState, useEffect, useRef, useCallback } from 'react';
import { createPortal } from 'react-dom';
import { Trash2, AlertTriangle, Archive, X, ArrowLeft, Bot, FileText, Wrench, Server } from 'lucide-react';
import type { InstalledSwarmSummary } from './types';
import { i18n } from '../../i18n';

function t_or(key: string, fallback: string): string {
    const val = i18n.t(key);
    return val && val.trim() !== '' ? val : fallback;
}

interface UninstallSwarmModalProps {
    swarm: InstalledSwarmSummary | null;
    isOpen?: boolean;
    isUninstalling?: boolean;
    error?: string | null;
    onClose: () => void;
    onConfirm: (swarmId: string, archive: boolean) => Promise<void> | void;
}

export function Uninstall_Swarm_Modal({
    swarm,
    isOpen = true,
    isUninstalling = false,
    error = null,
    onClose,
    onConfirm
}: UninstallSwarmModalProps) {
    const [archive, setArchive] = useState(true);
    const [prevSwarmId, setPrevSwarmId] = useState(swarm?.id);

    // Reset archive state on swarm change (safe default)
    if (swarm?.id !== prevSwarmId) {
        setPrevSwarmId(swarm?.id);
        setArchive(true);
    }

    const dialogRef = useRef<HTMLDivElement>(null);
    const cancelBtnRef = useRef<HTMLButtonElement>(null);
    const inFlightRef = useRef(false);
    const previousActiveElementRef = useRef<Element | null>(null);

    // Track previously active element and restore focus on unmount
    useEffect(() => {
        if (isOpen && swarm) {
            previousActiveElementRef.current = document.activeElement;
            // Focus cancel button (safe default rather than destructive confirm button)
            const timer = setTimeout(() => {
                cancelBtnRef.current?.focus();
            }, 50);

            return () => {
                clearTimeout(timer);
                if (previousActiveElementRef.current instanceof HTMLElement) {
                    previousActiveElementRef.current.focus();
                }
            };
        }
    }, [isOpen, swarm]);

    // Handle Escape key and focus trapping
    useEffect(() => {
        if (!isOpen || !swarm) return;

        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === 'Escape') {
                if (!isUninstalling) {
                    e.preventDefault();
                    onClose();
                }
                return;
            }

            // Focus trap Tab cycling
            if (e.key === 'Tab' && dialogRef.current) {
                const focusableElements = dialogRef.current.querySelectorAll<HTMLElement>(
                    'button:not([disabled]), [tabindex]:not([tabindex="-1"]):not([disabled]), input:not([disabled])'
                );
                if (focusableElements.length === 0) return;

                const firstElement = focusableElements[0];
                const lastElement = focusableElements[focusableElements.length - 1];

                if (e.shiftKey) {
                    if (document.activeElement === firstElement) {
                        e.preventDefault();
                        lastElement.focus();
                    }
                } else {
                    if (document.activeElement === lastElement) {
                        e.preventDefault();
                        firstElement.focus();
                    }
                }
            }
        };

        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [isOpen, swarm, isUninstalling, onClose]);

    const handleConfirm = useCallback(async () => {
        if (inFlightRef.current || isUninstalling || !swarm) return;
        inFlightRef.current = true;
        try {
            await onConfirm(swarm.id, archive);
        } finally {
            inFlightRef.current = false;
        }
    }, [isUninstalling, swarm, onConfirm, archive]);

    if (!isOpen || !swarm) {
        return null;
    }

    const agentsCount = swarm.agents?.length || 0;
    const workflowsCount = swarm.workflows?.length || 0;
    const skillsCount = swarm.skills?.length || 0;
    const mcpCount = swarm.mcp_servers?.length || 0;
    const hasSkills = skillsCount > 0;
    const gridColsClass = hasSkills ? 'grid-cols-4' : 'grid-cols-3';

    const modalContent = (
        <div
            className="fixed inset-0 z-50 flex items-center justify-center bg-zinc-950/80 backdrop-blur-sm p-4 animate-in fade-in duration-200"
            role="presentation"
        >
            <div
                ref={dialogRef}
                role="dialog"
                aria-modal="true"
                aria-labelledby="uninstall-title"
                className="bg-[color:var(--color-background)] border border-[color:var(--color-border)] rounded-2xl w-full max-w-lg shadow-2xl overflow-hidden relative"
            >
                <button
                    onClick={onClose}
                    disabled={isUninstalling}
                    aria-label={t_or('template_store.close_modal', 'Close modal')}
                    className="absolute top-4 right-4 text-zinc-500 hover:text-zinc-100 transition-colors p-2 hover:bg-[color:var(--color-surface)] rounded-full z-10 disabled:opacity-40"
                >
                    <X size={20} />
                </button>

                {/* Header */}
                <div className="p-6 border-b border-[color:var(--color-border)]/50">
                    <div className="flex items-center gap-2 text-xs font-bold text-red-400 uppercase tracking-widest mb-1">
                        <Trash2 size={14} />
                        {t_or('template_store.uninstall_badge', 'Uninstall Swarm')}
                    </div>
                    <h3 id="uninstall-title" className="text-xl font-bold text-white tracking-tight">
                        {t_or('template_store.uninstall_title', 'Uninstall Swarm')}: {swarm.name}
                    </h3>
                </div>

                {/* Content */}
                <div className="p-6 space-y-5">
                    {/* Error Banner */}
                    {error && (
                        <div className="p-3.5 bg-red-500/20 border border-red-500/40 rounded-xl text-xs font-mono text-red-200 flex items-center gap-2 animate-in fade-in">
                            <AlertTriangle size={16} className="text-red-400 shrink-0" />
                            <span>{error}</span>
                        </div>
                    )}

                    {/* Impact Notice */}
                    <div className="p-4 bg-red-500/10 border border-red-500/20 rounded-xl space-y-2">
                        <div className="flex items-center gap-2 text-xs font-bold text-red-300">
                            <AlertTriangle size={16} className="text-red-400 shrink-0" />
                            {t_or('template_store.uninstall_impact_title', 'Deactivation Notice')}
                        </div>
                        <p className="text-xs text-zinc-300 leading-relaxed">
                            {t_or(
                                'template_store.uninstall_impact_desc',
                                `This action will de-register and deactivate ${agentsCount} agent(s), ${workflowsCount} workflow directive(s)${hasSkills ? `, ${skillsCount} skill(s)` : ''}, and prune ${mcpCount} MCP server entry(ies) from .agent/mcp_config.json.`
                            )}
                        </p>
                    </div>

                    {/* Breakdown */}
                    <div className={`grid ${gridColsClass} gap-2 text-center text-xs`}>
                        <div className="p-2.5 bg-zinc-900/60 border border-zinc-800 rounded-lg">
                            <div className="flex items-center justify-center gap-1 text-[10px] text-zinc-500 mb-0.5">
                                <Bot size={11} />
                                <span>{t_or('template_store.asset_agents', 'Agents')}</span>
                            </div>
                            <span className="font-bold text-white font-mono text-sm">{agentsCount}</span>
                        </div>

                        <div className="p-2.5 bg-zinc-900/60 border border-zinc-800 rounded-lg">
                            <div className="flex items-center justify-center gap-1 text-[10px] text-zinc-500 mb-0.5">
                                <FileText size={11} />
                                <span>{t_or('template_store.asset_workflows', 'Workflows')}</span>
                            </div>
                            <span className="font-bold text-white font-mono text-sm">{workflowsCount}</span>
                        </div>

                        {hasSkills && (
                            <div className="p-2.5 bg-zinc-900/60 border border-zinc-800 rounded-lg">
                                <div className="flex items-center justify-center gap-1 text-[10px] text-zinc-500 mb-0.5">
                                    <Wrench size={11} />
                                    <span>{t_or('template_store.asset_skills', 'Skills')}</span>
                                </div>
                                <span className="font-bold text-white font-mono text-sm">{skillsCount}</span>
                            </div>
                        )}

                        <div className="p-2.5 bg-zinc-900/60 border border-zinc-800 rounded-lg">
                            <div className="flex items-center justify-center gap-1 text-[10px] text-zinc-500 mb-0.5">
                                <Server size={11} />
                                <span>{t_or('template_store.asset_mcps', 'MCPs')}</span>
                            </div>
                            <span className="font-bold text-white font-mono text-sm">{mcpCount}</span>
                        </div>
                    </div>

                    {/* Archive Option */}
                    <label className={`flex items-start gap-3 p-3.5 bg-[color:var(--color-surface)]/40 border border-[color:var(--color-border)]/50 rounded-xl transition-colors ${isUninstalling ? 'opacity-60 cursor-not-allowed' : 'cursor-pointer hover:bg-[color:var(--color-surface)]/60'}`}>
                        <input
                            type="checkbox"
                            checked={archive}
                            disabled={isUninstalling}
                            onChange={e => setArchive(e.target.checked)}
                            className="mt-0.5 rounded border-zinc-700 text-emerald-500 focus:ring-emerald-500 bg-zinc-900"
                        />
                        <div className="space-y-0.5">
                            <span className="text-xs font-bold text-zinc-200 flex items-center gap-1.5">
                                <Archive size={13} className="text-emerald-400" />
                                {t_or('template_store.archive_checkbox', 'Preserve Configuration in Archive Vault')}
                            </span>
                            <p className="text-[11px] text-zinc-400 leading-normal">
                                {t_or(
                                    'template_store.archive_desc',
                                    'Saves agent profiles, workflow directives, skills, and swarm manifests to data/swarm_config/archive/ before removing from the active registry.'
                                )}
                            </p>
                        </div>
                    </label>
                </div>

                {/* Footer */}
                <div className="p-4 bg-[color:var(--color-surface)]/30 border-t border-[color:var(--color-border)]/50 flex items-center justify-between gap-3">
                    <button
                        ref={cancelBtnRef}
                        type="button"
                        onClick={onClose}
                        disabled={isUninstalling}
                        className="px-4 py-2 rounded-lg font-medium text-xs text-zinc-400 hover:text-white hover:bg-[color:var(--color-surface)] transition-all flex items-center gap-1.5 disabled:opacity-40"
                    >
                        <ArrowLeft size={14} />
                        {t_or('template_store.btn_cancel', 'Cancel')}
                    </button>

                    <button
                        type="button"
                        disabled={isUninstalling}
                        onClick={handleConfirm}
                        className="px-5 py-2 rounded-lg font-bold text-xs bg-red-600 hover:bg-red-500 text-white shadow-lg shadow-red-500/20 transition-all flex items-center gap-2 disabled:opacity-50"
                    >
                        {isUninstalling ? (
                            <>
                                <div className="w-3.5 h-3.5 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
                                {t_or('template_store.uninstalling_label', 'Uninstalling...')}
                            </>
                        ) : (
                            <>
                                <Trash2 size={14} />
                                {t_or('template_store.btn_confirm_uninstall', 'Confirm Uninstall')}
                            </>
                        )}
                    </button>
                </div>
            </div>
        </div>
    );

    return typeof document !== 'undefined' ? createPortal(modalContent, document.body) : modalContent;
}

// Metadata: [Template_Store]
