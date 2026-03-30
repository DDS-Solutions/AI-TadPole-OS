import React, { useState, useEffect } from 'react';
import { Wifi, WifiOff, RefreshCcw } from 'lucide-react';
import { TadpoleOSSocket, type ConnectionState } from '../../services/socket';
import { i18n } from '../../i18n';

export const ConnectionBanner: React.FC = () => {
    const [state, setState] = useState<ConnectionState>(TadpoleOSSocket.getConnectionState());

    useEffect(() => {
        return TadpoleOSSocket.subscribeStatus((newState) => {
            setState(newState);
        });
    }, []);

    if (state === 'connected') return null;

    const isReconnecting = state === 'reconnecting' || state === 'connecting';

    return (
        <div className={`
            fixed top-0 left-0 right-0 z-[100] h-8 flex items-center justify-center px-4
            transition-all duration-500 animate-in slide-in-from-top
            ${state === 'disconnected' ? 'bg-red-500 text-white' : 'bg-amber-500 text-black'}
            backdrop-blur-md bg-opacity-90 border-b border-black/10 shadow-lg
        `}>
            <div className="flex items-center gap-3 max-w-7xl w-full">
                <div className="flex items-center gap-2">
                    {state === 'disconnected' ? (
                        <WifiOff size={14} className="animate-pulse" />
                    ) : (
                        <Wifi size={14} className="animate-spin-slow" />
                    )}
                    <span className="text-[10px] font-bold uppercase tracking-[0.2em]">
                        {state === 'disconnected' 
                            ? i18n.t('system.connection_lost') 
                            : i18n.t('system.reconnecting')}
                    </span>
                </div>

                <div className="h-3 w-px bg-current opacity-20 hidden sm:block" />

                <span className="text-[10px] font-medium opacity-90 hidden sm:block truncate">
                    {state === 'disconnected' 
                        ? i18n.t('system.reconnect_fail_hint') 
                        : i18n.t('system.reconnect_attempt_hint')}
                </span>

                <div className="flex-1" />

                <button 
                    onClick={() => TadpoleOSSocket.connect()}
                    disabled={isReconnecting}
                    className={`
                        flex items-center gap-1.5 px-3 py-1 rounded text-[9px] font-bold uppercase tracking-wider
                        transition-all hover:scale-105 active:scale-95
                        ${state === 'disconnected' 
                            ? 'bg-white text-red-600 hover:bg-zinc-100' 
                            : 'bg-black text-amber-500 hover:bg-zinc-900'}
                        disabled:opacity-50 disabled:cursor-not-allowed
                    `}
                >
                    <RefreshCcw size={10} className={isReconnecting ? 'animate-spin' : ''} />
                    {i18n.t('common.retry')}
                </button>
            </div>
        </div>
    );
};
