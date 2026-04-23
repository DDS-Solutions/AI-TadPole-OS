/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Sub-nav controller for the Intelligence Layer. 
 * Manages routing to Provider, Agent, Skill, and Telemetry sectors with scale-based hover animations.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Nested route mismatch for skills/benchmarks sub-links, or tooltip occlusion by parent sidebar scroll container.
 * - **Telemetry Link**: Search for `[Intelligence_Nav]` in UI logs.
 */

import React from 'react';
import { NavLink } from 'react-router-dom';
import { Cpu, Users, Settings, Activity, BarChart3 } from 'lucide-react';
import { Tooltip } from './ui';
import { i18n } from '../i18n';

interface IntelligenceNavProps {
    nav_item_class: (props: { isActive: boolean }) => string;
}

export const Intelligence_Nav: React.FC<IntelligenceNavProps> = ({ nav_item_class }): React.ReactElement => {
    // PERF: Prevent inline function allocation on every render
    const skills_nav_class = React.useCallback((props: { isActive: boolean }) => 
        nav_item_class(props) + " ml-2 lg:ml-6 scale-95 opacity-90 border-l-2 border-emerald-500/20 pl-2 lg:pl-3",
        [nav_item_class]
    );

    return (
        <div className="space-y-1">
            <div className="text-[10px] font-bold text-zinc-600 uppercase tracking-widest mb-2 px-2 hidden lg:block">
                {i18n.t('nav.intelligence')}
            </div>
            <Tooltip content={i18n.t('nav.providers_tooltip')} position="right">
                <NavLink to="/models" className={nav_item_class}>
                    <Cpu size={18} />
                    <span className="hidden lg:block">{i18n.t('nav.providers')}</span>
                </NavLink>
            </Tooltip>
            <Tooltip content={i18n.t('nav.agents_tooltip')} position="right">
                <NavLink to="/agents" className={nav_item_class}>
                    <Users size={18} />
                    <span className="hidden lg:block">{i18n.t('nav.agents')}</span>
                </NavLink>
            </Tooltip>
            <Tooltip content={i18n.t('nav.skills_tooltip')} position="right">
                <NavLink to="/skills" className={skills_nav_class}>
                    <Settings size={16} />
                    <span className="hidden lg:block text-xs">{i18n.t('nav.skills')}</span>
                </NavLink>
            </Tooltip>
            <Tooltip content={i18n.t('nav.telemetry_tooltip')} position="right">
                <NavLink to="/engine" className={nav_item_class}>
                    <Activity size={18} />
                    <span className="hidden lg:block">{i18n.t('nav.telemetry')}</span>
                </NavLink>
            </Tooltip>
            <Tooltip content={i18n.t('nav.performance_tooltip')} position="right">
                <NavLink to="/benchmarks" className={nav_item_class}>
                    <BarChart3 size={18} />
                    <span className="hidden lg:block">{i18n.t('nav.performance')}</span>
                </NavLink>
            </Tooltip>
        </div>
    );
};


// Metadata: [Intelligence_Nav]
