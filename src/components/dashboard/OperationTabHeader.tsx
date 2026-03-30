import React from 'react';
import { motion } from 'framer-motion';
import { 
    Globe, 
    Layers, 
    Power, 
    ExternalLink,
    Activity
} from 'lucide-react';
import { i18n } from '../../i18n';
import type { MissionCluster } from '../../types';
import { Tooltip } from '../ui';

interface OperationTabHeaderProps {
    activeTabId: string;
    clusters: MissionCluster[];
    onTabChange: (id: string) => void;
    onToggleCluster: (clusterId: string) => void;
    onDetachTab: (id: string) => void;
}

export const OperationTabHeader: React.FC<OperationTabHeaderProps> = ({
    activeTabId,
    clusters,
    onTabChange,
    onToggleCluster,
    onDetachTab
}) => {
    const renderTab = (id: string, label: string, icon: React.ReactNode, isCluster = false, isActiveCluster = false) => {
        const isSelected = activeTabId === id;

        return (
            <div 
                key={id}
                className={`flex-shrink-0 flex items-center gap-2 px-4 h-12 relative cursor-pointer transition-all duration-300 group
                    ${isSelected ? 'text-blue-400' : 'text-zinc-500 hover:text-zinc-300'}
                `}
                onClick={() => onTabChange(id)}
            >
                {/* Active Indicator Bar */}
                {isSelected && (
                    <motion.div 
                        layoutId="activeTab"
                        className="absolute bottom-0 left-0 right-0 h-0.5 bg-blue-500 shadow-[0_0_8px_rgba(59,130,246,0.6)] z-10"
                    />
                )}

                <div className="flex items-center gap-3">
                    <div className="flex items-center gap-2">
                        {icon}
                        <span className="text-[10px] uppercase font-bold tracking-widest whitespace-nowrap">
                            {label}
                        </span>
                    </div>
                    
                    <div className="flex items-center gap-1.5 opacity-0 group-hover:opacity-100 transition-opacity">
                         <Tooltip content="Pop Out Sector">
                            <button
                                onClick={(e) => {
                                    e.stopPropagation();
                                    onDetachTab(id);
                                }}
                                className="p-1 text-zinc-600 hover:text-blue-400 hover:bg-blue-500/10 rounded transition-all"
                            >
                                <ExternalLink size={10} />
                            </button>
                        </Tooltip>

                        {isCluster && (
                            <div className="flex items-center gap-2 ml-1 pl-1 border-l border-white/5">
                                <Tooltip content={isActiveCluster ? 'Deactivate Mission' : 'Activate Mission'}>
                                    <button
                                        onClick={(e) => {
                                            e.stopPropagation();
                                            onToggleCluster(id);
                                        }}
                                        className={`p-1 rounded-md transition-all duration-300 
                                            ${isActiveCluster 
                                                ? 'text-emerald-500 bg-emerald-500/10 hover:bg-emerald-500/20 shadow-[0_0_10px_rgba(16,185,129,0.2)]' 
                                                : 'text-zinc-600 hover:text-zinc-400 hover:bg-white/5'
                                            }
                                        `}
                                    >
                                        <Power size={10} className={isActiveCluster ? 'animate-pulse' : ''} />
                                    </button>
                                </Tooltip>
                                
                                {/* Small Status Dot */}
                                <div className={`w-1 h-1 rounded-full ${isActiveCluster ? 'bg-emerald-500 animate-pulse shadow-[0_0_5px_rgba(16,185,129,0.5)]' : 'bg-zinc-700'}`} />
                            </div>
                        )}
                    </div>
                </div>
            </div>
        );
    };

    return (
        <div className="sticky top-0 bg-zinc-950/80 backdrop-blur-md z-30 border-b border-zinc-800/50 flex items-center justify-between px-2">
            <div className="flex-1 flex items-center overflow-x-auto no-scrollbar min-w-0">
                {/* Global View Tab */}
                {renderTab('global', i18n.t('dashboard.global_view') || 'Global View', <Globe size={12} className="group-hover:rotate-12 transition-transform" />)}
 
                <div className="h-4 w-px bg-white/5 mx-2" />

                {/* Dynamic Cluster Tabs */}
                {clusters.map(cluster => renderTab(
                    cluster.id, 
                    cluster.name, 
                    <Layers size={12} />, 
                    true, 
                    cluster.isActive
                ))}
            </div>

            {/* Actions Sidebar in Header */}
            <div className="flex items-center gap-4 pr-4">
                 <div className="flex items-center gap-2 px-3 py-1 bg-blue-500/5 border border-blue-500/10 rounded-full">
                    <Activity size={10} className="text-blue-500 animate-pulse" />
                    <span className="text-[9px] font-mono text-blue-400/80 uppercase tracking-tighter">Live Status</span>
                </div>
            </div>
        </div>
    );
};
