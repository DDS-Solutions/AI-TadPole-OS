import { lazy } from 'react';

// Lazy-loaded pages
const OrgChart = lazy(() => import('../pages/OrgChart'));
const Standups = lazy(() => import('../pages/Standups'));
const Workspaces = lazy(() => import('../pages/Workspaces'));
const Docs = lazy(() => import('../pages/Docs'));
const Settings = lazy(() => import('../pages/Settings'));
const OversightDashboard = lazy(() => import('../pages/OversightDashboard'));
const ModelManager = lazy(() => import('../pages/ModelManager'));
const AgentManager = lazy(() => import('../pages/AgentManager'));
const EngineDashboard = lazy(() => import('../pages/EngineDashboard'));
const Missions = lazy(() => import('../pages/Missions'));
const Capabilities = lazy(() => import('../pages/Capabilities'));
const BenchmarkAnalytics = lazy(() => import('../pages/BenchmarkAnalytics'));
const ScheduledJobs = lazy(() => import('../pages/ScheduledJobs'));
const TemplateStore = lazy(() => import('../pages/TemplateStore'));
const SecurityDashboard = lazy(() => import('../pages/SecurityDashboard'));
const OpsDashboard = lazy(() => import('../pages/OpsDashboard'));

export interface RouteConfig {
  path: string;
  component: React.ComponentType<object>;
  label: string;
  icon?: string;
}

export const APP_ROUTES: RouteConfig[] = [
  { path: '/dashboard', component: OpsDashboard, label: 'Operations', icon: 'LayoutDashboard' },
  { path: '/org-chart', component: OrgChart, label: 'Hierarchy', icon: 'Users' },
  { path: '/standups', component: Standups, label: 'Standups', icon: 'MessagesSquare' },
  { path: '/workspaces', component: Workspaces, label: 'Workspaces', icon: 'Grid' },
  { path: '/missions', component: Missions, label: 'Missions', icon: 'Target' },
  { path: '/models', component: ModelManager, label: 'Models', icon: 'Cpu' },
  { path: '/agents', component: AgentManager, label: 'Agents', icon: 'Bot' },
  { path: '/engine', component: EngineDashboard, label: 'Engine', icon: 'Zap' },
  { path: '/oversight', component: OversightDashboard, label: 'Oversight', icon: 'Shield' },
  { path: '/capabilities', component: Capabilities, label: 'Skills', icon: 'Wrench' },
  { path: '/benchmarks', component: BenchmarkAnalytics, label: 'Benchmarks', icon: 'BarChart' },
  { path: '/scheduled-jobs', component: ScheduledJobs, label: 'Jobs', icon: 'Clock' },
  { path: '/docs', component: Docs, label: 'Docs', icon: 'BookOpen' },
  { path: '/settings', component: Settings, label: 'Settings', icon: 'Settings' },
  { path: '/store', component: TemplateStore, label: 'Store', icon: 'ShoppingBag' },
  { path: '/security', component: SecurityDashboard, label: 'Security', icon: 'Lock' },
];

export const getRouteByPath = (path: string) => {
  const normalized = path === '/' ? '/dashboard' : path.replace(/\/$/, '');
  return APP_ROUTES.find(r => r.path === normalized) || APP_ROUTES[0];
};
