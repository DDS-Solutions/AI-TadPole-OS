import React from 'react';
import { WifiOff, AlertTriangle } from 'lucide-react';
import type { Connection_State } from '../../services/socket';
import { i18n } from '../../i18n';

interface Connection_Banner_Props {
    state: Connection_State;
}

export const Connection_Banner: React.FC<Connection_Banner_Props> = ({ state }) => {
    if (state === 'connected' || state === 'connecting') return null;

    return (
        <div className={`p-2 flex items-center justify-center gap-2 text-xs font-bold uppercase tracking-wider animate-in fade-in slide-in-from-top-1 ${state === 'error' ? 'bg-red-500/10 text-red-500 border-b border-red-500/20' : 'bg-amber-500/10 text-amber-500 border-b border-amber-500/20'}`}>
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
