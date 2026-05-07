/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **@docs ARCHITECTURE:Interface**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Empty_State]` in observability traces.
 */

/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Reusable fallback visualization for null data states. 
 * Provides a standardized "Neutral Footprint" using Tailwind tokens.
 */

import React from 'react';

interface EmptyStateProps {
    icon?: string | React.ReactNode;
    title: string;
    description?: string;
    variant?: 'simple' | 'dashed';
    class_name?: string;
    action?: React.ReactNode | {
        label: string;
        onClick: () => void;
    };
}

type ActionObject = { label: string; onClick: () => void };

const is_action_object = (action: unknown): action is ActionObject =>
    typeof action === 'object' && action !== null && 'label' in action && 'onClick' in action;

const render_action = (action: React.ReactNode | ActionObject): React.ReactNode => {
    if (!action) return null;
    if (React.isValidElement(action)) return action;
    if (is_action_object(action)) {
        return (
            <button
                onClick={action.onClick}
                className="px-6 py-2 bg-green-500/10 text-green-400 border border-green-500/20 rounded-md text-[10px] font-bold uppercase tracking-widest hover:bg-green-500/20 transition-all duration-300 active:scale-95 shadow-lg shadow-green-500/5"
            >
                {action.label}
            </button>
        );
    }
    return action;
};

export const Empty_State: React.FC<EmptyStateProps> = ({
    icon = '📭',
    title,
    description,
    variant = 'simple',
    class_name = '',
    action,
}) => {
    if (variant === 'dashed') {
        return (
            <div className={`col-span-full flex flex-col items-center justify-center py-16 px-6 border border-dashed border-zinc-800 rounded-3xl text-zinc-600 gap-4 bg-zinc-950/20 animate-in fade-in zoom-in-95 duration-500 ${class_name}`}>
                {icon && (
                    <div className="p-4 bg-zinc-900 rounded-full border border-zinc-800/50 mb-2 opacity-50">
                        {typeof icon === 'string' ? <span role="img" aria-label="icon">{icon}</span> : icon}
                    </div>
                )}
                <div className="text-center space-y-1">
                    <h3 className="text-sm font-bold text-zinc-400 uppercase tracking-widest">{title}</h3>
                    {description && <p className="text-xs text-zinc-600 max-w-sm leading-relaxed">{description}</p>}
                </div>
                {action && (
                    <div className="mt-2">
                        {render_action(action)}
                    </div>
                )}
            </div>
        );
    }

    return (
        <div className={`flex flex-col items-center justify-center py-12 px-6 text-center animate-in fade-in duration-500 ${class_name}`}>
            <div className="text-4xl mb-4 opacity-50 grayscale hover:grayscale-0 transition-all duration-500 transform hover:scale-110">
                {typeof icon === 'string' ? (
                    <span role="img" aria-label="icon">{icon}</span>
                ) : (
                    icon
                )}
            </div>
            <h3 className="text-base font-bold text-zinc-100 mb-1 uppercase tracking-tight">
                {title}
            </h3>
            {description && (
                <p className="text-xs text-zinc-500 mb-6 max-w-[280px] leading-relaxed">
                    {description}
                </p>
            )}
            {action && (
                <div className="mt-2">
                    {render_action(action)}
                </div>
            )}
        </div>
    );
};

// Backwards compatibility for existing imports
export const Tw_Empty_State: React.FC<EmptyStateProps> = (props) => <Empty_State {...props} variant="dashed" />;

// Metadata: [Empty_State]
