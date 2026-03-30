import { useEffect, Suspense } from 'react';
import { BrowserRouter as Router, Routes, Route, useLocation, Navigate } from 'react-router-dom';
import { useProviderStore } from './stores/providerStore';
import { getSettings } from './stores/settingsStore';
import { useTabStore } from './stores/tabStore';
import { getRouteByPath } from './constants/routes';
import DashboardLayout from './layouts/DashboardLayout';
import ErrorBoundary from './components/ErrorBoundary';

import { i18n } from './i18n';

function RouteLoading(): React.ReactElement {
  return (
    <div className="h-full flex flex-col items-center justify-center gap-4 p-8">
      <div className="w-full max-w-2xl space-y-3">
        <div className="h-6 w-48 bg-zinc-800 rounded animate-pulse" />
        <div className="h-4 w-full bg-zinc-800/60 rounded animate-pulse" />
        <div className="h-4 w-5/6 bg-zinc-800/40 rounded animate-pulse" />
        <div className="h-32 w-full bg-zinc-800/30 rounded-lg animate-pulse mt-4" />
        <div className="flex gap-3 mt-4">
          <div className="h-10 w-28 bg-zinc-800/50 rounded animate-pulse" />
          <div className="h-10 w-28 bg-zinc-800/50 rounded animate-pulse" />
        </div>
      </div>
      <span className="text-zinc-600 animate-pulse font-mono text-xs uppercase tracking-widest mt-4">
        {i18n.t('common.loading')}
      </span>
    </div>
  );
}

// Internal component to sync URL with Tab Store
function TabSync(): null {
  const location = useLocation();
  const openTab = useTabStore(s => s.openTab); 
  const activeTabId = useTabStore(s => s.activeTabId);
  const tabs = useTabStore(s => s.tabs);
  
  useEffect(() => {
    const route = getRouteByPath(location.pathname);
    if (!route) return;

    // Preventive check: don't trigger openTab if the active tab already matches the path
    const activeTab = tabs.find(t => t.id === activeTabId);
    const normalizedTarget = route.path === '/' ? '/dashboard' : route.path.replace(/\/$/, '');
    const normalizedActive = activeTab ? (activeTab.path === '/' ? '/dashboard' : activeTab.path.replace(/\/$/, '')) : null;

    if (normalizedActive === normalizedTarget) {
      return;
    }

    openTab({
        title: route.label,
        path: route.path,
        icon: route.icon
    });
  }, [location.pathname, openTab, activeTabId, tabs]);

  return null;
}

export default function App(): React.ReactElement {
  const syncDefaults = useProviderStore(state => state.syncDefaults);
  const syncWithBackend = useProviderStore(state => state.syncWithBackend);

  useEffect(() => {
    syncDefaults();
    syncWithBackend();

    const settings = getSettings();
    document.documentElement.setAttribute('data-theme', settings.theme);
    document.documentElement.setAttribute('data-density', settings.density);
  }, [syncDefaults, syncWithBackend]);


  return (
    <Router>
      <TabSync />
      <ErrorBoundary name="Global OS Hub">
        <Suspense fallback={<RouteLoading />}>
          <Routes>
            {/* Main Application Layout */}
            <Route path="/*" element={<DashboardLayout />}>
              {/* 
                We redirect / to /dashboard specifically.
                All other paths (e.g. /agents, /telemetry) are handled 
                by DashboardLayout's custom tab system. 
              */}
              <Route index element={<Navigate to="/dashboard" replace />} />
            </Route>

            {/* Error Fallback */}
            <Route path="*" element={<Navigate to="/dashboard" replace />} />
          </Routes>
        </Suspense>
      </ErrorBoundary>
    </Router>
  );
}


