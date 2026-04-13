/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Top-level layout header for system pages. 
 * Orchestrates title propagation, sub-navigation actions, and the global `Connection_Banner` for engine status monitoring.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Hook-driven title desync, `Connection_Banner` flickering on rapid re-renders, or action item overflow on mobile viewports.
 * - **Telemetry Link**: Search for `[Page_Header]` or `use_engine_status` in UI tracing.
 */

import React from 'react';
import { useEngineStatus } from '../../hooks/use_engine_status';
import { Connection_Banner } from './Connection_Banner';
import type { Connection_State as Connection_State_Type } from '../../services/socket';

interface Page_Header_Props {
    title: string;
    subtitle?: string;
    actions?: React.ReactNode;
}

export const Page_Header: React.FC<Page_Header_Props> = ({ title, subtitle, actions }) => {
    const { status } = useEngineStatus();

    return (
        <div className="space-y-4 mb-8">
            <Connection_Banner state={status as Connection_State_Type} />
            <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
                <div>
                    <h1 className="text-3xl font-bold text-zinc-100 tracking-tight">{title}</h1>
                    {subtitle && <p className="text-zinc-500 mt-1">{subtitle}</p>}
                </div>
                {actions && <div className="flex items-center gap-3">{actions}</div>}
            </div>
        </div>
    );
};

