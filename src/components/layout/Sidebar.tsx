/**
 * @docs ARCHITECTURE:Interface
 * @docs OPERATIONS_MANUAL:Navigation
 * 
 * ### AI Assist Note
 * **UI Shell Sidebar**: Core navigational component for Sector switching and Dashboard orchestration. 
 * Implements high-fidelity glassmorphism design with responsive collapse/expand triggers via `sidebar_store`.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Active route mismatch if `location` sync fails, button click hijacking on mobile viewports, or CSS layout shift during rapid collapse/expand.
 * - **Telemetry Link**: Search for `[Sidebar]` or `nav_item_click` in UI tracing.
 */

import { NavLink } from 'react-router-dom';
import { LayoutDashboard, Users, Target, BookOpen, Settings, Shield, Clock, Zap } from 'lucide-react';
import clsx from 'clsx';
import { Intelligence_Nav } from '../Intelligence_Nav';
import { Asset_Nav } from '../Asset_Nav';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';

interface SidebarProps {
    nav_item_class: (props: { isActive: boolean }) => string;
}

export function Sidebar({ nav_item_class }: SidebarProps) {
    const CORE_OPS_ITEMS = [
        { to: '/dashboard', icon: LayoutDashboard, label: 'sidebar.ops_nav', tooltip: 'sidebar.ops_tooltip', end: true },
        { to: '/org-chart', icon: Users, label: 'sidebar.hierarchy_nav', tooltip: 'sidebar.hierarchy_tooltip' },
        { to: '/missions', icon: Target, label: 'sidebar.missions_nav', tooltip: 'sidebar.missions_tooltip' },
        { to: '/scheduled-jobs', icon: Clock, label: 'sidebar.scheduled_jobs_nav', tooltip: 'sidebar.scheduled_jobs_tooltip' },
        { to: '/oversight', icon: Shield, label: 'sidebar.oversight_nav', tooltip: 'sidebar.oversight_tooltip' }
    ];

    return (
        <aside className="w-16 lg:w-56 bg-zinc-950 border-r border-zinc-800 flex flex-col z-20 transition-all duration-300">
            <div className="p-4 lg:p-6 border-b border-zinc-800 flex items-center gap-3 justify-center lg:justify-start">
                <Tooltip content={i18n.t('sidebar.root_tooltip')} position="right">
                    <div className="w-10 h-10 rounded-xl flex items-center justify-center overflow-hidden bg-zinc-900/50 border border-zinc-800 shrink-0 shadow-lg group cursor-pointer">
                        <Zap
                            size={20}
                            aria-label={i18n.t('sidebar.brand_name')}
                            className="text-zinc-100 transition-transform duration-500 group-hover:scale-110"
                        />
                    </div>
                </Tooltip>
                <span className="font-bold text-lg tracking-tight hidden lg:block text-zinc-100">{i18n.t('sidebar.brand_name')}</span>
            </div>

            <nav className="flex-1 p-3 space-y-4 mt-4 overflow-y-auto custom-scrollbar">
                {/* CORE OPERATIONS */}
                <div className="space-y-1">
                    <div className="text-[10px] font-bold text-zinc-600 uppercase tracking-widest mb-2 px-2 hidden lg:block">{i18n.t('sidebar.core_ops_label')}</div>
                    {CORE_OPS_ITEMS.map((item) => (
                        <Tooltip key={item.to} content={i18n.t(item.tooltip)} position="right">
                            <NavLink to={item.to} end={item.end} className={nav_item_class}>
                                <item.icon size={18} />
                                <span className="hidden lg:block">{i18n.t(item.label)}</span>
                            </NavLink>
                        </Tooltip>
                    ))}
                </div>

                {/* COMMUNICATION & ASSETS */}
                <Asset_Nav nav_item_class={nav_item_class} />

                {/* INTELLIGENCE LAYER */}
                <Intelligence_Nav nav_item_class={nav_item_class} />

                <div className="pt-2 border-t border-zinc-800 mx-2"></div>

                <Tooltip content={i18n.t('sidebar.docs_tooltip')} position="right">
                    <NavLink to="/docs" className={nav_item_class}>
                        <BookOpen size={18} />
                        <span className="hidden lg:block">{i18n.t('sidebar.docs_nav')}</span>
                    </NavLink>
                </Tooltip>
            </nav>

            {/* Social Proof / Certification Badge */}
            <div className="p-4 border-t border-zinc-900 bg-zinc-950/80 backdrop-blur-sm hidden lg:block">
                <Tooltip content={i18n.t('sidebar.node_v_tooltip')} position="top">
                    <div className="flex items-center gap-2 p-3 rounded-xl bg-emerald-500/5 border border-emerald-500/10 hover:border-emerald-500/30 transition-all group cursor-default">
                        <Shield className="w-5 h-5 text-emerald-500 group-hover:animate-pulse" />
                        <div className="flex flex-col">
                            <span className="text-[9px] font-bold text-emerald-500 uppercase tracking-widest leading-none">{i18n.t('sidebar.node_certified')}</span>
                            <span className="text-[8px] text-zinc-500 font-mono mt-1">{i18n.t('sidebar.node_version')}</span>
                        </div>
                    </div>
                </Tooltip>
            </div>

            <div className="p-4 border-t border-zinc-800">
                <Tooltip content={i18n.t('sidebar.settings_tooltip')} position="right">
                    <NavLink to="/settings" className={({ isActive: is_active }) => clsx(
                        "flex items-center gap-3 p-2 rounded-md font-medium cursor-pointer transition-all duration-200 text-sm justify-center lg:justify-start",
                        is_active ? "bg-zinc-800 text-zinc-100 shadow-inner border border-zinc-700/50" : "text-zinc-500 hover:bg-zinc-800/50 hover:text-zinc-300"
                    )}>
                        <Settings size={18} />
                        <span className="hidden lg:block">{i18n.t('sidebar.settings_nav')}</span>
                    </NavLink>
                </Tooltip>
            </div>
        </aside>
    );
}


// Metadata: [Sidebar]
