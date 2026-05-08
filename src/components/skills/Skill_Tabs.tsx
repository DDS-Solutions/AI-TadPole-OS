/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Sub-sector navigator for the Capability Forge. 
 * Manages transitions between Skills, Workflows, Hooks, and the MCP Lab with active-state indicators.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Unhandled sector path mismatch, or indicator "stretching" during rapid tab switching.
 * - **Telemetry Link**: Search for `[Skill_Tabs]` or `nav_sector_change` in UI tracing.
 */

import React from 'react';
import { Code, FileText, Shield, Terminal } from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';

interface Skill_Tabs_Props {
    active_tab: 'skills' | 'workflows' | 'hooks' | 'mcp';
    set_active_tab: (tab: 'skills' | 'workflows' | 'hooks' | 'mcp') => void;
}

export const Skill_Tabs: React.FC<Skill_Tabs_Props> = ({ active_tab, set_active_tab }) => {
    const tabs: Array<{ id: 'skills' | 'workflows' | 'hooks' | 'mcp', icon: React.ReactNode, label: string, tooltip: string, themeColor: string }> = [
        { id: 'skills', icon: <Code className="w-4 h-4" />, label: i18n.t('skills.tab_skills'), tooltip: i18n.t('skills.tooltip_skills'), themeColor: 'border-green-500 text-green-400' },
        { id: 'workflows', icon: <FileText className="w-4 h-4" />, label: i18n.t('skills.tab_workflows'), tooltip: i18n.t('skills.tooltip_workflows'), themeColor: 'border-green-500 text-green-400' },
        { id: 'hooks', icon: <Shield className="w-4 h-4" />, label: i18n.t('skills.tab_hooks'), tooltip: i18n.t('skills.tooltip_hooks'), themeColor: 'border-green-500 text-green-400' },
        { id: 'mcp', icon: <Terminal className="w-4 h-4" />, label: i18n.t('skills.tab_mcp'), tooltip: i18n.t('skills.tooltip_mcp'), themeColor: 'border-cyan-500 text-cyan-400' },
    ];

    return (
        <div className="flex border-b border-zinc-800 shrink-0">
            {tabs.map(tab => (
                <Tooltip key={tab.id} content={tab.tooltip} position="bottom">
                    <button
                        onClick={() => set_active_tab(tab.id)}
                        className={`flex items-center gap-2 px-6 py-3 font-medium transition-colors border-b-2 ${active_tab === tab.id ? tab.themeColor : 'border-transparent text-zinc-500 hover:text-zinc-300'}`}
                    >
                        {tab.icon} {tab.label}
                    </button>
                </Tooltip>
            ))}
        </div>
    );
};


// Metadata: [Skill_Tabs]
