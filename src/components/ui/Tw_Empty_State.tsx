import React from 'react';

interface Tw_Empty_State_Props {
    icon?: React.ReactNode;
    title: string;
    description?: string;
    class_name?: string;
    action?: React.ReactNode;
}

/**
 * AI-Centric Empty State
 * Consistent with Tadpole OS premium aesthetics.
 */
export const Tw_Empty_State: React.FC<Tw_Empty_State_Props> = ({
    icon,
    title,
    description,
    class_name = '',
    action
}) => (
    <div className={`col-span-full flex flex-col items-center justify-center py-16 px-6 border border-dashed border-zinc-800 rounded-3xl text-zinc-600 gap-4 bg-zinc-950/20 animate-in fade-in zoom-in-95 duration-500 ${class_name}`}>
        {icon && <div className="p-4 bg-zinc-900 rounded-full border border-zinc-800/50 mb-2 opacity-50">{icon}</div>}
        <div className="text-center space-y-1">
            <h3 className="text-sm font-bold text-zinc-400 uppercase tracking-widest">{title}</h3>
            {description && <p className="text-xs text-zinc-600 max-w-sm leading-relaxed">{description}</p>}
        </div>
        {action && (
            <div className="mt-2">
                {action}
            </div>
        )}
    </div>
);
