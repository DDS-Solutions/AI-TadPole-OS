import React from 'react';
import { motion } from 'framer-motion';
import { Crown } from 'lucide-react';
import type { MissionCluster } from '../../stores/workspaceStore';
import type { Agent } from '../../types';
import { i18n } from '../../i18n';

interface NeuralMapProps {
    cluster: MissionCluster;
    agents: Agent[];
    themeColor: string;
}

export const NeuralMap: React.FC<NeuralMapProps> = ({ cluster, agents, themeColor }) => {
    const alphaAgent = agents.find(a => a.id === cluster.alphaId);
    const collaborators = agents.filter(a => cluster.collaborators.includes(a.id) && a.id !== cluster.alphaId);

    if (!alphaAgent) return (
        <div
            className="relative w-full bg-zinc-900/40 rounded-xl border border-dashed border-zinc-700/50 flex items-center justify-center mb-6"
            style={{ height: '192px', minHeight: '192px' }}
        >
            <span className="text-[10px] text-zinc-500 uppercase font-bold tracking-[0.3em] italic animate-pulse">{i18n.t('missions.label_visualize_alpha')}</span>
        </div>
    );

    return (
        <div
            className="relative w-full bg-zinc-950/60 rounded-xl border border-zinc-800/50 mb-6 overflow-hidden"
            style={{ height: '192px', minHeight: '192px' }}
        >
            <div className="absolute inset-0 neural-grid opacity-10" />

            <svg className="absolute inset-0 w-full h-full" viewBox="0 0 100 100" preserveAspectRatio="none">
                {collaborators.map((agent, i) => {
                    const angle = (i / collaborators.length) * Math.PI * 2;
                    const targetX = 50 + Math.cos(angle) * 35;
                    const targetY = 50 + Math.sin(angle) * 35;

                    return (
                        <motion.path
                            key={agent.id}
                            d={`M 50 50 L ${targetX} ${targetY}`}
                            stroke={themeColor}
                            strokeWidth="0.5"
                            strokeDasharray="1 1"
                            initial={{ pathLength: 0, opacity: 0 }}
                            animate={{
                                pathLength: 1,
                                opacity: [0.1, 0.5, 0.1],
                                strokeDashoffset: [0, -2]
                            }}
                            transition={{
                                pathLength: { duration: 1.5, ease: "easeOut" },
                                opacity: { duration: 2, repeat: Infinity, ease: "easeInOut" },
                                strokeDashoffset: { duration: 5, repeat: Infinity, ease: "linear" }
                            }}
                        />
                    );
                })}
            </svg>

            {/* Alpha Node Visualization */}
            <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 z-10">
                <motion.div
                    animate={{
                        scale: [1, 1.1, 1],
                        boxShadow: [`0 0 10px ${themeColor}20`, `0 0 30px ${themeColor}40`, `0 0 10px ${themeColor}20`]
                    }}
                    transition={{ duration: 4, repeat: Infinity }}
                    className="w-14 h-14 rounded-full border-2 flex items-center justify-center bg-zinc-950"
                    style={{ borderColor: themeColor }}
                >
                    <Crown size={24} style={{ color: themeColor }} className="animate-pulse" />
                </motion.div>
                <div className="absolute -bottom-6 left-1/2 -translate-x-1/2 whitespace-nowrap">
                    <span className="text-[10px] font-bold uppercase tracking-[0.2em]" style={{ color: themeColor }}>{alphaAgent.name}</span>
                </div>
            </div>

            {/* Collaborator Nodes */}
            {collaborators.map((agent, i) => {
                const angle = (i / collaborators.length) * Math.PI * 2;
                const targetX = 50 + Math.cos(angle) * 35;
                const targetY = 50 + Math.sin(angle) * 35;

                return (
                    <div
                        key={agent.id}
                        className="absolute z-10 -translate-x-1/2 -translate-y-1/2"
                        style={{ left: `${targetX}%`, top: `${targetY}%` }}
                    >
                        <motion.div
                            initial={{ scale: 0, opacity: 0 }}
                            animate={{ scale: 1, opacity: 1 }}
                            transition={{ delay: i * 0.1 + 0.5, type: "spring", stiffness: 200 }}
                            className="w-8 h-8 rounded-lg border flex items-center justify-center bg-zinc-900/90 backdrop-blur-sm"
                            style={{ borderColor: `${themeColor}40` }}
                        >
                            <span className="text-[10px] font-bold" style={{ color: themeColor }}>{agent.name[0]}</span>
                        </motion.div>
                        <div className="absolute -bottom-4 left-1/2 -translate-x-1/2 whitespace-nowrap opacity-60">
                            <span className="text-[8px] text-zinc-400 font-mono uppercase">{agent.name}</span>
                        </div>
                    </div>
                );
            })}
        </div>
    );
};
