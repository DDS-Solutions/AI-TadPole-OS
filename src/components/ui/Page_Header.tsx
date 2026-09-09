/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Ui / Page_Header
 * - **Primary Entrypoints**: `Page_Header`
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
import { useEngineStatus } from '../../hooks/use_engine_status';
import { Connection_Banner } from './Connection_Banner';
import type { Connection_State as Connection_State_Type } from '../../services/socket';

interface Page_Header_Props {
    title: string;
    subtitle?: string;
    actions?: React.ReactNode;
    margin_bottom?: string;
}

export const Page_Header: React.FC<Page_Header_Props> = React.memo(({ 
    title, 
    subtitle, 
    actions,
    margin_bottom = 'mb-8'
}) => {
    const { status } = useEngineStatus();

    return (
        <div className={`space-y-4 relative ${margin_bottom}`}>
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
});
