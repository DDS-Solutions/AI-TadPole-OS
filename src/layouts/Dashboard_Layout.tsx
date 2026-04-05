/**
 * @docs ARCHITECTURE:Interface
 * @docs OPERATIONS_MANUAL:Navigation
 * 
 * ### AI Assist Note
 * **Root Layout**: Orchestrates the primary dashboard container, sidebar, and tabbed navigation interface. 
 * Manages the high-fidelity backdrop blur effects and Z-index layering for global components (Terminal, Detached Windows). 
 * Integrates `tab_store` for multi-tab state persistence and `sidebar_store` for layout responsiveness.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Viewport overflow (if sidebar is pinned on mobile), tab sync mismatch (BroadcastChannel data collision), or layout layout flickering during page transitions.
 * - **Telemetry Link**: Watch for `tab_store` mutations or search `[DashboardLayout]` in UI traces.
 */

import React, { useEffect, useState, Suspense } from 'react';
import { useLocation, useNavigate } from 'react-router-dom';
import clsx from 'clsx';
import { tadpole_os_socket } from '../services/socket';
import { Sovereign_Chat } from '../components/Sovereign_Chat';
import { Lineage_Stream } from '../components/Lineage_Stream';
import { Neural_Waterfall } from '../components/Neural_Waterfall';
import { Command_Palette } from '../components/Command_Palette';
import { use_skill_store } from '../stores/skill_store';
import { Sidebar } from '../components/layout/Sidebar';
import { Page_Header, Portal_Window } from '../components/ui';
import { Tab_Bar } from '../components/layout/Tab_Bar';
import { use_tab_store } from '../stores/tab_store';
import { use_header_store } from '../stores/header_store';
import { get_route_by_path } from '../constants/routes';
import Error_Boundary from '../components/Error_Boundary';
import { ExternalLink } from 'lucide-react';
import { i18n } from '../i18n';
import { System_Log } from '../components/dashboard/System_Log';
import { Observability_Sidebar } from '../components/layout/Observability_Sidebar';
import { Toast_Center } from '../components/ui/Toast_Center';
import { use_notification_store } from '../stores/notification_store';
import { event_bus } from '../services/event_bus';

/**
 * Dashboard_Layout
 * The primary architectural shell for the Tadpole OS frontend.
 * Orchestrates global services, navigation, and multi-window workspace synchronization.
 */
export default function Dashboard_Layout() {
    const location = useLocation();
    const navigate = useNavigate();
    const { 
        tabs, 
        active_tab_id, 
        is_system_log_detached, 
        toggle_system_log_detachment, 
        is_trace_stream_detached, 
        toggle_trace_stream_detachment, 
        is_lineage_stream_detached, 
        toggle_lineage_stream_detachment 
    } = use_tab_store();
    const { actions: header_actions } = use_header_store();
    const [is_command_palette_open, set_is_command_palette_open] = useState(false);

    // Synchronize tab store with URL on first load and browser navigation
    useEffect(() => {
        const active_tab = tabs.find(t => t.id === active_tab_id);
        if (active_tab && active_tab.path !== location.pathname) {
            navigate(active_tab.path);
        }
    // eslint-disable-next-line react-hooks/exhaustive-deps -- navigate on tab switch only, not on every tabs/navigate change
    }, [active_tab_id]);

    // ── Connection Status ──────────────────────────────────
    useEffect(() => {
        tadpole_os_socket.connect();
        const unsubscribe_pulse = tadpole_os_socket.subscribe_pulse((pulse) => {
            use_skill_store.getState().handle_pulse(pulse.tool, pulse.status, pulse.latency);
        });
        return () => {
            unsubscribe_pulse();
        };
    }, []);

    // ── Unified Notification Hub ───────────────────────────
    useEffect(() => {
        const { add_notification } = use_notification_store.getState();

        const unsubscribe = event_bus.subscribe_logs((entry) => {
            // Only pipe errors or specifically tagged security/governance events to the Hub
            const is_high_priority = 
                entry.severity === 'error' || 
                entry.severity === 'warning' ||
                entry.text.toLowerCase().includes('budget') ||
                entry.text.toLowerCase().includes('security') ||
                entry.text.toLowerCase().includes('injection');

            if (is_high_priority) {
                // Determine persistence based on user preference (Manual dismissal for Governance/Security)
                const is_persistent = 
                    entry.text.toLowerCase().includes('budget') || 
                    entry.text.toLowerCase().includes('injection') ||
                    entry.text.toLowerCase().includes('sanitizer');

                add_notification({
                    severity: entry.severity,
                    title: entry.source === 'Agent' ? `Agent Alert: ${entry.agent_name || entry.agent_id}` : 'System Alert',
                    message: entry.text,
                    type_id: entry.metadata?.type_id as string,
                    persistent: is_persistent,
                });
            }
        });

        return unsubscribe;
    }, []);

    // ── Keyboard Shortcuts ──────────────────────────────────
    useEffect(() => {
        const handle_key_down = (e: KeyboardEvent) => {
            const tag = (e.target as HTMLElement)?.tagName;
            if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return;

            if ((e.ctrlKey || e.metaKey) && (e.key === 'k' || e.key === '/')) {
                e.preventDefault();
                set_is_command_palette_open(prev => !prev);
                return;
            }

            if (!e.ctrlKey && !e.metaKey && !e.altKey) {
                const routes: Record<string, string> = {
                    '1': '/',
                    '2': '/org-chart',
                    '3': '/standups',
                    '4': '/workspaces',
                    '5': '/docs',
                    '6': '/settings',
                };
                if (routes[e.key]) {
                    navigate(routes[e.key]);
                    return;
                }
            }
        };
        window.addEventListener('keydown', handle_key_down);
        return () => window.removeEventListener('keydown', handle_key_down);
    }, [navigate]);

    const nav_item_class = ({ isActive: is_active }: { isActive: boolean }) => clsx(
        "flex items-center gap-3 p-2 rounded-md font-medium cursor-pointer transition-all duration-200 text-sm",
        is_active
            ? "bg-zinc-800 text-zinc-100 shadow-inner border border-zinc-700/50"
            : "text-zinc-500 hover:bg-zinc-800/50 hover:text-zinc-300"
    );

    const active_route = get_route_by_path(location.pathname);

    return (
        <div className="flex h-screen bg-zinc-950 text-zinc-100 overflow-hidden font-sans antialiased selection:bg-zinc-700/30">
            <Sidebar nav_item_class={nav_item_class} />

            <main className="flex-1 flex flex-col bg-zinc-950 relative overflow-hidden">
                <Page_Header
                    title={active_route.label || 'Tadpole OS'}
                    subtitle={active_route.path === '/' ? i18n.t('layout.system_active') : ''}
                    actions={header_actions}
                />

                <Tab_Bar />

                <div className="flex-1 flex overflow-hidden">
                    <div className="flex-1 relative">
                        {tabs.map((tab) => {
                            const route = get_route_by_path(tab.path);
                            const Component = route.component;
                            const is_detached = tab.is_detached;
                            const is_active = tab.id === active_tab_id;

                            // Content shared between portal and inline
                            const content = (
                                <Error_Boundary name={`Sector: ${tab.title}`}>
                                    <Suspense fallback={<div className="p-8 text-zinc-500 font-mono text-xs animate-pulse">{i18n.t('layout.initializing_sector')}</div>}>
                                        <Component />
                                    </Suspense>
                                </Error_Boundary>
                            );

                            if (is_detached) {
                                return (
                                    <React.Fragment key={tab.id}>
                                        {/* Render Portal Window (Native Browser Window) */}
                                        <Portal_Window 
                                            id={tab.id} 
                                            title={tab.title} 
                                            on_close={() => use_tab_store.getState().toggle_tab_detachment(tab.id)}
                                        >
                                            <div className="h-screen bg-zinc-950 p-6 flex flex-row gap-6 overflow-hidden">
                                                <div className="flex-1 relative overflow-auto custom-scrollbar">
                                                    {content}
                                                </div>
                                                <Observability_Sidebar is_detached_context />
                                            </div>
                                        </Portal_Window>

                                        {/* Placeholder in Main Layout */}
                                        <div
                                            className={clsx(
                                                "absolute inset-0 flex items-center justify-center bg-zinc-950/80 backdrop-blur-sm z-20",
                                                is_active ? "visible opacity-100" : "invisible opacity-0"
                                            )}
                                        >
                                            <div className="text-center space-y-4">
                                                <div className="relative inline-block">
                                                    <ExternalLink size={48} className="text-zinc-800 animate-pulse" />
                                                    <div className="absolute inset-0 bg-blue-500/10 blur-xl rounded-full" />
                                                </div>
                                                <div className="space-y-1">
                                                    <h3 className="text-lg font-bold tracking-tight text-zinc-200">{i18n.t('layout.sector_detached')}</h3>
                                                    <p className="text-sm text-zinc-500 font-mono">{i18n.t('layout.link_established')} :: {i18n.t('layout.monitor_label')}_{tab.id.toUpperCase()}</p>
                                                </div>
                                                <button 
                                                    onClick={() => use_tab_store.getState().toggle_tab_detachment(tab.id)}
                                                    className="px-4 py-2 bg-zinc-100 text-black text-xs font-bold uppercase tracking-widest rounded-lg hover:bg-white transition-all shadow-lg active:scale-95"
                                                >
                                                    {i18n.t('layout.recall_sector')}
                                                </button>
                                            </div>
                                        </div>
                                    </React.Fragment>
                                );
                            }

                            return (
                                <div
                                    key={tab.id}
                                    className={clsx(
                                        "absolute inset-0 overflow-y-auto overflow-x-hidden p-0 custom-scrollbar",
                                        is_active ? "visible opacity-100 z-10" : "invisible opacity-0 z-0 pointer-events-none"
                                    )}
                                >
                                    {content}
                                </div>
                            );
                        })}
                    </div>

                    <Observability_Sidebar />

                    <Sovereign_Chat />
                    <Command_Palette
                        is_open={is_command_palette_open}
                        on_close={() => set_is_command_palette_open(false)}
                    />

                    {/* Global Detached System Log */}
                    {is_system_log_detached && (
                        <Portal_Window
                            id="system-log-detached"
                            title={i18n.t('dashboard.log_title')}
                            on_close={toggle_system_log_detachment}
                        >
                            <div className="h-screen bg-zinc-950 p-6 flex flex-col">
                                <System_Log is_detached_view />
                            </div>
                        </Portal_Window>
                    )}

                    {/* Global Detached Trace Stream */}
                    {is_trace_stream_detached && (
                        <Portal_Window
                            id="trace-stream-detached"
                            title={i18n.t('trace_stream.title')}
                            on_close={toggle_trace_stream_detachment}
                        >
                            <div className="h-screen bg-zinc-950 p-0 flex flex-col">
                                <Neural_Waterfall is_detached_view />
                            </div>
                        </Portal_Window>
                    )}

                    {/* Global Detached Lineage Stream */}
                    {is_lineage_stream_detached && (
                        <Portal_Window
                            id="lineage-stream-detached"
                            title={i18n.t('trace.stream_title')}
                            on_close={toggle_lineage_stream_detachment}
                        >
                            <div className="h-screen bg-zinc-950 p-0 flex flex-col">
                                <Lineage_Stream is_detached_view />
                            </div>
                        </Portal_Window>
                    )}
                </div>

                <Toast_Center />
            </main>
        </div>
    )
}

