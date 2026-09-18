/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Skills / Security_Report_Modal
 * - **Primary Entrypoints**: `Security_Report_Modal`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import React from 'react';
import { Shield, ShieldCheck, ShieldAlert, CheckCircle2, AlertTriangle, X } from 'lucide-react';
import { i18n } from '../../i18n';
import type { Skill_Definition } from '../../stores/skill_store';

interface Security_Report_Modal_Props {
    is_open: boolean;
    on_close: () => void;
    skill: Skill_Definition | null;
}

export const Security_Report_Modal: React.FC<Security_Report_Modal_Props> = ({
    is_open,
    on_close,
    skill
}) => {
    if (!is_open || !skill) return null;

    const report = skill.security_report;
    const score = skill.security_score ?? 0;
    const severity = skill.security_severity ?? 'LOW';

    let headerColor = 'text-[color:var(--color-cyber-green)]';
    let ShieldIcon = ShieldCheck;
    let scoreBg = 'bg-[color:var(--color-cyber-green)]/10 border-[color:var(--color-cyber-green)]/20';

    if (score > 20 && score <= 50) {
        headerColor = 'text-[color:var(--color-cyber-amber)]';
        ShieldIcon = ShieldAlert;
        scoreBg = 'bg-[color:var(--color-cyber-amber)]/10 border-[color:var(--color-cyber-amber)]/20';
    } else if (score > 50) {
        headerColor = 'text-[color:var(--color-cyber-red)]';
        ShieldIcon = ShieldAlert;
        scoreBg = 'bg-[color:var(--color-cyber-red)]/10 border-[color:var(--color-cyber-red)]/25';
    }

    const findings = report?.filtered_findings ?? [];

    return (
        <div className="fixed inset-0 z-50 bg-zinc-950/80 backdrop-blur-md flex items-center justify-center p-4">
            <div className="bg-[color:var(--color-background)] border border-[color:var(--color-border)] w-full max-w-2xl rounded-2xl shadow-2xl flex flex-col max-h-[90vh] relative overflow-hidden">
                <div className="neural-grid opacity-10" />
                
                {/* Header */}
                <div className="p-5 border-b border-[color:var(--color-border)] flex justify-between items-center shrink-0 relative z-10 bg-[color:color-mix(in_srgb,var(--color-background)_50%,transparent)]">
                    <h2 className="text-base font-bold text-zinc-100 flex items-center gap-2 font-mono">
                        <Shield className="w-4 h-4 text-zinc-400" />
                        {i18n.t('skills.security_report_title', { defaultValue: 'Security Audit Report' })}
                        <span className="text-zinc-500 font-normal">/</span>
                        <span className="text-green-400 font-normal text-sm">{skill.name}</span>
                    </h2>
                    <button onClick={on_close} aria-label={i18n.t('common.close', { defaultValue: 'Close' })} className="text-zinc-500 hover:text-zinc-300 p-1 cursor-pointer transition-colors">
                        <X className="w-4 h-4" />
                    </button>
                </div>

                {/* Content */}
                <div className="p-6 overflow-y-auto space-y-6 custom-scrollbar relative z-10 bg-[color:color-mix(in_srgb,var(--color-background)_80%,transparent)]">
                    {/* Score Panel */}
                    <div className={`p-4 rounded-xl border flex items-center justify-between ${scoreBg}`}>
                        <div className="flex items-center gap-3">
                            <ShieldIcon className={`w-8 h-8 ${headerColor}`} />
                            <div>
                                <h3 className="font-mono text-xs uppercase tracking-wider text-zinc-400 font-bold">
                                    {i18n.t('skills.risk_evaluation', { defaultValue: 'Security Assessment' })}
                                </h3>
                                <p className="text-zinc-200 text-sm font-semibold mt-0.5">
                                    {score > 50 
                                        ? i18n.t('skills.threat_flagged', { defaultValue: 'Potential threats detected. Oversight mandated.' })
                                        : score > 20 
                                        ? i18n.t('skills.threat_caution', { defaultValue: 'Minor warnings flagged. Review recommended.' })
                                        : i18n.t('skills.threat_clean', { defaultValue: 'No vulnerabilities detected. Verified secure.' })
                                    }
                                </p>
                            </div>
                        </div>
                        <div className="text-right flex flex-col items-end">
                            <span className="text-[10px] font-mono text-zinc-500 uppercase tracking-widest">{i18n.t('skills.risk_score', { defaultValue: 'Risk Score' })}</span>
                            <span className={`text-2xl font-bold font-mono leading-none mt-1 ${headerColor}`}>{score}/100</span>
                            <span className="text-[9px] font-mono text-zinc-500 mt-1 uppercase bg-zinc-900 border border-[color:var(--color-border)] px-1.5 py-0.5 rounded">
                                {severity}
                            </span>
                        </div>
                    </div>

                    {/* Recommendation Card */}
                    {report?.risk_recommendation && (
                        <div className="bg-[color:var(--color-surface)] border border-[color:var(--color-border)] rounded-xl p-4">
                            <h4 className="mono-label mb-1.5">{i18n.t('skills.remediation_steps', { defaultValue: 'Remediation / Recommendation' })}</h4>
                            <p className="text-zinc-300 text-xs font-mono leading-relaxed">{report.risk_recommendation}</p>
                        </div>
                    )}

                    {/* Findings list */}
                    <div>
                        <h4 className="mono-label mb-3">{i18n.t('skills.vulnerabilities_list', { defaultValue: 'Scanner Violations' })} ({findings.length})</h4>
                        {findings.length === 0 ? (
                            <div className="border border-[color:var(--color-border)]/50 rounded-xl p-6 text-center bg-zinc-950/20 flex flex-col items-center justify-center">
                                <CheckCircle2 className="w-8 h-8 text-[color:var(--color-cyber-green)] mb-2" />
                                <p className="text-zinc-300 font-mono text-xs font-bold">{i18n.t('skills.no_violations', { defaultValue: 'NVIDIA SkillSpector Gate Passed' })}</p>
                                <p className="text-zinc-500 font-mono text-[10px] mt-1">{i18n.t('skills.no_violations_detail', { defaultValue: 'No AST taints, prompt injections, or malicious signatures detected.' })}</p>
                            </div>
                        ) : (
                            <div className="space-y-4">
                                {findings.map((finding, idx) => {
                                    let severityColor = 'text-[color:var(--color-cyber-green)] bg-[color:var(--color-cyber-green)]/15 border-[color:var(--color-cyber-green)]/20';
                                    if (finding.severity.toUpperCase() === 'MEDIUM' || finding.severity.toUpperCase() === 'WARNING') {
                                        severityColor = 'text-[color:var(--color-cyber-amber)] bg-[color:var(--color-cyber-amber)]/15 border-[color:var(--color-cyber-amber)]/20';
                                    } else if (finding.severity.toUpperCase() === 'HIGH' || finding.severity.toUpperCase() === 'CRITICAL' || finding.severity.toUpperCase() === 'ERROR') {
                                        severityColor = 'text-[color:var(--color-cyber-red)] bg-[color:var(--color-cyber-red)]/15 border-[color:var(--color-cyber-red)]/20';
                                    }

                                    return (
                                        <div key={idx} className="border border-[color:var(--color-border)] rounded-xl overflow-hidden bg-zinc-950/10">
                                            {/* Finding header */}
                                            <div className="px-4 py-2.5 border-b border-[color:var(--color-border)]/80 flex items-center justify-between bg-zinc-900/40">
                                                <span className="font-mono text-xs font-bold text-zinc-300">{finding.rule_id}</span>
                                                <span className={`text-[9px] font-mono font-bold px-2 py-0.5 rounded border uppercase ${severityColor}`}>
                                                    {finding.severity}
                                                </span>
                                            </div>
                                            {/* Finding Body */}
                                            <div className="p-4 space-y-3">
                                                {finding.finding && (
                                                    <div>
                                                        <span className="text-[10px] font-mono text-zinc-500 uppercase block mb-1">{i18n.t('skills.finding_desc', { defaultValue: 'Violation Source' })}</span>
                                                        <p className="text-zinc-200 text-xs font-mono bg-zinc-950 p-2.5 rounded border border-[color:var(--color-border)]/40 overflow-x-auto whitespace-pre-wrap leading-relaxed">
                                                            {finding.finding}
                                                        </p>
                                                    </div>
                                                )}
                                                {finding.explanation && (
                                                    <div>
                                                        <span className="text-[10px] font-mono text-zinc-500 uppercase block mb-1">{i18n.t('skills.finding_explanation', { defaultValue: 'Explanation' })}</span>
                                                        <p className="text-zinc-400 text-xs leading-relaxed font-mono">
                                                            {finding.explanation}
                                                        </p>
                                                    </div>
                                                )}
                                                {finding.location && (
                                                    <div className="flex items-center gap-1.5 text-[10px] text-zinc-500 font-mono">
                                                        <AlertTriangle className="w-3.5 h-3.5 text-zinc-600 shrink-0" />
                                                        <span>{i18n.t('skills.finding_location', { defaultValue: 'Location' })}:</span>
                                                        <span className="text-zinc-400 bg-zinc-900 px-1.5 py-0.5 rounded">{finding.location}</span>
                                                    </div>
                                                )}
                                            </div>
                                        </div>
                                    );
                                })}
                            </div>
                        )}
                    </div>
                </div>

                {/* Footer */}
                <div className="p-5 border-t border-[color:var(--color-border)] flex justify-end shrink-0 relative z-10 bg-[color:var(--color-background)]/90">
                    <button 
                        onClick={on_close} 
                        className="bg-zinc-800 hover:bg-zinc-700 border border-[color:var(--color-border)] text-zinc-300 px-6 py-2 rounded-lg text-xs font-bold transition-colors cursor-pointer"
                    >
                        {i18n.t('skills.btn_close', { defaultValue: 'Acknowledge' })}
                    </button>
                </div>
            </div>
        </div>
    );
};
