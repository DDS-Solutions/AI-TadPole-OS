/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **Root Navigation Registry**: Map of all interactive dashboard coordinates and their corresponding components. 
 * Orchestrates lazy-loading of domain pages (Missions, Engine, Oversight) to minimize initial bundle footprint and ensure responsive transitions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: 404 Route mismatch (if `get_route_by_path` fallback fails), or ChunkLoadError during lazy-loading on slow networks.
 * - **Telemetry Link**: Search for `Route_Config` or `APP_ROUTES` in navigation trace spans.
 */

import { lazy } from 'react';

// Lazy-loaded pages
const Org_Chart = lazy(() => import('../pages/Org_Chart'));
const Standups = lazy(() => import('../pages/Standups'));
const Workspaces = lazy(() => import('../pages/Workspaces'));
const Docs = lazy(() => import('../pages/Docs'));
const Settings = lazy(() => import('../pages/Settings'));
const Oversight_Dashboard = lazy(() => import('../pages/Oversight_Dashboard'));
const Model_Manager = lazy(() => import('../pages/Model_Manager'));
const Agent_Manager = lazy(() => import('../pages/Agent_Manager'));
const Engine_Dashboard = lazy(() => import('../pages/Engine_Dashboard'));
const Missions = lazy(() => import('../pages/Missions'));
const Skills = lazy(() => import('../pages/Skills'));
const Benchmark_Analytics = lazy(() => import('../pages/Benchmark_Analytics'));
const Scheduled_Jobs = lazy(() => import('../pages/Scheduled_Jobs'));
const Template_Store = lazy(() => import('../pages/Template_Store'));
const Security_Dashboard = lazy(() => import('../pages/Security_Dashboard'));
const Ops_Dashboard = lazy(() => import('../pages/Ops_Dashboard'));
const Model_Store = lazy(() => import('../pages/Model_Store'));

export interface Route_Config {
  path: string;
  component: React.ComponentType<object>;
  label: string;
  icon?: string;
}

export const APP_ROUTES: Route_Config[] = [
  { path: '/dashboard', component: Ops_Dashboard, label: 'Operations', icon: 'LayoutDashboard' },
  { path: '/org-chart', component: Org_Chart, label: 'Hierarchy', icon: 'Users' },
  { path: '/standups', component: Standups, label: 'Standups', icon: 'MessagesSquare' },
  { path: '/workspaces', component: Workspaces, label: 'Workspaces', icon: 'Grid' },
  { path: '/missions', component: Missions, label: 'Missions', icon: 'Target' },
  { path: '/models', component: Model_Manager, label: 'Models', icon: 'Cpu' },
  { path: '/agents', component: Agent_Manager, label: 'Agents', icon: 'Bot' },
  { path: '/engine', component: Engine_Dashboard, label: 'Engine', icon: 'Zap' },
  { path: '/oversight', component: Oversight_Dashboard, label: 'Oversight', icon: 'Shield' },
  { path: '/skills', component: Skills, label: 'Skills', icon: 'Wrench' },
  { path: '/benchmarks', component: Benchmark_Analytics, label: 'Benchmarks', icon: 'BarChart' },
  { path: '/scheduled-jobs', component: Scheduled_Jobs, label: 'Jobs', icon: 'Clock' },
  { path: '/infra/model-store', component: Model_Store, label: 'Intelligence Store', icon: 'Store' },
  { path: '/docs', component: Docs, label: 'Docs', icon: 'BookOpen' },
  { path: '/settings', component: Settings, label: 'Settings', icon: 'Settings' },
  { path: '/store', component: Template_Store, label: 'Store', icon: 'ShoppingBag' },
  { path: '/security', component: Security_Dashboard, label: 'Security', icon: 'Lock' },
];

export const get_route_by_path = (path: string) => {
  const normalized = path === '/' ? '/dashboard' : path.replace(/\/$/, '');
  return APP_ROUTES.find(r => r.path === normalized) || APP_ROUTES[0];
};

