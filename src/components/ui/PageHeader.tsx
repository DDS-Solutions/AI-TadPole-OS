import React from 'react';
import { Command } from 'lucide-react';
import { HeaderTicker, Tooltip } from './';
import { useSwarmMetrics } from '../../hooks/useSwarmMetrics';
import type { ConnectionState } from '../../services/socket';

interface PageHeaderProps {
    title?: string;
    pathname?: string;
    connectionState?: ConnectionState;
    engineHealth?: { uptime: number; agentCount: number } | null;
    actions?: React.ReactNode;
}

const PATH_TITLES: Record<string, string> = {
    '/dashboard': 'Operations Center',
    '/org-chart': 'Agent Hierarchy Layer',
    '/missions': 'Mission Management',
    '/standups': 'Voice Interface',
    '/workspaces': 'Workspace Manager',
    '/models': 'AI Provider Manager',
    '/engine': 'System Telemetry',
    '/oversight': 'Oversight & Compliance',
    '/agents': 'Agent Swarm Manager',
    '/skills': 'Skills & Workflows',
    '/benchmarks': 'Performance Analysis',
    '/scheduled-jobs': 'Scheduled Jobs',
    '/docs': 'Knowledge Base',
    '/settings': 'System Configuration',
};

export const PageHeader: React.FC<PageHeaderProps> = ({ 
    title, 
    pathname, 
    connectionState, 
    engineHealth,
    actions 
}) => {
    const displayTitle = title || (pathname ? PATH_TITLES[pathname] : '');
    const metrics = useSwarmMetrics();

    return (
        <header className="h-14 border-b border-zinc-800 bg-zinc-950 flex items-center justify-between px-6 shrink-0 gap-4">
            <div className="flex items-center gap-4 text-sm text-zinc-500 shrink-0">
                <h1 className="text-zinc-200 font-medium truncate m-0 p-0 text-base whitespace-nowrap tracking-wide">
                    {displayTitle}
                </h1>
            </div>

            <div className="flex-1 flex justify-center overflow-hidden">
                <HeaderTicker duration={60}>
                    {/* Status Metrics */}
                    {metrics.map((m) => (
                        <Tooltip key={m.label} content={m.tooltip} position="bottom">
                            <div className="flex items-center gap-2 cursor-help group">
                                <m.icon size={13} className={`${m.color} opacity-70 group-hover:opacity-100 transition-opacity`} />
                                <span className="text-[10px] font-bold text-zinc-500 uppercase tracking-tighter whitespace-nowrap">
                                    {m.label}: <span className={`${m.color} font-mono tracking-normal text-zinc-200`}>{m.value}</span>
                                </span>
                            </div>
                        </Tooltip>
                    ))}

                    {/* Actions if present */}
                    {actions && (
                        <div className="flex items-center gap-2">
                            {actions}
                        </div>
                    )}

                    {/* Connection Status */}
                    {connectionState && (
                        <Tooltip content={connectionState === 'connected' ? "WebSocket Connection Established" : "Engine Connection Offline"} position="left">
                            <div className="flex items-center gap-2 text-xs cursor-help">
                                <span className="flex h-2 w-2 relative">
                                    {connectionState === 'connected' && (
                                        <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-500 opacity-75"></span>
                                    )}
                                    {connectionState === 'connecting' && (
                                        <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-amber-500 opacity-75"></span>
                                    )}
                                    <span className={`relative inline-flex rounded-full h-2 w-2 ${connectionState === 'connected' ? 'bg-emerald-500' :
                                        connectionState === 'connecting' ? 'bg-amber-500' : 'bg-red-500'
                                        }`}></span>
                                </span>
                                <span className={`font-mono font-medium ${connectionState === 'connected' ? 'text-emerald-500' :
                                    connectionState === 'connecting' ? 'text-amber-500' : 'text-red-500'
                                    }`}>
                                    {connectionState === 'connected' ? (
                                        engineHealth ? `ONLINE • ${engineHealth.agentCount} AGENTS` : 'ENGINE ONLINE'
                                    ) : connectionState === 'connecting' ? 'CONNECTING...' : 'ENGINE OFFLINE'}
                                </span>
                            </div>
                        </Tooltip>
                    )}
                </HeaderTicker>
            </div>

            <div className="flex items-center gap-4 shrink-0">
                <Tooltip content="Command Palette (Ctrl+K)" position="left">
                    <div className="hidden md:flex items-center gap-1 text-[10px] text-zinc-600 font-mono border border-zinc-800 rounded px-1.5 py-0.5 cursor-help">
                        <Command size={10} /> K
                    </div>
                </Tooltip>
            </div>
        </header>
    );
};
