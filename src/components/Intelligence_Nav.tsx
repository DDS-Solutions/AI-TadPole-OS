/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / General / Intelligence_Nav
 * - **Primary Entrypoints**: `Intelligence_Nav`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import React from 'react';
import { NavLink } from 'react-router-dom';
import { Cpu, Users, Settings, Activity, BarChart3, LineChart, Store, ShoppingBag, Network, Server } from 'lucide-react';
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
                {i18n.t('NAV_INTELLIGENCE')}
            </div>
            <Tooltip content={i18n.t('NAV_PROVIDERS_TOOLTIP')} position="right">
                <NavLink to="/models" className={nav_item_class}>
                    <Cpu size={18} />
                    <span className="hidden lg:block">{i18n.t('NAV_PROVIDERS')}</span>
                </NavLink>
            </Tooltip>
            <Tooltip content={i18n.t('NAV_MODEL_STORE_TOOLTIP')} position="right">
                <NavLink to="/infra/model-store" className={nav_item_class}>
                    <Store size={18} />
                    <span className="hidden lg:block">{i18n.t('NAV_MODEL_STORE')}</span>
                </NavLink>
            </Tooltip>
            <Tooltip content={i18n.t('NAV_TEMPLATE_STORE_TOOLTIP')} position="right">
                <NavLink to="/store" className={nav_item_class}>
                    <ShoppingBag size={18} />
                    <span className="hidden lg:block">{i18n.t('NAV_TEMPLATE_STORE')}</span>
                </NavLink>
            </Tooltip>
            <Tooltip content={i18n.t('NAV_MCP_STORE_TOOLTIP')} position="right">
                <NavLink to="/mcp-store" className={nav_item_class}>
                    <Server size={18} />
                    <span className="hidden lg:block">{i18n.t('NAV_MCP_STORE')}</span>
                </NavLink>
            </Tooltip>
            <Tooltip content="Outward Customer Catalog & A2A Gateway Manager" position="right">
                <NavLink to="/customer-catalog" className={nav_item_class}>
                    <Store size={18} />
                    <span className="hidden lg:block">Customer Catalog</span>
                </NavLink>
            </Tooltip>
            <Tooltip content={i18n.t('NAV_NEURAL_MAP_TOOLTIP')} position="right">
                <NavLink to="/intelligence/map" className={nav_item_class}>
                    <Network size={18} />
                    <span className="hidden lg:block">{i18n.t('NAV_NEURAL_MAP')}</span>
                </NavLink>
            </Tooltip>
            <Tooltip content={i18n.t('NAV_AGENTS')} position="right">
                <NavLink to="/agents" className={nav_item_class}>
                    <Users size={18} />
                    <span className="hidden lg:block">{i18n.t('NAV_AGENTS')}</span>
                </NavLink>
            </Tooltip>
            <Tooltip content={i18n.t('NAV_SKILLS_TOOLTIP')} position="right">
                <NavLink to="/skills" className={skills_nav_class}>
                    <Settings size={16} />
                    <span className="hidden lg:block">{i18n.t('NAV_SKILLS')}</span>
                </NavLink>
            </Tooltip>
            <Tooltip content={i18n.t('NAV_TELEMETRY_TOOLTIP')} position="right">
                <NavLink to="/engine" className={nav_item_class}>
                    <Activity size={18} />
                    <span className="hidden lg:block">{i18n.t('NAV_TELEMETRY')}</span>
                </NavLink>
            </Tooltip>
            <Tooltip content={i18n.t('NAV_PERFORMANCE_TOOLTIP')} position="right">
                <NavLink to="/benchmarks" className={nav_item_class}>
                    <BarChart3 size={18} />
                    <span className="hidden lg:block">{i18n.t('NAV_PERFORMANCE')}</span>
                </NavLink>
            </Tooltip>
            <Tooltip content="Benchmark & Dual Trace Swarm Analytics" position="right">
                <NavLink to="/analytics" className={nav_item_class}>
                    <LineChart size={18} />
                    <span className="hidden lg:block">Analytics</span>
                </NavLink>
            </Tooltip>
        </div>
    );
};
