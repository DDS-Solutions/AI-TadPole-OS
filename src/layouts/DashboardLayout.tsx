import React, { useEffect, useState, Suspense } from 'react';
import { useLocation, useNavigate } from 'react-router-dom';
import clsx from 'clsx';
import { TadpoleOSSocket, type ConnectionState } from '../services/socket';
import { SovereignChat } from '../components/SovereignChat';
import { LineageStream } from '../components/LineageStream';
import { NeuralWaterfall } from '../components/NeuralWaterfall';
import { CommandPalette } from '../components/CommandPalette';
import { useSkillStore } from '../stores/skillStore';
import { Sidebar } from '../components/layout/Sidebar';
import { PageHeader, PortalWindow, ConnectionBanner } from '../components/ui';
import { TabBar } from '../components/layout/TabBar';
import { useTabStore } from '../stores/tabStore';
import { useHeaderStore } from '../stores/headerStore';
import { getRouteByPath } from '../constants/routes';
import ErrorBoundary from '../components/ErrorBoundary';
import { ExternalLink } from 'lucide-react';
import { i18n } from '../i18n';
import { SystemLog } from '../components/dashboard/SystemLog';
import { ObservabilitySidebar } from '../components/layout/ObservabilitySidebar';
import { ToastCenter } from '../components/ui/ToastCenter';
import { useNotificationStore } from '../stores/notificationStore';
import { EventBus } from '../services/eventBus';

export default function DashboardLayout() {
    const location = useLocation();
    const navigate = useNavigate();
    const { tabs, activeTabId, isSystemLogDetached, toggleSystemLogDetachment, isTraceStreamDetached, toggleTraceStreamDetachment, isLineageStreamDetached, toggleLineageStreamDetachment } = useTabStore();
    const { actions: headerActions } = useHeaderStore();
    const [connectionState, setConnectionState] = useState<ConnectionState>('disconnected');
    const [engineHealth, setEngineHealth] = useState<{ uptime: number; agentCount: number } | null>(null);
    const [isCommandPaletteOpen, setIsCommandPaletteOpen] = useState(false);

    // Synchronize tab store with URL on first load and browser navigation
    useEffect(() => {
        const activeTab = tabs.find(t => t.id === activeTabId);
        if (activeTab && activeTab.path !== location.pathname) {
            navigate(activeTab.path);
        }
    // eslint-disable-next-line react-hooks/exhaustive-deps -- navigate on tab switch only, not on every tabs/navigate change
    }, [activeTabId]);

    // ── Connection Status ──────────────────────────────────
    useEffect(() => {
        TadpoleOSSocket.connect();
        const unsubscribeStatus = TadpoleOSSocket.subscribeStatus((state) => {
            setConnectionState(state);
            if (state !== 'connected') setEngineHealth(null);
        });
        const unsubscribeHealth = TadpoleOSSocket.subscribeHealth((health) => {
            setEngineHealth({ uptime: health.uptime || 0, agentCount: health.agentCount || 0 });
        });
        const unsubscribePulse = TadpoleOSSocket.subscribePulse((pulse) => {
            useSkillStore.getState().handlePulse(pulse.tool, pulse.status, pulse.latency);
        });
        return () => {
            unsubscribeStatus();
            unsubscribeHealth();
            unsubscribePulse();
        };
    }, []);

    // ── Unified Notification Hub ───────────────────────────
    useEffect(() => {
        const { addNotification } = useNotificationStore.getState();

        const unsubscribe = EventBus.subscribe((entry) => {
            // Only pipe errors or specifically tagged security/governance events to the Hub
            const isHighPriority = 
                entry.severity === 'error' || 
                entry.severity === 'warning' ||
                entry.text.toLowerCase().includes('budget') ||
                entry.text.toLowerCase().includes('security') ||
                entry.text.toLowerCase().includes('injection');

            if (isHighPriority) {
                // Determine persistence based on user preference (Manual dismissal for Governance/Security)
                const isPersistent = 
                    entry.text.toLowerCase().includes('budget') || 
                    entry.text.toLowerCase().includes('injection') ||
                    entry.text.toLowerCase().includes('sanitizer');

                addNotification({
                    severity: entry.severity,
                    title: entry.source === 'Agent' ? `Agent Alert: ${entry.agentName || entry.agentId}` : 'System Alert',
                    message: entry.text,
                    typeId: entry.typeId,
                    persistent: isPersistent,
                });
            }
        });

        return unsubscribe;
    }, []);

    // ── Keyboard Shortcuts ──────────────────────────────────
    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            const tag = (e.target as HTMLElement)?.tagName;
            if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return;

            if ((e.ctrlKey || e.metaKey) && (e.key === 'k' || e.key === '/')) {
                e.preventDefault();
                setIsCommandPaletteOpen(prev => !prev);
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
        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [navigate]);

    const navItemClass = ({ isActive }: { isActive: boolean }) => clsx(
        "flex items-center gap-3 p-2 rounded-md font-medium cursor-pointer transition-all duration-200 text-sm",
        isActive
            ? "bg-zinc-800 text-zinc-100 shadow-inner border border-zinc-700/50"
            : "text-zinc-500 hover:bg-zinc-800/50 hover:text-zinc-300"
    );

    return (
        <div className="flex h-screen bg-zinc-950 text-zinc-100 overflow-hidden font-sans antialiased selection:bg-zinc-700/30">
            <Sidebar navItemClass={navItemClass} />

            <main className="flex-1 flex flex-col bg-zinc-950 relative overflow-hidden">
                <ConnectionBanner />
                <PageHeader
                    pathname={location.pathname}
                    connectionState={connectionState}
                    engineHealth={engineHealth}
                    actions={headerActions}
                />

                <TabBar />

                <div className="flex-1 flex overflow-hidden">
                    <div className="flex-1 relative">
                        {tabs.map((tab) => {
                            const route = getRouteByPath(tab.path);
                            const Component = route.component;
                            const isDetached = tab.isDetached;
                            const isActive = tab.id === activeTabId;

                            // Content shared between portal and inline
                            const content = (
                                <ErrorBoundary name={`Sector: ${tab.title}`}>
                                    <Suspense fallback={<div className="p-8 text-zinc-500 font-mono text-xs animate-pulse">{i18n.t('layout.initializing_sector')}</div>}>
                                        <Component />
                                    </Suspense>
                                </ErrorBoundary>
                            );

                            if (isDetached) {
                                return (
                                    <React.Fragment key={tab.id}>
                                        {/* Render Portal Window (Native Browser Window) */}
                                        <PortalWindow 
                                            id={tab.id} 
                                            title={tab.title} 
                                            onClose={() => useTabStore.getState().toggleTabDetachment(tab.id)}
                                        >
                                            <div className="h-screen bg-zinc-950 p-6 flex flex-row gap-6 overflow-hidden">
                                                <div className="flex-1 relative overflow-auto custom-scrollbar">
                                                    {content}
                                                </div>
                                                <ObservabilitySidebar isDetachedContext />
                                            </div>
                                        </PortalWindow>

                                        {/* Placeholder in Main Layout */}
                                        <div
                                            className={clsx(
                                                "absolute inset-0 flex items-center justify-center bg-zinc-950/80 backdrop-blur-sm z-20",
                                                isActive ? "visible opacity-100" : "invisible opacity-0"
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
                                                    onClick={() => useTabStore.getState().toggleTabDetachment(tab.id)}
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
                                        isActive ? "visible opacity-100 z-10" : "invisible opacity-0 z-0 pointer-events-none"
                                    )}
                                >
                                    {content}
                                </div>
                            );
                        })}
                    </div>

                    <ObservabilitySidebar />

                    <SovereignChat />
                    <CommandPalette
                        isOpen={isCommandPaletteOpen}
                        onClose={() => setIsCommandPaletteOpen(false)}
                    />

                    {/* Global Detached System Log */}
                    {isSystemLogDetached && (
                        <PortalWindow
                            id="system-log-detached"
                            title={i18n.t('dashboard.log_title')}
                            onClose={toggleSystemLogDetachment}
                        >
                            <div className="h-screen bg-zinc-950 p-6 flex flex-col">
                                <SystemLog isDetachedView />
                            </div>
                        </PortalWindow>
                    )}

                    {/* Global Detached Trace Stream */}
                    {isTraceStreamDetached && (
                        <PortalWindow
                            id="trace-stream-detached"
                            title={i18n.t('trace_stream.title')}
                            onClose={toggleTraceStreamDetachment}
                        >
                            <div className="h-screen bg-zinc-950 p-0 flex flex-col">
                                <NeuralWaterfall isDetachedView />
                            </div>
                        </PortalWindow>
                    )}

                    {/* Global Detached Lineage Stream */}
                    {isLineageStreamDetached && (
                        <PortalWindow
                            id="lineage-stream-detached"
                            title={i18n.t('trace.stream_title')}
                            onClose={toggleLineageStreamDetachment}
                        >
                            <div className="h-screen bg-zinc-950 p-0 flex flex-col">
                                <LineageStream isDetachedView />
                            </div>
                        </PortalWindow>
                    )}
                </div>

                <ToastCenter />
            </main>
        </div>
    )
}


