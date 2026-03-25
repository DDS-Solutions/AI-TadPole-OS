import React from 'react';
import { Search, Loader2, Rocket, UserPlus } from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';
import type { SwarmNode } from '../../types';

interface DashboardHeaderActionsProps {
    onDiscover: () => void;
    nodesLoading: boolean;
    nodes: SwarmNode[];
    onDeploy: (nodeId: string, nodeName: string) => void;
    deployingTarget: string | null;
    onInitializeAgent: () => void;
}

export const DashboardHeaderActions: React.FC<DashboardHeaderActionsProps> = ({
    onDiscover: onDiscoverNodes,
    nodesLoading,
    nodes,
    onDeploy,
    deployingTarget,
    onInitializeAgent
}) => {
    return (
        <div className="flex items-center gap-[100px]">
            <Tooltip content={i18n.t('dashboard.init_agent_tooltip')} position="bottom">
                <button
                    onClick={onInitializeAgent}
                    className="flex items-center gap-2 px-3 py-1.5 bg-emerald-600 hover:bg-emerald-500 text-white font-bold rounded-lg text-[10px] transition-all shadow-[0_0_15px_rgba(16,185,129,0.2)]"
                    aria-label={i18n.t('dashboard.init_agent_tooltip')}
                >
                    <UserPlus size={12} />
                    {i18n.t('dashboard.init_agent')}
                </button>
            </Tooltip>

            <div className="h-4 w-px bg-zinc-800 mx-1" />

            <Tooltip content={i18n.t('dashboard.discover_nodes_tooltip')} position="bottom">
                <button
                    onClick={onDiscoverNodes}
                    disabled={nodesLoading}
                    className="flex items-center gap-2 px-3 py-1.5 bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 text-zinc-300 font-bold rounded-lg text-[10px] transition-all uppercase"
                    aria-label={i18n.t('dashboard.discover_nodes_tooltip')}
                >
                    {nodesLoading ? <Loader2 size={12} className="animate-spin" /> : <Search size={12} />}
                    {nodesLoading ? i18n.t('dashboard.scanning') : i18n.t('dashboard.discover_nodes')}
                </button>
            </Tooltip>

            {nodes.map(node => {
                const nodeNumber = node.id.split('-').pop();
                return (
                    <Tooltip key={node.id} content={i18n.t('dashboard.deploy_to_node_tooltip', { name: node.name })} position="bottom">
                        <button
                            onClick={() => onDeploy(node.id, node.name)}
                            disabled={deployingTarget !== null}
                            className={`flex items-center gap-2 px-4 py-1.5 text-white font-bold rounded-lg text-xs transition-all shadow-lg disabled:opacity-50
                                ${node.id === 'bunker-1' ? 'bg-blue-600 hover:bg-blue-500 shadow-blue-500/20' :
                                    node.id === 'bunker-2' ? 'bg-amber-600 hover:bg-amber-500 shadow-amber-500/20' :
                                        'bg-zinc-800 hover:bg-zinc-700 shadow-zinc-500/20'}
                            `}
                            aria-label={i18n.t('dashboard.deploy_to_node_tooltip', { name: node.name })}
                        >
                            {deployingTarget === node.id ? <Loader2 size={14} className="animate-spin" /> : <Rocket size={14} />}
                            {deployingTarget === node.id ? i18n.t('dashboard.deploying') : i18n.t('dashboard.deploy_node', { name: nodeNumber?.toUpperCase() || 'NODE' })}
                        </button>
                    </Tooltip>
                );
            })}
        </div>
    );
};
