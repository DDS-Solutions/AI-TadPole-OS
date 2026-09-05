/**
 * @docs ARCHITECTURE:UI-Components
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Ui / Section_Header
 * - **Primary Entrypoints**: `Section_Header`
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
import { i18n } from '../../i18n';

interface SectionHeaderProps {
    icon?: string | React.ReactNode;
    title: string;
    subtitle?: string;
    badge?: string | number;
    action?: React.ReactNode;
}

export const Section_Header: React.FC<SectionHeaderProps> = ({
    icon,
    title,
    subtitle,
    badge,
    action,
}) => (
    <div className="flex items-center justify-between pb-3 border-b border-[color:var(--color-border)]/50 mb-6 group">
        <div className="flex items-center gap-3">
            {icon && (
                <div className="text-lg opacity-80 group-hover:opacity-100 transition-opacity">
                    {typeof icon === 'string' ? (
                        <span role="img" aria-label={i18n.t('common.icon_aria') || 'section-icon'}>{icon}</span>
                    ) : icon}
                </div>
            )}
            <div className="flex flex-col">
                <div className="flex items-center gap-2.5">
                    <h3 className="text-sm font-bold text-zinc-100 uppercase tracking-tight">
                        {title}
                    </h3>
                    {badge !== undefined && (
                        <span className="bg-zinc-800 text-zinc-400 text-[10px] font-bold px-2 py-0.5 rounded-md font-mono border border-zinc-700/50">
                            {badge}
                        </span>
                    )}
                </div>
                {subtitle && (
                    <span className="text-[10px] text-zinc-500 font-medium uppercase tracking-wider mt-0.5">
                        {subtitle}
                    </span>
                )}
            </div>
        </div>
        {action && (
            <div className="flex items-center gap-2 animate-in fade-in slide-in-from-right-2 duration-300">
                {action}
            </div>
        )}
    </div>
);
