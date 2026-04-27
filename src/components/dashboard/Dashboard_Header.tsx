/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Contextual header for the Operations Hub. 
 * Propagates system titles and integrates `Dashboard_Header_Actions` for session-wide orchestration (Safe Mode/Purge).
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Title propagation delay from the router context, or action group overflow on mobile viewports.
 * - **Telemetry Link**: Search for `[Dashboard_Header]` or `update_setting` in UI tracing.
 */

import React from 'react';
import { Activity, Globe, Loader2, Rocket } from 'lucide-react';
import { Tooltip } from '../ui';
import type { Swarm_Node } from '../../types';
import { i18n } from '../../i18n';

interface DashboardHeaderProps {
    onDiscoverNodes: () => void;
    nodesLoading: boolean;
    nodes: Swarm_Node[];
    onDeploy: (nodeId: string, nodeName: string) => void;
    deployingTarget: string | null;
}

export const Dashboard_Header: React.FC<DashboardHeaderProps> = ({
    onDiscoverNodes,
    nodesLoading,
    nodes,
    onDeploy,
    deployingTarget
}): React.ReactElement => {
    return (
        <div className="flex items-center justify-between border-b border-zinc-900 pb-2 px-1 shrink-0">
            <div>
                <Tooltip content={i18n.t('dashboard.ops_center_tooltip')} position="right">
                    <h2 className="text-xl font-bold flex items-center gap-2 text-zinc-100 cursor-help">
                        <Activity className="text-green-500" /> {i18n.t('command.ops_center')}
                    </h2>
                </Tooltip>
            </div>
            <div className="flex gap-3">
                <Tooltip content={i18n.t('dashboard.scan_network')} position="bottom">
                    <button
                        onClick={onDiscoverNodes}
                        disabled={nodesLoading}
                        className="flex items-center gap-2 px-3 py-2 bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 text-zinc-400 font-bold rounded-lg text-[10px] transition-all uppercase tracking-widest"
                    >
                        {nodesLoading ? <Loader2 size={12} className="animate-spin" /> : <Globe size={12} />}
                        {nodesLoading ? i18n.t('dashboard.scanning') : i18n.t('dashboard.discover_nodes')}
                    </button>
                </Tooltip>

                {nodes.map((node): React.ReactElement => {
                    const nodeNumber = node.id.split('-').pop();
                    const deployLabel = i18n.t('dashboard.deploy_node', { name: nodeNumber?.toUpperCase() || 'NODE' });
                    
                    const NODE_THEME_MAP: Record<string, string> = {
                        'bunker-1': 'bg-green-600 hover:bg-green-500 shadow-green-500/20',
                        'bunker-2': 'bg-cyan-600 hover:bg-cyan-500 shadow-cyan-500/20'
                    };

                    const themeClass = NODE_THEME_MAP[node.id] || 'bg-zinc-800 hover:bg-zinc-700 shadow-zinc-500/20';

                    return (
                        <Tooltip key={node.id} content={i18n.t('dashboard.initiate_deployment', { name: node.name, address: node.address })} position="bottom">
                            <button
                                onClick={() => onDeploy(node.id, node.name)}
                                disabled={deployingTarget !== null}
                                className={`flex items-center gap-2 px-4 py-2 text-white font-bold rounded-lg text-xs transition-all shadow-lg disabled:opacity-50 ${themeClass}`}
                            >
                                {deployingTarget === node.id ? <Loader2 size={14} className="animate-spin" /> : <Rocket size={14} />}
                                {deployingTarget === node.id ? i18n.t('dashboard.deploying') : deployLabel}
                            </button>
                        </Tooltip>
                    );
                })}
            </div>
        </div>
    );
};


// Metadata: [Dashboard_Header]
