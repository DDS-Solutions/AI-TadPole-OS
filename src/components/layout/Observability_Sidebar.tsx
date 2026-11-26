/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Layout / Observability_Sidebar
 * - **Primary Entrypoints**: `Observability_Sidebar`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import React from 'react';
import { clsx } from 'clsx';
import { useShallow } from 'zustand/react/shallow';
import { i18n } from '../../i18n';
import { use_tab_store } from '../../stores/tab_store';
import { Lineage_Stream } from '../Lineage_Stream';
import { Neural_Waterfall } from '../Neural_Waterfall';
import { System_Log } from '../dashboard/System_Log';

interface Observability_Sidebar_Props {
  is_detached_context?: boolean;
  className?: string;
}

interface Detached_Sector_Placeholder_Props {
  title: string;
  pulse_color: string;
  on_recall: () => void;
}

const Detached_Sector_Placeholder: React.FC<Detached_Sector_Placeholder_Props> = ({
  title,
  pulse_color,
  on_recall
}) => (
  <div className="p-3 border border-dashed border-zinc-800 rounded-xl bg-zinc-950/40 flex items-center justify-between gap-3 text-zinc-400 shrink-0">
    <div className="flex items-center gap-2.5 min-w-0">
      <div className={clsx("w-2 h-2 rounded-full animate-pulse", pulse_color)} />
      <div className="flex flex-col min-w-0">
        <span className="text-[10px] font-mono uppercase tracking-wider font-semibold text-zinc-300 truncate">
          {title}
        </span>
        <span className="text-[9px] font-mono text-zinc-600">
          Detached in external window
        </span>
      </div>
    </div>
    <button
      onClick={on_recall}
      className="px-2.5 py-1 text-[9px] font-mono font-bold uppercase tracking-wider bg-zinc-900 hover:bg-zinc-800 text-zinc-200 rounded border border-zinc-700 hover:border-zinc-600 transition-all shadow-sm active:scale-95 shrink-0"
      title={`Dock ${title} back to sidebar`}
    >
      Recall
    </button>
  </div>
);

const All_Detached_Empty_State: React.FC = () => {
  const handle_recall = () => {
    use_tab_store.getState().reattach_observability_sectors();
  };

  return (
    <div className="flex-1 flex flex-col items-center justify-center text-zinc-500 p-8 text-center space-y-4 my-auto">
      <div className="w-12 h-12 rounded-full border border-[color:var(--color-surface)] flex items-center justify-center">
        <div className="w-2 h-2 rounded-full bg-cyan-500 animate-pulse" />
      </div>
      <div className="space-y-1">
        <p className="text-[10px] uppercase tracking-[0.2em] font-bold text-zinc-400">
          {i18n.t('layout.monitor_detached') || 'Monitors Detached'}
        </p>
        <p className="text-[9px] text-zinc-600 font-mono">
          Event Log, Waterfall & OTel detached to external windows
        </p>
      </div>
      <button
        onClick={handle_recall}
        className="px-4 py-1.5 bg-zinc-800 hover:bg-zinc-700 text-zinc-200 text-[10px] font-mono uppercase tracking-widest rounded border border-zinc-700 transition-colors shadow-sm"
      >
        Recall All Monitors
      </button>
    </div>
  );
};

const Sector_Stack: React.FC = () => {
  const { 
    is_system_log_detached, 
    is_trace_stream_detached, 
    is_lineage_stream_detached 
  } = use_tab_store(
    useShallow((s) => ({
      is_system_log_detached: s.is_system_log_detached,
      is_trace_stream_detached: s.is_trace_stream_detached,
      is_lineage_stream_detached: s.is_lineage_stream_detached,
    }))
  );
  const is_any_detached = is_system_log_detached || is_trace_stream_detached || is_lineage_stream_detached;

  return (
    <>
      {is_lineage_stream_detached ? (
        <Detached_Sector_Placeholder
          title={i18n.t('trace.stream_title') || 'OTel Trace Stream'}
          pulse_color="bg-emerald-500"
          on_recall={() => use_tab_store.getState().toggle_lineage_stream_detachment()}
        />
      ) : (
        <Lineage_Stream />
      )}

      {is_trace_stream_detached ? (
        <Detached_Sector_Placeholder
          title={i18n.t('trace_stream.title') || 'Neural Waterfall'}
          pulse_color="bg-cyan-500"
          on_recall={() => use_tab_store.getState().toggle_trace_stream_detachment()}
        />
      ) : (
        <Neural_Waterfall />
      )}

      {is_system_log_detached ? (
        <Detached_Sector_Placeholder
          title={i18n.t('system_log.title') || 'System Log'}
          pulse_color="bg-amber-500"
          on_recall={() => use_tab_store.getState().toggle_system_log_detachment()}
        />
      ) : (
        <System_Log />
      )}

      {is_any_detached && (
        <div className="pt-2 flex justify-center">
          <button
            onClick={() => use_tab_store.getState().reattach_observability_sectors()}
            className="px-3 py-1 bg-zinc-900 hover:bg-zinc-800 text-zinc-400 hover:text-zinc-200 text-[9px] font-mono uppercase tracking-widest rounded border border-zinc-800 transition-colors shadow-sm"
          >
            Recall All Detached Monitors
          </button>
        </div>
      )}
    </>
  );
};

/**
 * Observability_Sidebar
 * 
 * A unified sidebar containing the core observability sectors of Tadpole OS.
 * It intelligently displays placeholders with recall actions for sectors detached into external windows.
 * Refactored for strict snake_case compliance and consistent prop propagation.
 */
export const Observability_Sidebar: React.FC<Observability_Sidebar_Props> = ({ 
  is_detached_context = false,
  className 
}) => {
  const { 
    is_system_log_detached, 
    is_trace_stream_detached, 
    is_lineage_stream_detached 
  } = use_tab_store(
    useShallow((s) => ({
      is_system_log_detached: s.is_system_log_detached,
      is_trace_stream_detached: s.is_trace_stream_detached,
      is_lineage_stream_detached: s.is_lineage_stream_detached,
    }))
  );

  const is_all_detached = is_system_log_detached && is_trace_stream_detached && is_lineage_stream_detached;

  return (
    <div className={clsx(
      "flex flex-col gap-4 relative h-full overflow-y-auto custom-scrollbar p-3 border-l border-[color:var(--color-surface)] bg-zinc-950/20",
      is_detached_context ? "w-[400px]" : "w-[400px] shrink-0",
      className
    )}>
      {is_all_detached ? <All_Detached_Empty_State /> : <Sector_Stack />}
    </div>
  );
};
