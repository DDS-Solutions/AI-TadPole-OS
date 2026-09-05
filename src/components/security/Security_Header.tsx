/**
 * @docs ARCHITECTURE:UI-Components
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Security / Security_Header
 * - **Primary Entrypoints**: `Security_Header`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { ShieldCheck } from 'lucide-react';
import { Tooltip as UITooltip } from '../ui';
import { i18n } from '../../i18n';
import type { Agent_Health } from '../../services/tadpoleos_service';

interface SecurityHeaderProps {
    agent_health: Agent_Health[];
    merkle_integrity: number;
}

export function Security_Header({ agent_health, merkle_integrity }: SecurityHeaderProps) {
    return (
        <>
            <script type="application/ld+json">
                {JSON.stringify({
                    "@context": "https://schema.org",
                    "@type": "SoftwareApplication",
                    "name": "Tadpole OS Security Hub",
                    "description": "System-wide governance monitoring, budget enforcement, and cryptographic audit verification center.",
                    "provider": { "@type": "Organization", "name": "Sovereign Engineering" },
                    "applicationCategory": "Security System"
                })}
            </script>
            <header className="flex justify-between items-start bg-[color:var(--color-surface)] border border-[color:var(--color-border)] p-6 rounded-2xl">
                <div className="space-y-1">
                    <h1 className="text-2xl font-bold text-zinc-100 flex items-center gap-2">
                        <ShieldCheck className="text-emerald-500" />
                        {i18n.t('security.title')}
                    </h1>
                    <p className="text-zinc-500 text-sm mt-1">
                        {i18n.t('security.subtitle')}
                    </p>
                </div>
                <div className="flex items-center gap-3">
                    <div className="flex items-center -space-x-2 p-1">
                        {agent_health.slice(0, 5).map(a => (
                            <UITooltip key={a.agent_id} content={`${a.name}: ${a.is_healthy ? i18n.t('security.status_healthy') : i18n.t('security.status_degraded')}`}>
                                <div className={`w-8 h-8 rounded-full border-2 border-[color:var(--color-surface)] flex items-center justify-center text-[10px] font-bold relative transition-transform hover:scale-110 hover:z-10 ${a.is_healthy ? 'bg-emerald-500/20 text-emerald-400' : 'bg-red-500/20 text-red-400'}`}>
                                    {a.name.charAt(0)}
                                </div>
                            </UITooltip>
                        ))}
                        {agent_health.length > 5 && (
                            <UITooltip content={i18n.t('security.remaining_agents', { count: agent_health.length - 5, defaultValue: `${agent_health.length - 5} more agents` })}>
                                <div className="w-8 h-8 rounded-full border-2 border-[color:var(--color-surface)] bg-zinc-800 text-zinc-400 flex items-center justify-center text-[10px] font-bold relative z-10 hover:bg-zinc-700 transition-colors">
                                    +{agent_health.length - 5}
                                </div>
                            </UITooltip>
                        )}
                    </div>
                    <div className="h-8 w-px bg-zinc-800 mx-2" />
                    <UITooltip content={i18n.t('security.tooltip_audit_integrity')}>
                        <div className={`px-3 py-1 border rounded-full flex items-center gap-2 ${merkle_integrity === 1.0 ? 'bg-emerald-500/10 border-emerald-500/30' : 'bg-red-500/10 border-red-500/30'}`}>
                            <div className={`w-2 h-2 rounded-full animate-pulse ${merkle_integrity === 1.0 ? 'bg-emerald-500' : 'bg-red-500'}`} />
                            <span className={`text-[10px] font-bold uppercase tracking-widest ${merkle_integrity === 1.0 ? 'text-emerald-400' : 'text-red-400'}`}>
                                {merkle_integrity === 1.0 ? i18n.t('security.system_secured') : i18n.t('security.integrity_compromised')}
                            </span>
                        </div>
                    </UITooltip>
                </div>
            </header>
        </>
    );
}
