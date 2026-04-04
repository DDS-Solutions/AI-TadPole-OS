import React from 'react';
import { Code, FileText, Shield, Terminal } from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';

interface Skill_Tabs_Props {
    active_tab: 'skills' | 'workflows' | 'hooks' | 'mcp';
    set_active_tab: (tab: 'skills' | 'workflows' | 'hooks' | 'mcp') => void;
}

export const Skill_Tabs: React.FC<Skill_Tabs_Props> = ({ active_tab, set_active_tab }) => {
    return (
        <div className="flex border-b border-zinc-800 shrink-0">
            <Tooltip content={i18n.t('skills.tooltip_skills')} position="bottom">
                <button
                    onClick={() => set_active_tab('skills')}
                    className={`flex items-center gap-2 px-6 py-3 font-medium transition-colors border-b-2 ${active_tab === 'skills' ? 'border-blue-500 text-blue-400' : 'border-transparent text-zinc-500 hover:text-zinc-300'
                        }`}
                >
                    <Code className="w-4 h-4" /> {i18n.t('skills.tab_skills')}
                </button>
            </Tooltip>
            <Tooltip content={i18n.t('skills.tooltip_workflows')} position="bottom">
                <button
                    onClick={() => set_active_tab('workflows')}
                    className={`flex items-center gap-2 px-6 py-3 font-medium transition-colors border-b-2 ${active_tab === 'workflows' ? 'border-blue-500 text-blue-400' : 'border-transparent text-zinc-500 hover:text-zinc-300'
                        }`}
                >
                    <FileText className="w-4 h-4" /> {i18n.t('skills.tab_workflows')}
                </button>
            </Tooltip>
            <Tooltip content={i18n.t('skills.tooltip_hooks')} position="bottom">
                <button
                    onClick={() => set_active_tab('hooks')}
                    className={`flex items-center gap-2 px-6 py-3 font-medium transition-colors border-b-2 ${active_tab === 'hooks' ? 'border-blue-500 text-blue-400' : 'border-transparent text-zinc-500 hover:text-zinc-300'
                        }`}
                >
                    <Shield className="w-4 h-4" /> {i18n.t('skills.tab_hooks')}
                </button>
            </Tooltip>
            <Tooltip content={i18n.t('skills.tooltip_mcp')} position="bottom">
                <button
                    onClick={() => set_active_tab('mcp')}
                    className={`flex items-center gap-2 px-6 py-3 font-medium transition-colors border-b-2 ${active_tab === 'mcp' ? 'border-cyan-500 text-cyan-400' : 'border-transparent text-zinc-500 hover:text-zinc-300'
                        }`}
                >
                    <Terminal className="w-4 h-4" /> {i18n.t('skills.tab_mcp')}
                </button>
            </Tooltip>
        </div>
    );
};
