import React from 'react';
import { Zap, ShieldCheck } from 'lucide-react';
import { Tooltip } from '../ui';
import { getDepartmentIcon, getThemeColors } from '../../utils/agentUIUtils';
import type { MissionCluster } from '../../stores/workspaceStore';
import { i18n } from '../../i18n';

interface MissionHeaderProps {
    activeCluster: MissionCluster;
    agentsLoading: boolean;
    hasAgents: boolean;
    onRunMission: () => void;
    onToggleAnalysis: (id: string) => void;
}

export const MissionHeader: React.FC<MissionHeaderProps> = ({
    activeCluster,
    agentsLoading,
    hasAgents,
    onRunMission,
    onToggleAnalysis
}) => {
    const theme = getThemeColors(activeCluster.theme);
    const deptIcon = getDepartmentIcon(activeCluster.department);

    return (
        <div className="p-6 border-b border-zinc-800 bg-zinc-900/50 backdrop-blur flex justify-between items-center relative overflow-hidden">
            <div className={`absolute top-0 left-0 w-1 h-full ${theme.text.replace('text', 'bg')}`} />
            <div>
                <h2 className="text-lg font-bold text-zinc-100 uppercase tracking-tight">{activeCluster.name}</h2>
                <p className="text-xs text-zinc-500 mt-1">{i18n.t('missions.label_root_path')} <code className={`${theme.text}/80`}>{activeCluster.path}</code></p>
            </div>
            <div className="flex items-center gap-3">
                <Tooltip content={i18n.t('missions.tooltip_run')} position="left">
                    <button
                        disabled={agentsLoading || !hasAgents}
                        onClick={onRunMission}
                        className={`flex items-center gap-2 px-4 py-2 rounded-xl border ${theme.border} bg-zinc-900 ${theme.text} hover:scale-105 active:scale-95 transition-all font-bold uppercase tracking-tighter shadow-lg ${theme.glow} disabled:opacity-40 disabled:pointer-events-none`}
                    >
                        <Zap size={16} fill="currentColor" />
                        {i18n.t('missions.btn_run')}
                    </button>
                </Tooltip>

                <Tooltip content={i18n.t('missions.tooltip_analysis')} position="top">
                    <div className="flex flex-col items-center gap-1">
                        <button
                            onClick={() => onToggleAnalysis(activeCluster.id)}
                            className={`flex items-center gap-2 px-3 py-1.5 rounded-lg border transition-all text-[10px] font-bold uppercase tracking-wider ${activeCluster.analysisEnabled
                                ? `${theme.text} ${theme.border} bg-zinc-900 shadow-lg`
                                : 'text-zinc-500 border-zinc-800 bg-zinc-950 grayscale'
                                }`}
                        >
                            <ShieldCheck size={14} className={activeCluster.analysisEnabled ? 'animate-pulse' : ''} />
                            {activeCluster.analysisEnabled ? i18n.t('missions.label_analysis_on') : i18n.t('missions.label_analysis_off')}
                        </button>
                    </div>
                </Tooltip>

                <div className={`p-3 bg-zinc-900 rounded-xl border border-zinc-800 ${theme.text}`}>
                    {React.createElement(deptIcon, { size: 24 })}
                </div>
            </div>
        </div>
    );
};
