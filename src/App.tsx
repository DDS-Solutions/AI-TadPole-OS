/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **Root Application Kernel**: Top-level React entry point that orchestrates routing, global state synchronization, and the `Dashboard_Layout` shell. 
 * Manages the unified theme injection, default provider syncing, and URL-to-Tab state reconciliation.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Route dead-loops (if `TabSync` logic fails), theme injection flicker, or `Error_Boundary` catch during lazy-loading.
 * - **Telemetry Link**: Look for `Global OS Hub` in error traces or search `[AppKernel]` in component logs.
 */

import { useEffect, Suspense } from 'react';
import { BrowserRouter as Router, Routes, Route, useLocation, Navigate } from 'react-router-dom';
import { use_provider_store } from './stores/provider_store';
import { get_settings } from './stores/settings_store';
import { use_tab_store } from './stores/tab_store';
import { get_route_by_path } from './constants/routes';
import Dashboard_Layout from './layouts/Dashboard_Layout';
import Error_Boundary from './components/Error_Boundary';

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
  const open_tab = use_tab_store(s => s.open_tab); 
  const active_tab_id = use_tab_store(s => s.active_tab_id);
  const tabs = use_tab_store(s => s.tabs);
  
  useEffect(() => {
    const route = get_route_by_path(location.pathname);
    if (!route) return;

    // Preventive check: don't trigger open_tab if the active tab already matches the path
    const active_tab = tabs.find(t => t.id === active_tab_id);
    const normalized_target = route.path === '/' ? '/dashboard' : route.path.replace(/\/$/, '');
    const normalized_active = active_tab ? (active_tab.path === '/' ? '/dashboard' : active_tab.path.replace(/\/$/, '')) : null;

    if (normalized_active === normalized_target) {
      return;
    }

    open_tab({
        title: route.label,
        path: route.path,
        icon: route.icon
    });
  }, [location.pathname, open_tab, active_tab_id, tabs]);

  return null;
}

export default function App(): React.ReactElement {
  const sync_defaults = use_provider_store(state => state.sync_defaults);
  const sync_with_backend = use_provider_store(state => state.sync_with_backend);

  useEffect(() => {
    sync_defaults();
    sync_with_backend();

    const settings = get_settings();
    document.documentElement.setAttribute('data-theme', settings.theme);
    document.documentElement.setAttribute('data-density', settings.density);
  }, [sync_defaults, sync_with_backend]);


  return (
    <Router>
      <TabSync />
      <Error_Boundary name="Global OS Hub">
        <Suspense fallback={<RouteLoading />}>
          <Routes>
            {/* Main Application Layout */}
            <Route path="/*" element={<Dashboard_Layout />}>
              {/* 
                We redirect / to /dashboard specifically.
                All other paths (e.g. /agents, /telemetry) are handled 
                by Dashboard_Layout's custom tab system. 
              */}
              <Route index element={<Navigate to="/dashboard" replace />} />
            </Route>

            {/* Error Fallback */}
            <Route path="*" element={<Navigate to="/dashboard" replace />} />
          </Routes>
        </Suspense>
      </Error_Boundary>
    </Router>
  );
}

