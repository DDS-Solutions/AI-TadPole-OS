import React, { useState, useMemo } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { } from 'lucide-react'; // Removing unused but keeping line for potential future imports
import { SwarmOversightNode } from './SwarmOversightNode';
import { getAgentStatusStyles } from '../utils/agentUIUtils';
import { useDropdownStore } from '../stores/dropdownStore';
import { useWorkspaceStore } from '../stores/workspaceStore';
import type { Agent } from '../types';

// Sub-components
import { NodeHeader } from './hierarchy/NodeHeader';
import { NodeStats } from './hierarchy/NodeStats';
import { NodeMission } from './hierarchy/NodeMission';
import { NodeTaskBox } from './hierarchy/NodeTaskBox';
import { NodeModelSlots } from './hierarchy/NodeModelSlots';
import { NodeHealth } from './hierarchy/NodeHealth';

/**
 * Props for the HierarchyNode component.
 */
interface HierarchyNodeProps {
    agent?: Agent;
    isRoot?: boolean;
    onSkillTrigger?: (agentId: string, skill: string, slot?: 1 | 2 | 3) => void;
    onConfigureClick?: (agentId: string) => void;
    isAlpha?: boolean;
    isActive?: boolean;
    missionObjective?: string;
    onModelChange?: (agentId: string, newModel: string) => void;
    onModel2Change?: (agentId: string, newModel: string) => void;
    onModel3Change?: (agentId: string, newModel: string) => void;
    onRoleChange?: (agentId: string, newRole: string) => void;
    onUpdate?: (agentId: string, updates: Partial<Agent>) => void;
    availableRoles?: string[];
    themeColor?: string;
}

/**
 * A specialized agent card component designed for the Neural Command Hierarchy.
 * Features status indicators, mission data, and per-model configuration badges.
 * Refactored into a modular orchestration component.
 */
const HierarchyNodeBase: React.FC<HierarchyNodeProps> = ({
    agent,
    isRoot = false,
    onSkillTrigger,
    onConfigureClick,
    isAlpha = false,
    isActive = false,
    missionObjective,
    onModelChange,
    onModel2Change,
    onModel3Change,
    onRoleChange,
    onUpdate,
    availableRoles = [],
}): React.ReactElement | null => {
    const isAnyDropdownOpen = useDropdownStore(s => s.openId === agent?.id);
    const [isOversightOpen, setIsOversightOpen] = useState(false);
    const [isHealthOpen, setIsHealthOpen] = useState(false);

    const { clusters, activeProposals } = useWorkspaceStore();
    
    const cluster = useMemo(() => 
        clusters.find(c => c.collaborators.map(String).includes(String(agent?.id))),
    [clusters, agent?.id]);

    const hasOversight = cluster ? !!activeProposals[cluster.id] : false;

    if (!agent) return null;

    const statusStyles = getAgentStatusStyles(agent.status);
    const agentColor = agent.themeColor || statusStyles.hex;

    return (
        <div className={`relative group transition-all duration-300 w-full ${isAnyDropdownOpen ? 'z-[100]' : 'z-0'}`}>
            <div
                className={`
                    relative z-10 p-3 rounded-xl border backdrop-blur-xl transition-all duration-300
                    bg-[#1b1b1e]/60
                    ${agent.status !== 'offline' && agent.status !== 'idle' ? 'border-zinc-800' : 'border-zinc-800/30'}
                    hover:border-zinc-600 hover:scale-[1.02] active:scale-[0.98]
                    overflow-visible flex flex-col gap-3 shadow-2xl
                `}
                style={{
                    borderColor: `${agentColor}30`,
                    boxShadow: `0 0 15px ${agentColor}10`,
                    color: agentColor
                }}
            >
                {/* Header: Identity & Status */}
                <NodeHeader
                    agent={agent}
                    isAlpha={isAlpha}
                    isActive={isActive}
                    availableRoles={availableRoles}
                    onRoleChange={onRoleChange}
                    onConfigureClick={onConfigureClick}
                    hasOversight={hasOversight}
                    isOversightOpen={isOversightOpen}
                    onOversightToggle={() => setIsOversightOpen(!isOversightOpen)}
                    isHealthOpen={isHealthOpen}
                    onHealthToggle={() => setIsHealthOpen(!isHealthOpen)}
                />

                {/* Metrics & Skills Row */}
                <NodeStats agent={agent} onSkillTrigger={onSkillTrigger} />

                {/* Mission Badge area */}
                <NodeMission agent={agent} isActive={isActive} missionObjective={missionObjective} />

                {/* Current Task Box */}
                <NodeTaskBox agent={agent} />

                {/* Model Badges (Interactive Picker) */}
                <NodeModelSlots
                    agent={agent}
                    onModelChange={onModelChange}
                    onModel2Change={onModel2Change}
                    onModel3Change={onModel3Change}
                    onUpdate={onUpdate}
                />

                {/* Health/Security Overlay */}
                <AnimatePresence>
                    {isHealthOpen && (
                        <NodeHealth 
                            agent={agent} 
                            onClose={() => setIsHealthOpen(false)} 
                        />
                    )}
                </AnimatePresence>
            </div>

            {/* Connection Point Indicators */}
            {!isRoot && (
                <div className="absolute -top-1 left-1/2 -translate-x-1/2 w-2 h-2 rounded-full border border-current bg-zinc-950 opacity-0 group-hover:opacity-100 transition-opacity" />
            )}
            <div className="absolute -bottom-1 left-1/2 -translate-x-1/2 w-2 h-2 rounded-full border border-current bg-zinc-950 opacity-0 group-hover:opacity-100 transition-opacity" />

            {/* Neural Glow (Active Mission) */}
            <AnimatePresence>
                {(isActive || agent.status === 'active') && (
                    <motion.div
                        initial={{ opacity: 0, scale: 0.95 }}
                        animate={{
                            opacity: [0.1, 0.4, 0.1],
                            scale: [1, 1.05, 1],
                        }}
                        exit={{ opacity: 0, scale: 0.95 }}
                        transition={{
                            duration: 4,
                            repeat: Infinity,
                            ease: "easeInOut"
                        }}
                        className="absolute -inset-2 rounded-2xl blur-2xl -z-10"
                        style={{ backgroundColor: `${agentColor}30` }}
                    />
                )}
            </AnimatePresence>

            {/* Swarm Oversight Integration (Alpha Only) */}
            <AnimatePresence>
                {isOversightOpen && isAlpha && hasOversight && cluster && (
                    <motion.div 
                        initial={{ opacity: 0, x: -20, scale: 0.95 }}
                        animate={{ opacity: 1, x: 0, scale: 1 }}
                        exit={{ opacity: 0, x: -20, scale: 0.95 }}
                        transition={{ type: "spring", damping: 20, stiffness: 300 }}
                        className="absolute top-0 -right-[420px] pointer-events-none z-[110]"
                    >
                        <div className="pointer-events-auto">
                            <SwarmOversightNode 
                                clusterId={cluster.id} 
                                onClose={() => setIsOversightOpen(false)}
                            />
                        </div>
                    </motion.div>
                )}
            </AnimatePresence>
        </div >
    );
};

export const HierarchyNode = React.memo(HierarchyNodeBase, (prev, next) => {
    const agentPrev = prev.agent;
    const agentNext = next.agent;

    if (!agentPrev || !agentNext) return false;

    // Optimization: Skip re-render if core visual state is identical
    return (
        agentPrev.status === agentNext.status &&
        agentPrev.currentTask === agentNext.currentTask &&
        agentPrev.costUsd === agentNext.costUsd &&
        agentPrev.tokensUsed === agentNext.tokensUsed &&
        agentPrev.model === agentNext.model &&
        agentPrev.model2 === agentNext.model2 &&
        agentPrev.model3 === agentNext.model3 &&
        agentPrev.activeModelSlot === agentNext.activeModelSlot &&
        agentPrev.role === agentNext.role &&
        agentPrev.valence === agentNext.valence &&
        prev.isActive === next.isActive &&
        prev.missionObjective === next.missionObjective &&
        prev.isAlpha === next.isAlpha &&
        agentPrev.activeMission?.is_degraded === agentNext.activeMission?.is_degraded &&
        agentPrev.themeColor === agentNext.themeColor &&
        agentPrev.failureCount === agentNext.failureCount &&
        agentPrev.lastFailureAt === agentNext.lastFailureAt
    );
});
