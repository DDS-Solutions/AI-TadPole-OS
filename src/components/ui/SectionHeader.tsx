import React from 'react';

interface SectionHeaderProps {
    icon?: string;
    title: string;
    subtitle?: string;
    badge?: string | number;
    action?: React.ReactNode;
}

/**
 * Reusable section header with icon, title, subtitle, badge count, and optional action.
 *
 * Replaces the duplicated header blocks in:
 * - Missions (section headers with counts)
 * - ModelManager (provider/model section headers)
 * - Capabilities (skills/workflows headers)
 * - OversightDashboard (pending/ledger section headers)
 * - AgentConfigPanel (configuration section headers)
 */
export const SectionHeader: React.FC<SectionHeaderProps> = ({
    icon,
    title,
    subtitle,
    badge,
    action,
}) => (
    <div
        style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            padding: '0 0 12px',
            borderBottom: '1px solid rgba(148, 163, 184, 0.08)',
            marginBottom: 16,
        }}
    >
        <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
            {icon && <span style={{ fontSize: '1.1rem' }}>{icon}</span>}
            <div>
                <h3
                    style={{
                        margin: 0,
                        fontSize: '0.85rem',
                        fontWeight: 700,
                        color: 'var(--text-primary, #e2e8f0)',
                        display: 'flex',
                        alignItems: 'center',
                        gap: 6,
                    }}
                >
                    {title}
                    {badge !== undefined && (
                        <span
                            style={{
                                background: 'rgba(96, 165, 250, 0.12)',
                                color: '#60a5fa',
                                fontSize: '0.6rem',
                                fontWeight: 700,
                                padding: '2px 6px',
                                borderRadius: 4,
                                fontFamily: "'JetBrains Mono', monospace",
                            }}
                        >
                            {badge}
                        </span>
                    )}
                </h3>
                {subtitle && (
                    <span
                        style={{
                            fontSize: '0.7rem',
                            color: 'var(--text-secondary, #94a3b8)',
                            marginTop: 2,
                        }}
                    >
                        {subtitle}
                    </span>
                )}
            </div>
        </div>
        {action && <div>{action}</div>}
    </div>
);
