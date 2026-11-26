/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Pages / detached_shell
 * - **Primary Entrypoints**: `Detached_Shell`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: `[DetachedShell]`
 */

import { Suspense, lazy, useState, useMemo } from 'react';
import { useSearchParams } from 'react-router-dom';
import { Minimize2 } from 'lucide-react';
import clsx from 'clsx';
import { useDashboardData } from '../hooks/use_dashboard_data';
import { useWindowAutoScale } from '../hooks/useWindowAutoScale';
import { useLayoutServices } from '../hooks/layout/useLayoutServices';
import { get_route_by_path } from '../constants/routes';
import Error_Boundary from '../components/Error_Boundary';
import { i18n } from '../i18n';
import type { Agent } from '../types';

// lazy-loaded components to match the main app's architecture
const System_Log = lazy(() => import('../components/dashboard/System_Log').then(module => ({ default: module.System_Log })));
const Neural_Waterfall = lazy(() => import('../components/Neural_Waterfall').then(module => ({ default: module.Neural_Waterfall })));
const Lineage_Stream = lazy(() => import('../components/Lineage_Stream').then(module => ({ default: module.Lineage_Stream })));
const Swarm_Visualizer = lazy(() => import('../components/Swarm_Visualizer').then(module => ({ default: module.Swarm_Visualizer })));
const Agent_Status_Grid = lazy(() => import('../components/dashboard/Agent_Status_Grid').then(module => ({ default: module.Agent_Status_Grid })));
const AgentConfigPanel = lazy(() => import('../components/AgentConfigPanel').then(module => ({ default: module.default })));
const SovereignChat = lazy(() => import('../components/SovereignChat').then(module => ({ default: module.SovereignChat })));

import { resolve_provider, resolve_agent_model_config, parse_active_model_slot } from '../utils/model_utils';
import { get_settings } from '../stores/settings_store';
import { use_role_store } from '../stores/role_store';

function create_detached_agent_record(params: Partial<Agent>): Agent {
  const settings = get_settings();
  const default_model = settings.default_model || 'gemini-1.5-flash';
  const new_id = `agent-${typeof crypto !== 'undefined' && crypto.randomUUID ? crypto.randomUUID().substring(0, 8) : Date.now().toString(36)}`;
  return {
    id: new_id,
    name: params.name || i18n.t('ops.placeholder_name'),
    role: params.role || 'assistant',
    department: params.department || 'Operations',
    status: 'idle',
    tokens_used: 0,
    model: params.model || default_model,
    skills: params.skills || [],
    workflows: params.workflows || [],
    cost_usd: 0,
    budget_usd: params.budget_usd || 0,
    theme_color: params.theme_color || '#10b981',
    last_pulse: new Date().toISOString(),
    category: 'user',
    ...params,
  };
}

async function handle_agent_skill_trigger(
  agent_id: string,
  skill: string,
  agents: Agent[],
  clusters: Array<{ id: string; collaborators?: string[]; budget_usd?: number }>,
  slot?: 1 | 2 | 3,
  update_agent?: (id: string, updates: Partial<Agent>) => Promise<boolean | void>
) {
  const agent = agents.find(a => a.id === agent_id);
  if (!agent) return;
  const target_slot = slot !== undefined ? slot : parse_active_model_slot(agent.active_model_slot);
  void update_agent?.(agent_id, {
    status: 'active',
    current_task: i18n.t('ops.event_executing', { skill }),
    active_model_slot: target_slot,
  });
  const cluster = clusters.find(c => (c.collaborators || []).includes(agent_id));
  try {
    const { tadpole_os_service } = await import('../services/tadpoleos_service');
    const { model_id, provider } = resolve_agent_model_config({ ...agent, active_model_slot: target_slot });
    const success = await tadpole_os_service.send_command(
      agent_id,
      skill,
      model_id,
      provider,
      cluster?.id,
      agent.department,
      cluster?.budget_usd
    );
    if (!success) {
      void update_agent?.(agent_id, { status: 'idle', current_task: undefined });
    }
  } catch (e) {
    console.error('[DetachedShell] Skill trigger failed:', e);
    void update_agent?.(agent_id, { status: 'idle', current_task: undefined });
  }
}

function update_agent_model_slot(
  agent: Agent | undefined,
  new_model: string,
  model_key: 'model' | 'model_2' | 'model_3',
  config_key: 'model_config' | 'model_config2' | 'model_config3',
  update_agent: (id: string, updates: Partial<Agent>) => Promise<boolean | void>
) {
  if (!agent) return;
  const provider = resolve_provider(new_model);
  const current_config = agent[config_key];
  const updated_config = current_config
    ? { ...current_config, modelId: new_model, provider }
    : { modelId: new_model, provider };
  void update_agent(agent.id, { [model_key]: new_model, [config_key]: updated_config });
}

type ModelKey = 'model' | 'model_2' | 'model_3';
type ConfigKey = 'model_config' | 'model_config2' | 'model_config3';

function Detached_Config_Modal({
  agent,
  is_new,
  on_close,
  on_update,
  on_delete
}: {
  agent: Agent | undefined;
  is_new: boolean;
  on_close: () => void;
  on_update: (id: string, updates: Partial<Agent>) => Promise<boolean | void>;
  on_delete: (id: string) => Promise<void>;
}) {
  return (
    <AgentConfigPanel
      agent={agent}
      onClose={on_close}
      onUpdate={on_update}
      onDelete={on_delete}
      isNew={is_new}
    />
  );
}

function apply_detached_role_change(
  agent_id: string,
  new_role: string,
  update_agent: (id: string, updates: Partial<Agent>) => Promise<boolean | void>
) {
  const roles = use_role_store.getState().roles;
  const new_actions = roles[new_role] || { skills: [], workflows: [] };
  void update_agent(agent_id, {
    role: new_role,
    skills: new_actions.skills,
    workflows: new_actions.workflows,
  });
}

function Detached_Agent_Status({ tab_id }: { tab_id?: string }) {
  const {
    agents_list,
    assigned_agent_ids,
    available_roles,
    clusters,
    toggle_cluster_active,
    update_agent,
    add_agent,
    delete_agent
  } = useDashboardData();

  const [config_agent_id, set_config_agent_id] = useState<string | null>(null);
  const config_agent = useMemo(
    () => (!config_agent_id || config_agent_id === 'new' ? undefined : agents_list.find(a => a.id === config_agent_id)),
    [config_agent_id, agents_list]
  );
  const on_slot = (id: string, model: string, k1: ModelKey, k2: ConfigKey) =>
    update_agent_model_slot(agents_list.find(a => a.id === id), model, k1, k2, update_agent);

  return (
    <div className="h-full bg-[color:var(--color-background)] p-6 flex flex-col overflow-hidden">
      <Agent_Status_Grid
        agents={agents_list}
        assigned_agent_ids={assigned_agent_ids}
        available_roles={available_roles}
        clusters={clusters}
        initial_tab_id={tab_id}
        on_skill_trigger={(id, skill, slot) => handle_agent_skill_trigger(id, skill, agents_list, clusters, slot, update_agent)}
        on_model_change={(id, model) => on_slot(id, model, 'model', 'model_config')}
        on_model_2_change={(id, model) => on_slot(id, model, 'model_2', 'model_config2')}
        on_model_3_change={(id, model) => on_slot(id, model, 'model_3', 'model_config3')}
        on_role_change={(id, role) => apply_detached_role_change(id, role, update_agent)}
        on_configure_click={(id) => set_config_agent_id(id)}
        handle_agent_update={update_agent}
        on_toggle_cluster={toggle_cluster_active}
      />

      {config_agent_id && (
        <Detached_Config_Modal
          agent={config_agent}
          is_new={config_agent_id === 'new'}
          on_close={() => set_config_agent_id(null)}
          on_update={(id, updates) => (id === 'new' ? add_agent(create_detached_agent_record(updates)) : update_agent(id, updates))}
          on_delete={(id) => delete_agent(id).then(() => set_config_agent_id(null))}
        />
      )}
    </div>
  );
}


function Detached_Agent_Config({ id }: { id: string | null }) {
  const { agents_list, update_agent, add_agent, delete_agent } = useDashboardData();
  const agent = agents_list.find(a => a.id === id);

  const handle_close = async () => {
    const is_tauri = !!(window as unknown as { __TAURI_INTERNALS__: unknown }).__TAURI_INTERNALS__;
    if (is_tauri) {
      try {
        const { getCurrentWindow } = await import('@tauri-apps/api/window');
        await getCurrentWindow().close();
      } catch (err) {
        console.debug('[DetachedShell] Tauri window close failed, falling back to window.close():', err);
        window.close();
      }
    } else {
      window.close();
    }
  };

  return (
    <div className="h-screen bg-[color:var(--color-background)] p-6 flex items-center justify-center">
      <AgentConfigPanel
        agent={agent}
        onClose={handle_close}
        onUpdate={(agent_id, updates) =>
          agent_id === 'new'
            ? add_agent(create_detached_agent_record(updates))
            : update_agent(agent_id, updates)
        }
        onDelete={async (agent_id) => {
          await delete_agent(agent_id);
          await handle_close();
        }}
        isNew={id === 'new'}
        isDetachedMode={true}
      />
    </div>
  );
}

function Detached_Header({ title, on_recall }: { title: string; on_recall: () => void }) {
  return (
    <header className="h-9 px-3 bg-zinc-950/95 border-b border-zinc-800/80 backdrop-blur-md flex items-center justify-between shrink-0 select-none z-50">
      <div className="flex items-center gap-2.5 min-w-0">
        <span className="relative flex h-2 w-2 shrink-0">
          <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75" />
          <span className="relative inline-flex rounded-full h-2 w-2 bg-emerald-500" />
        </span>
        <span className="text-xs font-semibold text-zinc-200 truncate font-mono tracking-tight">
          {title}
        </span>
        <span className="hidden sm:inline-block px-1.5 py-0.5 text-[8px] font-mono font-bold uppercase tracking-wider text-emerald-400 bg-emerald-500/10 border border-emerald-500/20 rounded">
          DETACHED LINK ACTIVE
        </span>
      </div>

      <button
        onClick={on_recall}
        className="flex items-center gap-1.5 px-2.5 py-1 text-[10px] font-mono font-medium text-zinc-300 hover:text-zinc-100 bg-zinc-900/80 hover:bg-zinc-800 border border-zinc-700/80 hover:border-zinc-600 rounded transition-all active:scale-95 group"
        title="Recall / Dock Window"
      >
        <Minimize2 size={12} className="group-hover:-rotate-45 transition-transform" />
        <span>RECALL</span>
      </button>
    </header>
  );
}

function Detached_Sector_Frame({ 
  title, 
  on_recall, 
  padding = "p-0", 
  children 
}: { 
  title: string; 
  on_recall: () => void; 
  padding?: string; 
  children: React.ReactNode; 
}) {
  return (
    <div className="h-screen bg-[color:var(--color-background)] p-0 flex flex-col overflow-hidden">
      <Detached_Header title={title} on_recall={on_recall} />
      <div className={clsx("flex-1 overflow-hidden", padding)}>
        {children}
      </div>
    </div>
  );
}

function Detached_Tab_Frame({ path, on_recall }: { path: string | null; on_recall: () => void }) {
  const route = get_route_by_path(path || '/dashboard');
  const Component = route.component;
  return (
    <div className="h-screen w-full bg-[color:var(--color-background)] p-0 flex flex-col overflow-hidden">
      <Detached_Header title={route.label} on_recall={on_recall} />
      <div className="flex-1 relative overflow-auto custom-scrollbar w-full h-full p-4 sm:p-6">
        <Error_Boundary name={`Sector: ${route.label}`}>
          <Suspense fallback={<div className="p-8 text-zinc-500 font-mono text-xs animate-pulse">{i18n.t('common.initializing_sector')}</div>}>
            <Component />
          </Suspense>
        </Error_Boundary>
      </div>
    </div>
  );
}

/**
 * Detached_Shell
 * A unified dispatcher for all native detached windows in the Tadpole OS.
 * Reads the ?type= parameter and renders the appropriate component standalone.
 */
export default function Detached_Shell() {
  const [search_params] = useSearchParams();
  const type = search_params.get('type');
  const id = search_params.get('id');
  const tab_id = search_params.get('tabId');
  const path = search_params.get('path');

  // Dynamic auto-scaling based on window size for native detached webviews
  useWindowAutoScale(1200, 800, 0.55, 1.0);
  useLayoutServices();

  const handle_recall = async () => {
    const bc = typeof window !== 'undefined' && typeof BroadcastChannel !== 'undefined'
      ? new BroadcastChannel('tab_store_sync')
      : null;
    if (bc) {
      bc.postMessage({
        type: 'RECALL_SECTOR',
        payload: { type, id, path, tab_id }
      });
      bc.close();
    }

    const is_tauri = !!(window as unknown as { __TAURI_INTERNALS__: unknown }).__TAURI_INTERNALS__;
    if (is_tauri) {
      try {
        const { getCurrentWindow } = await import('@tauri-apps/api/window');
        await getCurrentWindow().close();
      } catch {
        window.close();
      }
    } else {
      window.close();
    }
  };

  // Mapping of types to components
  const render_content = () => {
    if (type === 'chat') return <SovereignChat isDetachedView />;
    if (type === 'system-log') {
      return (
        <Detached_Sector_Frame title={i18n.t('system_log.title') || 'System Log'} on_recall={handle_recall} padding="p-6">
          <System_Log is_detached_view />
        </Detached_Sector_Frame>
      );
    }
    if (type === 'trace-stream') {
      return (
        <Detached_Sector_Frame title={i18n.t('trace_stream.title') || 'Neural Waterfall'} on_recall={handle_recall}>
          <Neural_Waterfall is_detached_view />
        </Detached_Sector_Frame>
      );
    }
    if (type === 'lineage-stream') {
      return (
        <Detached_Sector_Frame title={i18n.t('trace.stream_title') || 'OTel Trace Stream'} on_recall={handle_recall}>
          <Lineage_Stream is_detached_view />
        </Detached_Sector_Frame>
      );
    }
    if (type === 'swarm-pulse') {
      return (
        <Detached_Sector_Frame title="Swarm Pulse Visualizer" on_recall={handle_recall} padding="p-6">
          <Swarm_Visualizer is_detached={true} />
        </Detached_Sector_Frame>
      );
    }
    if (type === 'agent-status') {
      return (
        <Detached_Sector_Frame title={tab_id ? `Agent Operations (${tab_id})` : 'Agent Status Grid'} on_recall={handle_recall}>
          <Detached_Agent_Status tab_id={tab_id || undefined} />
        </Detached_Sector_Frame>
      );
    }
    if (type === 'agent-config') return <Detached_Agent_Config id={id} />;
    if (type === 'tab') return <Detached_Tab_Frame path={path} on_recall={handle_recall} />;

    return (
      <div className="h-screen bg-[color:var(--color-background)] flex items-center justify-center text-zinc-500 font-mono">
        {i18n.t('common.initializing_neural_link')}
      </div>
    );
  };

  return (
    <div className="min-h-screen bg-[color:var(--color-background)] text-zinc-100 font-sans antialiased">
      <h1 className="sr-only">{i18n.t('common.detached_shell_title')}</h1>
      <Suspense fallback={<div className="h-screen bg-[color:var(--color-background)] flex items-center justify-center animate-pulse font-mono uppercase tracking-widest text-zinc-700 text-xs">{i18n.t('common.synchronizing_channels')}</div>}>
        {render_content()}
      </Suspense>
    </div>
  );
}
