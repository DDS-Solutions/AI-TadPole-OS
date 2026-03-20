import React from 'react';
import { NavLink } from 'react-router-dom';
import { Cpu, Users, Wrench, Activity, BarChart3 } from 'lucide-react';
import { Tooltip } from './ui';
import { i18n } from '../i18n';

interface IntelligenceNavProps {
    navItemClass: (props: { isActive: boolean }) => string;
}

export const IntelligenceNav: React.FC<IntelligenceNavProps> = ({ navItemClass }): React.ReactElement => {
    return (
        <div className="space-y-1">
            <div className="text-[10px] font-bold text-zinc-600 uppercase tracking-widest mb-2 px-2 hidden lg:block">
                {i18n.t('nav.intelligence')}
            </div>
            <Tooltip content={i18n.t('nav.providers_tooltip')} position="right">
                <NavLink to="/models" className={navItemClass}>
                    <Cpu size={18} />
                    <span className="hidden lg:block">{i18n.t('nav.providers')}</span>
                </NavLink>
            </Tooltip>
            <Tooltip content={i18n.t('nav.agents_tooltip')} position="right">
                <NavLink to="/agents" className={navItemClass}>
                    <Users size={18} />
                    <span className="hidden lg:block">{i18n.t('nav.agents')}</span>
                </NavLink>
            </Tooltip>
            <Tooltip content={i18n.t('nav.capabilities_tooltip')} position="right">
                <NavLink to="/capabilities" className={navItemClass}>
                    <Wrench size={18} />
                    <span className="hidden lg:block">{i18n.t('nav.capabilities')}</span>
                </NavLink>
            </Tooltip>
            <Tooltip content={i18n.t('nav.telemetry_tooltip')} position="right">
                <NavLink to="/engine" className={navItemClass}>
                    <Activity size={18} />
                    <span className="hidden lg:block">{i18n.t('nav.telemetry')}</span>
                </NavLink>
            </Tooltip>
            <Tooltip content={i18n.t('nav.performance_tooltip')} position="right">
                <NavLink to="/benchmarks" className={navItemClass}>
                    <BarChart3 size={18} />
                    <span className="hidden lg:block">{i18n.t('nav.performance')}</span>
                </NavLink>
            </Tooltip>
        </div>
    );
};
