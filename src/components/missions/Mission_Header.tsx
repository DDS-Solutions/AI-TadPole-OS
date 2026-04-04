import React from 'react';
import { Zap, ShieldCheck } from 'lucide-react';
import { Tooltip } from '../ui';
import { get_department_icon, get_theme_colors } from '../../utils/agent_uiutils';
import type { Mission_Cluster } from '../../stores/workspace_store';
import { i18n } from '../../i18n';

interface MissionHeaderProps {
    active_cluster: Mission_Cluster;
    agents_loading: boolean;
    has_agents: boolean;
    on_run_mission: () => void;
    on_toggle_analysis: (id: string) => void;
}

export const Mission_Header: React.FC<MissionHeaderProps> = ({
    active_cluster,
    agents_loading,
    has_agents,
    on_run_mission,
    on_toggle_analysis
}) => {
    const theme = get_theme_colors(active_cluster.theme);
    const dept_icon = get_department_icon(active_cluster.department);

    return (
        <div className="p-6 border-b border-zinc-800 bg-zinc-900/50 backdrop-blur flex justify-between items-center relative overflow-hidden">
            <div className={`absolute top-0 left-0 w-1 h-full ${theme.text.replace('text', 'bg')}`} />
            <div>
                <h2 className="text-lg font-bold text-zinc-100 uppercase tracking-tight">{active_cluster.name}</h2>
                <p className="text-xs text-zinc-500 mt-1">{i18n.t('missions.label_root_path')} <code className={`${theme.text}/80`}>{active_cluster.path}</code></p>
            </div>
            <div className="flex items-center gap-3">
                <Tooltip content={i18n.t('missions.tooltip_run')} position="left">
                    <button
                        disabled={agents_loading || !has_agents}
                        onClick={on_run_mission}
                        className={`flex items-center gap-2 px-4 py-2 rounded-xl border ${theme.border} bg-zinc-900 ${theme.text} hover:scale-105 active:scale-95 transition-all font-bold uppercase tracking-tighter shadow-lg ${theme.glow} disabled:opacity-40 disabled:pointer-events-none`}
                    >
                        <Zap size={16} fill="currentColor" />
                        {i18n.t('missions.btn_run')}
                    </button>
                </Tooltip>

                <Tooltip content={i18n.t('missions.tooltip_analysis')} position="top">
                    <div className="flex flex-col items-center gap-1">
                        <button
                            onClick={() => on_toggle_analysis(active_cluster.id)}
                            className={`flex items-center gap-2 px-3 py-1.5 rounded-lg border transition-all text-[10px] font-bold uppercase tracking-wider ${active_cluster.analysis_enabled
                                ? `${theme.text} ${theme.border} bg-zinc-900 shadow-lg`
                                : 'text-zinc-500 border-zinc-800 bg-zinc-950 grayscale'
                                }`}
                        >
                            <ShieldCheck size={14} className={active_cluster.analysis_enabled ? 'animate-pulse' : ''} />
                            {active_cluster.analysis_enabled ? i18n.t('missions.label_analysis_on') : i18n.t('missions.label_analysis_off')}
                        </button>
                    </div>
                </Tooltip>

                <div className={`p-3 bg-zinc-900 rounded-xl border border-zinc-800 ${theme.text}`}>
                    {React.createElement(dept_icon, { size: 24 })}
                </div>
            </div>
        </div>
    );
};
