import { Command } from 'lucide-react';
import { SwarmStatusHeader } from '../SwarmStatusHeader';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';
import type { ConnectionState } from '../../services/socket';

interface OrgHeaderProps {
    pathname: string;
    connectionState: ConnectionState;
    engineHealth: { uptime: number; agentCount: number } | null;
}

export function OrgHeader({ pathname, connectionState, engineHealth }: OrgHeaderProps) {
    return (
        <header className="h-14 border-b border-zinc-800 bg-zinc-950 flex items-center justify-between px-6 shrink-0 gap-4">
            <div className="flex items-center gap-4 text-sm text-zinc-500 shrink-0">
                <h1 className="text-zinc-200 font-medium truncate m-0 p-0 text-base whitespace-nowrap">
                    {pathname === '/' && i18n.t('header.ops_center')}
                    {pathname === '/org-chart' && i18n.t('header.hierarchy_layer')}
                    {pathname === '/missions' && i18n.t('header.mission_management')}
                    {pathname === '/standups' && i18n.t('header.voice_interface')}
                    {pathname === '/workspaces' && i18n.t('header.workspace_manager')}
                    {pathname === '/models' && i18n.t('header.provider_manager')}
                    {pathname === '/engine' && i18n.t('header.system_telemetry')}
                    {pathname === '/oversight' && i18n.t('header.oversight_compliance')}
                    {pathname === '/agents' && i18n.t('header.swarm_manager')}
                    {pathname === '/capabilities' && i18n.t('header.skills_workflows')}
                    {pathname === '/benchmarks' && i18n.t('header.performance_analysis')}
                    {pathname === '/scheduled-jobs' && i18n.t('header.scheduled_jobs')}
                    {pathname === '/docs' && i18n.t('header.knowledge_base')}
                    {pathname === '/settings' && i18n.t('header.system_config')}
                </h1>
            </div>

            <div className="flex-1 flex justify-center overflow-hidden">
                <SwarmStatusHeader />
            </div>
            <div className="flex items-center gap-4">
                <Tooltip content={connectionState === 'connected' ? i18n.t('header.ws_connected') : i18n.t('header.engine_offline')} position="left">
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
                                engineHealth ? i18n.t('header.engine_online_agents', { count: engineHealth.agentCount }) : i18n.t('header.engine_online')
                            ) : connectionState === 'connecting' ? i18n.t('header.connecting') : i18n.t('header.engine_offline_label')}
                        </span>
                    </div>
                </Tooltip>
                <Tooltip content={i18n.t('header.cmd_palette_tooltip')} position="left">
                    <div className="hidden md:flex items-center gap-1 text-[10px] text-zinc-600 font-mono border border-zinc-800 rounded px-1.5 py-0.5 cursor-help">
                        <Command size={10} /> K
                    </div>
                </Tooltip>
            </div>
        </header>
    );
}
