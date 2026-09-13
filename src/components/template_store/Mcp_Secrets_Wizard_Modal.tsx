/**
 * @docs ARCHITECTURE:UI-Components
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Template_Store / Mcp_Secrets_Wizard_Modal
 * - **Primary Entrypoints**: `Mcp_Secrets_Wizard_Modal`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 * - `[Structural]` Mask secret inputs by default with opt-in visibility toggle.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: `Mcp_Secrets_Wizard_Modal.test.tsx`
 */

import React, { useState, useCallback } from 'react';
import { KeyRound, Eye, EyeOff, ShieldCheck, Check, ArrowRight, X } from 'lucide-react';
import type { McpPlaceholderVariable } from './types';
import { system_api_service } from '../../services/system_api_service';
import { i18n } from '../../i18n';

export interface McpSecretsWizardModalProps {
    isOpen?: boolean;
    swarmName?: string;
    placeholders: McpPlaceholderVariable[];
    isSaving?: boolean;
    onClose: () => void;
    onSave?: (variables: Record<string, string>) => Promise<void> | void;
    onSaveSuccess?: () => void;
}

export function Mcp_Secrets_Wizard_Modal({
    isOpen = true,
    swarmName,
    placeholders,
    isSaving = false,
    onClose,
    onSave,
    onSaveSuccess
}: McpSecretsWizardModalProps) {
    const [values, setValues] = useState<Record<string, string>>({});
    const [showSecrets, setShowSecrets] = useState<Record<string, boolean>>({});
    const [internalSaving, setInternalSaving] = useState(false);

    const handleInputChange = (variable: string, value: string) => {
        setValues(prev => ({ ...prev, [variable]: value }));
    };

    const toggleShowSecret = (variable: string) => {
        setShowSecrets(prev => ({ ...prev, [variable]: !prev[variable] }));
    };

    const handleSubmit = useCallback(async (e: React.FormEvent) => {
        e.preventDefault();
        const nonBlankValues: Record<string, string> = {};
        for (const [k, v] of Object.entries(values)) {
            if (v.trim()) {
                nonBlankValues[k] = v.trim();
            }
        }

        if (onSave) {
            await onSave(nonBlankValues);
            if (onSaveSuccess) onSaveSuccess();
        } else {
            try {
                setInternalSaving(true);
                await system_api_service.engine.update_environment(nonBlankValues);
                if (onSaveSuccess) onSaveSuccess();
                onClose();
            } catch (err) {
                console.error('[Mcp_Secrets_Wizard_Modal] Failed to save environment variables:', err);
            } finally {
                setInternalSaving(false);
            }
        }
    }, [values, onSave, onSaveSuccess, onClose]);

    if (!isOpen || placeholders.length === 0) {
        return null;
    }

    const saving = isSaving || internalSaving;

    // Group by server name
    const groupedByServer = placeholders.reduce<Record<string, McpPlaceholderVariable[]>>((acc, p) => {
        if (!acc[p.server]) acc[p.server] = [];
        acc[p.server].push(p);
        return acc;
    }, {});

    const titleText = swarmName
        ? `Configure Secrets for ${swarmName}`
        : 'Connect External Tools & Services';

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-zinc-950/80 backdrop-blur-sm p-4 animate-in fade-in duration-200">
            <div className="bg-[color:var(--color-background)] border border-[color:var(--color-border)] rounded-2xl w-full max-w-2xl max-h-[90vh] flex flex-col shadow-2xl overflow-hidden relative">
                <button
                    onClick={onClose}
                    aria-label="Close MCP secrets wizard"
                    className="absolute top-4 right-4 text-zinc-500 hover:text-zinc-100 transition-colors p-2 hover:bg-[color:var(--color-surface)] rounded-full z-10"
                >
                    <X size={20} />
                </button>

                {/* Header */}
                <div className="p-6 border-b border-[color:var(--color-border)]/50">
                    <div className="flex items-center gap-2 mb-2">
                        <span className="p-1.5 bg-amber-500/10 text-amber-400 rounded-lg border border-amber-500/20">
                            <KeyRound size={16} />
                        </span>
                        <span className="text-[10px] uppercase tracking-wider font-bold text-amber-400">
                            {i18n.t('template_store.mcp_wizard_badge') || 'Secure Connector Setup'}
                        </span>
                    </div>
                    <h2 className="text-xl font-bold text-white tracking-tight">
                        {titleText}
                    </h2>
                    <p className="text-xs text-zinc-400 mt-1 leading-relaxed">
                        {i18n.t('template_store.mcp_wizard_desc') ||
                            'This swarm declares external tool connectors. Enter your API credentials to store them securely in the environment vault now, or skip to configure later in Settings.'}
                    </p>
                </div>

                {/* Form Content */}
                <form onSubmit={handleSubmit} className="flex-1 overflow-y-auto p-6 space-y-6 custom-scrollbar">
                    {Object.entries(groupedByServer).map(([serverName, vars]) => (
                        <div
                            key={serverName}
                            className="bg-[color:var(--color-surface)]/40 border border-[color:var(--color-border)]/50 rounded-xl p-4 space-y-4"
                        >
                            <div className="flex items-center justify-between">
                                <h3 className="text-sm font-bold text-zinc-200 flex items-center gap-2">
                                    <ShieldCheck size={16} className="text-emerald-400" />
                                    {serverName}
                                </h3>
                                <span className="text-[10px] uppercase font-mono px-2 py-0.5 rounded bg-zinc-800 text-zinc-400">
                                    {vars.length} {vars.length === 1 ? 'Secret Required' : 'Secrets Required'}
                                </span>
                            </div>

                            <div className="space-y-3">
                                {vars.map(p => (
                                    <div key={p.variable} className="space-y-1">
                                        <label
                                            htmlFor={`mcp-var-${p.variable}`}
                                            className="text-xs font-mono font-medium text-zinc-300 flex items-center justify-between"
                                        >
                                            <span>{p.variable}</span>
                                            {p.description && (
                                                <span className="text-[10px] text-zinc-500 font-sans">{p.description}</span>
                                            )}
                                        </label>
                                        <div className="relative flex items-center">
                                            <input
                                                id={`mcp-var-${p.variable}`}
                                                type={showSecrets[p.variable] ? 'text' : 'password'}
                                                placeholder={`Enter ${p.variable}...`}
                                                value={values[p.variable] || ''}
                                                onChange={e => handleInputChange(p.variable, e.target.value)}
                                                className="w-full bg-zinc-950/70 border border-zinc-700/60 rounded-lg px-3 py-2 pr-10 text-xs font-mono text-white placeholder-zinc-600 focus:outline-none focus:border-emerald-500 transition-colors"
                                            />
                                            <button
                                                type="button"
                                                onClick={() => toggleShowSecret(p.variable)}
                                                aria-label={showSecrets[p.variable] ? 'Hide secret' : 'Reveal secret'}
                                                className="absolute right-2 text-zinc-500 hover:text-zinc-300 p-1 rounded transition-colors"
                                            >
                                                {showSecrets[p.variable] ? <EyeOff size={14} /> : <Eye size={14} />}
                                            </button>
                                        </div>
                                    </div>
                                ))}
                            </div>
                        </div>
                    ))}

                    <div className="p-3 bg-emerald-500/10 border border-emerald-500/20 rounded-lg flex items-start gap-2.5">
                        <ShieldCheck size={16} className="text-emerald-400 shrink-0 mt-0.5" />
                        <p className="text-[11px] text-emerald-300/90 leading-relaxed">
                            {i18n.t('template_store.mcp_wizard_vault_note') ||
                                'Credentials are saved strictly to your local .env configuration and encrypted runtime memory. They are never transmitted to external registries.'}
                        </p>
                    </div>

                    {/* Footer Actions */}
                    <div className="pt-4 border-t border-[color:var(--color-border)]/50 flex items-center justify-between gap-4">
                        <button
                            type="button"
                            onClick={onClose}
                            className="px-5 py-2 rounded-lg font-medium text-xs text-zinc-400 hover:text-white hover:bg-[color:var(--color-surface)] transition-all"
                        >
                            {i18n.t('template_store.btn_skip_mcp') || 'Configure Later in Settings'}
                        </button>

                        <button
                            type="submit"
                            disabled={saving}
                            className="px-6 py-2 rounded-lg font-bold text-xs bg-emerald-600 hover:bg-emerald-500 text-white shadow-lg shadow-emerald-500/20 transition-all flex items-center gap-2 disabled:opacity-50"
                        >
                            {saving ? (
                                <>
                                    <div className="w-3.5 h-3.5 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
                                    Saving...
                                </>
                            ) : (
                                <>
                                    <Check size={14} />
                                    {i18n.t('template_store.btn_save_mcp') || 'Save & Activate Connectors'}
                                    <ArrowRight size={14} />
                                </>
                            )}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    );
}

// Metadata: [Template_Store]
