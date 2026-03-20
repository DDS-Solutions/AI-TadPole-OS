import React from 'react';

interface TwEmptyStateProps {
    icon?: React.ReactNode;
    title: string;
    description?: string;
    className?: string;
}

/**
 * Tailwind-styled empty-state placeholder, consistent with the dark theme.
 * 
 * Drop-in replacement for the ad-hoc "No items" blocks across:
 * - Capabilities (NO SKILLS CONFIGURED / NO WORKFLOWS CONFIGURED)
 * - OversightDashboard (No actions recorded / No optimization traces)
 * - Missions (All agents deployed)
 */
export const TwEmptyState: React.FC<TwEmptyStateProps> = ({
    icon,
    title,
    description,
    className = '',
}) => (
    <div className={`col-span-full flex flex-col items-center justify-center py-12 border border-dashed border-zinc-800 rounded-xl text-zinc-600 gap-3 ${className}`}>
        {icon && <div className="opacity-10">{icon}</div>}
        <p className="text-xs italic uppercase tracking-widest font-bold">{title}</p>
        {description && <p className="text-[10px] text-zinc-700 font-mono">{description}</p>}
    </div>
);
