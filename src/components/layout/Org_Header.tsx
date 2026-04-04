import { memo } from 'react';
import { Command, Shield } from 'lucide-react';
import { Swarm_Status_Header } from '../Swarm_Status_Header';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';
import type { Connection_State } from '../../services/socket';
import { use_settings_store } from '../../stores/settings_store';

interface Org_Header_Props {
    pathname: string;
    connection_state: Connection_State;
    engine_health: { uptime: number; agent_count: number } | null;
}

/**
 * Org_Header
 * Primary organizational header containing route titles, swarm telemetry, and system-wide toggles.
 * Refactored for strict snake_case compliance for backend parity.
 */
export const Org_Header = memo(({ pathname, connection_state, engine_health }: Org_Header_Props) => {
    const { settings, update_setting } = use_settings_store();

    const toggle_privacy = async () => {
        const new_mode = !settings.privacy_mode;
        update_setting('privacy_mode', new_mode);

        try {
            const { system_api_service } = await import('../../services/system_api_service');
            await system_api_service.update_governance_settings({
                privacy_mode: new_mode
            });
        } catch (e) {
            console.error("Failed to sync privacy mode with engine", e);
        }
    };

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
                    {pathname === '/skills' && i18n.t('header.skills_workflows')}
                    {pathname === '/benchmarks' && i18n.t('header.performance_analysis')}
                    {pathname === '/scheduled-jobs' && i18n.t('header.scheduled_jobs')}
                    {pathname === '/docs' && i18n.t('header.knowledge_base')}
                    {pathname === '/settings' && i18n.t('header.system_config')}
                </h1>
            </div>

            <div className="flex-1 flex justify-center overflow-hidden">
                <Swarm_Status_Header />
            </div>
            <div className="flex items-center gap-4">
                <Tooltip content={settings.privacy_mode ? i18n.t('settings.desc_privacy_mode_on') : i18n.t('settings.desc_privacy_mode_off')} position="left">
                    <button
                        onClick={toggle_privacy}
                        className={`flex items-center gap-2 px-2 py-1 rounded border transition-all ${settings.privacy_mode
                            ? 'bg-blue-500/10 border-blue-500/50 text-blue-400'
                            : 'bg-zinc-900 border-zinc-800 text-zinc-500 hover:border-zinc-700'
                            }`}
                    >
                        <Shield size={14} className={settings.privacy_mode ? 'animate-pulse' : ''} />
                        <span className="text-[10px] font-bold uppercase tracking-tighter">
                            {settings.privacy_mode ? i18n.t('settings.badge_air_gap') : i18n.t('settings.label_privacy_mode')}
                        </span>
                    </button>
                </Tooltip>

                <Tooltip content={connection_state === 'connected' ? i18n.t('header.ws_connected') : i18n.t('header.engine_offline')} position="left">
                    <div className="flex items-center gap-2 text-xs cursor-help">
                        <span className="flex h-2 w-2 relative">
                            {connection_state === 'connected' && (
                                <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-500 opacity-75"></span>
                            )}
                            {connection_state === 'connecting' && (
                                <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-amber-500 opacity-75"></span>
                            )}
                            <span className={`relative inline-flex rounded-full h-2 w-2 ${connection_state === 'connected' ? 'bg-emerald-500' :
                                connection_state === 'connecting' ? 'bg-amber-500' : 'bg-red-500'
                                }`}></span>
                        </span>
                        <span className={`font-mono font-medium ${connection_state === 'connected' ? 'text-emerald-500' :
                            connection_state === 'connecting' ? 'text-amber-500' : 'text-red-500'
                            }`}>
                            {connection_state === 'connected' ? (
                                engine_health ? i18n.t('header.engine_online_agents', { count: engine_health.agent_count }) : i18n.t('header.engine_online')
                            ) : connection_state === 'connecting' ? i18n.t('header.connecting') : i18n.t('header.engine_offline_label')}
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
});

Org_Header.displayName = 'Org_Header';
