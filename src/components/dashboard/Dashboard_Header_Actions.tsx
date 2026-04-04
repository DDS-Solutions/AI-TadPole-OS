import React from 'react';
import { Search, Loader2, Rocket, UserPlus } from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';
import type { Swarm_Node } from '../../types';

interface Dashboard_Header_Actions_Props {
    on_discover: () => void;
    nodes_loading: boolean;
    nodes: Swarm_Node[];
    on_deploy: (node_id: string, node_name: string) => void;
    deploying_target: string | null;
    on_initialize_agent: () => void;
}

export const Dashboard_Header_Actions: React.FC<Dashboard_Header_Actions_Props> = ({
    on_discover,
    nodes_loading,
    nodes,
    on_deploy,
    deploying_target,
    on_initialize_agent
}) => {
    return (
        <div className="flex items-center gap-[100px]">
            <Tooltip content={i18n.t('dashboard.init_agent_tooltip')} position="bottom">
                <button
                    onClick={on_initialize_agent}
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
                    onClick={on_discover}
                    disabled={nodes_loading}
                    className="flex items-center gap-2 px-3 py-1.5 bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 text-zinc-300 font-bold rounded-lg text-[10px] transition-all uppercase"
                    aria-label={i18n.t('dashboard.discover_nodes_tooltip')}
                >
                    {nodes_loading ? <Loader2 size={12} className="animate-spin" /> : <Search size={12} />}
                    {nodes_loading ? i18n.t('dashboard.scanning') : i18n.t('dashboard.discover_nodes')}
                </button>
            </Tooltip>

            {nodes.map(node => {
                const node_number = node.id.split('-').pop();
                return (
                    <Tooltip key={node.id} content={i18n.t('dashboard.deploy_to_node_tooltip', { name: node.name })} position="bottom">
                        <button
                            onClick={() => on_deploy(node.id, node.name)}
                            disabled={deploying_target !== null}
                            className={`flex items-center gap-2 px-4 py-1.5 text-white font-bold rounded-lg text-xs transition-all shadow-lg disabled:opacity-50
                                ${node.id === 'bunker-1' ? 'bg-blue-600 hover:bg-blue-500 shadow-blue-500/20' :
                                    node.id === 'bunker-2' ? 'bg-amber-600 hover:bg-amber-500 shadow-amber-500/20' :
                                        'bg-zinc-800 hover:bg-zinc-700 shadow-zinc-500/20'}
                            `}
                            aria-label={i18n.t('dashboard.deploy_to_node_tooltip', { name: node.name })}
                        >
                            {deploying_target === node.id ? <Loader2 size={14} className="animate-spin" /> : <Rocket size={14} />}
                            {deploying_target === node.id ? i18n.t('dashboard.deploying') : i18n.t('dashboard.deploy_node', { name: node_number?.toUpperCase() || 'NODE' })}
                        </button>
                    </Tooltip>
                );
            })}

        </div>
    );
};
