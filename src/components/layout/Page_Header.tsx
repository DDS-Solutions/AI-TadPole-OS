/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Restored page-level header for titles and local actions.
 * Sited below the Tab Bar, satisfying the "leave it there" user preference.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Missing title (shows placeholder), or actions not rendering (check memory store sync).
 * - **Telemetry Link**: Search for `[Page_Header]` in UI logs.
 */

import React from 'react';

interface Page_Header_Props {
    title: React.ReactNode;
    subtitle?: string;
    actions?: React.ReactNode;
}

export const Page_Header: React.FC<Page_Header_Props> = ({ 
    title, 
    subtitle, 
    actions 
}) => {
    return (
        <div className="px-6 py-4 flex items-center justify-between border-b border-zinc-900 bg-zinc-950/50 backdrop-blur-md shrink-0">
            <div className="flex flex-col">
                <div className="flex items-center gap-3">
                    <div className="h-4 w-1 bg-green-500 rounded-full" />
                    <h1 className="text-zinc-100 font-bold text-lg tracking-tight">
                        {title}
                    </h1>
                </div>
                {subtitle && (
                    <p className="text-[10px] font-medium text-zinc-500 uppercase tracking-[0.2em] mt-1 ml-4">
                        {subtitle}
                    </p>
                )}
            </div>

            {actions && (
                <div className="flex items-center gap-2 animate-in fade-in slide-in-from-right-4">
                    {actions}
                </div>
            )}
        </div>
    );
};

// Metadata: [Page_Header]
