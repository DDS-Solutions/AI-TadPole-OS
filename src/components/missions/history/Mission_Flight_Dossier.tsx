/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Missions / History / Mission_Flight_Dossier
 * - **Primary Entrypoints**: `Mission_Flight_Dossier`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Master dossier pane uniting metrics, 5 audit sub-tabs, and pin management.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import React, { useState } from 'react';
import { 
    GitCommit, 
    FileDiff, 
    Brain, 
    Network, 
    Cpu, 
    Pin, 
    PinOff, 
    Bot,
} from 'lucide-react';
import type { MissionDossier, DossierTabKey } from '../../../types/mission_dossier';
import { Tooltip } from '../../ui';
import { Metrics_Rollup_Pane } from './Metrics_Rollup_Pane';
import { Merkle_Audit_Viewer } from './Merkle_Audit_Viewer';
import { CAS_Diff_Inspector } from './CAS_Diff_Inspector';
import { Memory_Palace_Viewer } from './Memory_Palace_Viewer';
import { TrustGraph_Economics_Viewer } from './TrustGraph_Economics_Viewer';
import { Agent99_Debrief_Viewer } from './Agent99_Debrief_Viewer';

interface MissionFlightDossierProps {
    dossier: MissionDossier;
    on_toggle_pin: (id: string) => void;
    is_live_syncing?: boolean;
}

export const Mission_Flight_Dossier: React.FC<MissionFlightDossierProps> = ({
    dossier,
    on_toggle_pin,
    is_live_syncing = false,
}) => {
    const [active_tab, set_active_tab] = useState<DossierTabKey>('merkle');
    const { mission, logs, diffs, memory_palace, trust_graph, economics, debrief } = dossier;

    const tabs: { key: DossierTabKey; label: string; icon: React.ReactNode; tooltip: string; badge?: number }[] = [
        {
            key: 'merkle',
            label: 'Merkle Audit Trail',
            icon: <GitCommit size={14} />,
            tooltip: 'Cryptographic SHA-256 step execution logs & hash verification',
            badge: logs.length,
        },
        {
            key: 'cas',
            label: 'CAS File Diffs',
            icon: <FileDiff size={14} />,
            tooltip: 'Content-Addressable pre-mutation and post-mutation file diffs',
            badge: diffs.length,
        },
        {
            key: 'memory',
            label: 'Memory Palace',
            icon: <Brain size={14} />,
            tooltip: 'Extracted knowledge findings mapped into spatial memory rooms',
            badge: memory_palace.length,
        },
        {
            key: 'trustgraph',
            label: 'TrustGraph & Ledger',
            icon: <Network size={14} />,
            tooltip: 'Entity trust score evaluations and token budget ledger',
            badge: trust_graph.entities.length + economics.transactions.length,
        },
        {
            key: 'debrief',
            label: 'Agent 99 Debrief',
            icon: <Cpu size={14} />,
            tooltip: 'Agent 99 post-flight tactical analysis & system optimizations',
        },
    ];

    return (
        <main className="flex-1 flex flex-col h-full bg-zinc-950 overflow-hidden">
            {/* Mission Dossier Header */}
            <div className="p-5 border-b border-[color:var(--color-border)] bg-[color:var(--color-surface)]/30 backdrop-blur-md flex flex-col gap-4 flex-shrink-0">
                <div className="flex items-start justify-between gap-4">
                    <div className="flex flex-col gap-1.5 min-w-0">
                        <div className="flex items-center gap-2">
                            <Tooltip content="Unique Mission Run UUID" position="bottom">
                                <span className="font-mono text-[11px] font-bold text-cyan-400 bg-cyan-950/40 px-2 py-0.5 rounded border border-cyan-800/40">
                                    {mission.id}
                                </span>
                            </Tooltip>
                            <Tooltip content="Authoritative executing Alpha Agent" position="bottom">
                                <span className="flex items-center gap-1 font-mono text-[11px] text-zinc-400 bg-zinc-900 px-2 py-0.5 rounded border border-zinc-800">
                                    <Bot size={11} className="text-zinc-500" />
                                    {mission.agent_id}
                                </span>
                            </Tooltip>
                            <Tooltip content="Hybrid Engine: Real-time WebSocket notifications with debounced ACID REST synchronization" position="bottom">
                                <span className="flex items-center gap-1.5 px-2 py-0.5 rounded border border-emerald-500/30 bg-emerald-950/30 text-emerald-300 text-[10px] font-mono">
                                    <span className={`w-1.5 h-1.5 rounded-full ${is_live_syncing ? 'bg-cyan-400 animate-ping' : 'bg-emerald-400 animate-pulse'}`} />
                                    <span>{is_live_syncing ? 'Ingesting Stream...' : 'Live Stream Synced'}</span>
                                </span>
                            </Tooltip>
                        </div>
                        <h1 className="text-lg font-bold text-zinc-100 tracking-tight truncate">
                            {mission.title || 'Untitled Sovereign Mission'}
                        </h1>
                    </div>

                    <div className="flex items-center gap-2">
                        <Tooltip content={mission.is_pinned ? "Retention Locked: Protected from Reaper automated pruning" : "Click to pin this mission run to protect from Reaper garbage collection"} position="left">
                            <button
                                onClick={() => on_toggle_pin(mission.id)}
                                aria-label={mission.is_pinned ? "Unpin mission" : "Pin mission"}
                                className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold border transition-all ${
                                    mission.is_pinned
                                        ? 'bg-amber-500/10 text-amber-300 border-amber-500/30 hover:bg-amber-500/20'
                                        : 'bg-zinc-900 border-zinc-800 text-zinc-400 hover:text-zinc-200 hover:border-zinc-700'
                                }`}
                            >
                                {mission.is_pinned ? (
                                    <>
                                        <Pin size={13} className="fill-amber-400" />
                                        <span>Pinned (Protected)</span>
                                    </>
                                ) : (
                                    <>
                                        <PinOff size={13} />
                                        <span>Pin Mission</span>
                                    </>
                                )}
                            </button>
                        </Tooltip>
                    </div>
                </div>

                {/* 5-way KPI Metric Rollup Cards */}
                <Metrics_Rollup_Pane dossier={dossier} />

                {/* 5-tab Navigation Switcher */}
                <div 
                    role="tablist"
                    aria-label="Mission Dossier Views"
                    className="flex items-center gap-1 border-b border-zinc-800/80 -mb-5 pt-1 overflow-x-auto custom-scrollbar"
                >
                    {tabs.map((tab) => {
                        const is_active = active_tab === tab.key;
                        return (
                            <Tooltip key={tab.key} content={tab.tooltip} position="top">
                                <button
                                    role="tab"
                                    id={`tab-${tab.key}`}
                                    aria-controls={`tabpanel-${tab.key}`}
                                    aria-selected={is_active}
                                    onClick={() => set_active_tab(tab.key)}
                                    className={`flex items-center gap-1.5 px-3 py-2 border-b-2 text-xs font-semibold transition-all whitespace-nowrap flex-shrink-0 ${
                                        is_active
                                            ? 'border-cyan-400 text-cyan-300 bg-cyan-950/20'
                                            : 'border-transparent text-zinc-400 hover:text-zinc-200 hover:bg-zinc-900/40'
                                    }`}
                                >
                                    {tab.icon}
                                    <span>{tab.label}</span>
                                    {tab.badge !== undefined && (
                                        <span className={`text-[10px] font-mono px-1.5 py-0.2 rounded-full ${
                                            is_active ? 'bg-cyan-500/20 text-cyan-300' : 'bg-zinc-800 text-zinc-400'
                                        }`}>
                                            {tab.badge}
                                        </span>
                                    )}
                                </button>
                            </Tooltip>
                        );
                    })}
                </div>
            </div>

            {/* Active Sub-Tab View Container */}
            <div 
                role="tabpanel"
                id={`tabpanel-${active_tab}`}
                aria-labelledby={`tab-${active_tab}`}
                className="flex-1 overflow-hidden p-4"
            >
                {active_tab === 'merkle' && <Merkle_Audit_Viewer logs={logs} />}
                {active_tab === 'cas' && <CAS_Diff_Inspector diffs={diffs} />}
                {active_tab === 'memory' && <Memory_Palace_Viewer findings={memory_palace} />}
                {active_tab === 'trustgraph' && (
                    <TrustGraph_Economics_Viewer trust_graph={trust_graph} economics={economics} />
                )}
                {active_tab === 'debrief' && (
                    <Agent99_Debrief_Viewer debrief={debrief} mission_id={mission.id} />
                )}
            </div>
        </main>
    );
};
