/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Ui / Connection_Banner
 * - **Primary Entrypoints**: `Connection_Banner`
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
import { WifiOff, AlertTriangle } from 'lucide-react';
import type { Connection_State } from '../../services/socket';
import { i18n } from '../../i18n';
import { THEME_TOKENS, Z_INDEX_MAP } from './theme_tokens';

interface Connection_Banner_Props {
    state: Connection_State;
}

export const Connection_Banner: React.FC<Connection_Banner_Props> = ({ state }) => {
    if (state === 'connected' || state === 'connecting') return null;

    const variantStyle = state === 'error' ? THEME_TOKENS.danger : THEME_TOKENS.warning;

    return (
        <div 
            data-testid="connection-banner"
            className={`absolute top-0 left-0 w-full p-2 flex items-center justify-center gap-2 text-xs font-bold uppercase tracking-wider animate-in fade-in slide-in-from-top-1 border-b ${variantStyle}`}
            style={{ zIndex: Z_INDEX_MAP.banner }}
        >
            {state === 'error' ? (
                <>
                    <AlertTriangle size={12} />
                    {i18n.t('system.connection_error')}
                </>
            ) : (
                <>
                    <WifiOff size={12} />
                    {i18n.t('system.disconnected')}
                </>
            )}
        </div>
    );
};
