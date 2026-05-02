/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Real-time WebSocket connectivity status indicator. 
 * Renders high-visibility warnings for `error` or `disconnected` states using system-wide alert tokens.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Sticky banner if `Connection_State` fails to transition back to 'connected', or animation freeze during high CPU load.
 * - **Telemetry Link**: Search for `[Connection_Banner]` or `socket_state` in UI tracing.
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


// Metadata: [Connection_Banner]
