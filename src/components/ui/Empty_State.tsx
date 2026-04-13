/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Reusable fallback visualization for null data states. 
 * Provides a standardized "Neutral Footprint" for empty lists, tables, and mission grids with optional CTA buttons.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Action click-jacking if nested improperly, SVG emoji render fallback issues, or layout shift during data ingestion.
 * - **Telemetry Link**: Search for `[Empty_State]` or `Tw_Empty_State` in UI logs.
 */

import React from 'react';

interface EmptyStateProps {
    icon?: string;
    title: string;
    description?: string;
    action?: {
        label: string;
        onClick: () => void;
    };
}

/**
 * Reusable empty-state placeholder for lists and tables.
 *
 * Replaces the ad-hoc "No items found" blocks duplicated across:
 * - Missions (no missions)
 * - Model_Manager (no models/providers)
 * - Skills (no skills)
 * - Workspaces (no workspaces)
 * - Standups (no standups)
 */
export const Empty_State: React.FC<EmptyStateProps> = ({
    icon = '📭',
    title,
    description,
    action,
}) => (
    <div
        style={{
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            justifyContent: 'center',
            padding: '48px 24px',
            textAlign: 'center',
            opacity: 0.7,
        }}
    >
        <span style={{ fontSize: '2.5rem', marginBottom: 12 }}>{icon}</span>
        <h3
            style={{
                fontSize: '1rem',
                fontWeight: 600,
                color: 'var(--text-primary, #e2e8f0)',
                margin: '0 0 4px',
            }}
        >
            {title}
        </h3>
        {description && (
            <p
                style={{
                    fontSize: '0.8rem',
                    color: 'var(--text-secondary, #94a3b8)',
                    margin: '0 0 16px',
                    maxWidth: 320,
                }}
            >
                {description}
            </p>
        )}
        {action && (
            <button
                onClick={action.onClick}
                style={{
                    background: 'rgba(96, 165, 250, 0.15)',
                    color: '#60a5fa',
                    border: '1px solid rgba(96, 165, 250, 0.3)',
                    borderRadius: 6,
                    padding: '8px 16px',
                    fontSize: '0.75rem',
                    fontWeight: 600,
                    cursor: 'pointer',
                    transition: 'all 0.2s',
                }}
                onMouseOver={(e) => {
                    e.currentTarget.style.background = 'rgba(96, 165, 250, 0.25)';
                }}
                onMouseOut={(e) => {
                    e.currentTarget.style.background = 'rgba(96, 165, 250, 0.15)';
                }}
            >
                {action.label}
            </button>
        )}
    </div>
);

