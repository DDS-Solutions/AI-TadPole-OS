/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Main agent card within the Neural Command Hierarchy. 
 * Orchestrates status indicators, mission data, and per-model configuration badges with high-fidelity `framer-motion` glow effects.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Glow effect performance degradation during swarm storms, dropdown collision with parent containers (z-index), or oversight portal spawn failures for Alpha nodes.
 * - **Telemetry Link**: Search for `[Hierarchy_Node]` or `Neural Command Hierarchy` in UI tracing.
 */

/**
 * Hierarchy_Node
 * A specialized agent card component designed for the Neural Command Hierarchy.
 * Features status indicators, mission data, and per-model configuration badges.
 * Refactored for strict snake_case compliance for backend parity.
 */
import React, { useState, useMemo } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Swarm_Oversight_Node } from './Swarm_Oversight_Node';
import { get_agent_status_styles } from '../utils/agent_uiutils';
import { use_dropdown_store } from '../stores/dropdown_store';
import { use_workspace_store } from '../stores/workspace_store';
import type { Agent } from '../types';

// Sub-components
import { Node_Header } from './hierarchy/Node_Header';
import { Node_Stats } from './hierarchy/Node_Stats';
import { Node_Mission } from './hierarchy/Node_Mission';
import { Node_Task_Box } from './hierarchy/Node_Task_Box';
import { Node_Model_Slots } from './hierarchy/Node_Model_Slots';
import { Node_Health } from './hierarchy/Node_Health';

/**
 * Hierarchy_Node_Props
 */
interface Hierarchy_Node_Props {
    agent?: Agent;
    is_root?: boolean;
    on_skill_trigger?: (agent_id: string, skill: string, slot?: 1 | 2 | 3) => void;
    on_configure_click?: (agent_id: string) => void;
    is_alpha?: boolean;
    is_active?: boolean;
    mission_objective?: string;
    on_model_change?: (agent_id: string, new_model: string) => void;
    on_model_2_change?: (agent_id: string, new_model: string) => void;
    on_model_3_change?: (agent_id: string, new_model: string) => void;
    on_role_change?: (agent_id: string, new_role: string) => void;
    on_update?: (agent_id: string, updates: Partial<Agent>) => void;
    available_roles?: string[];
    theme_color?: string;
}

const Hierarchy_Node_Base: React.FC<Hierarchy_Node_Props> = ({
    agent,
    is_root = false,
    on_skill_trigger,
    on_configure_click,
    is_alpha = false,
    is_active = false,
    mission_objective,
    on_model_change,
    on_model_2_change,
    on_model_3_change,
    on_role_change,
    on_update,
    available_roles = [],
}): React.ReactElement | null => {
    const is_any_dropdown_open = use_dropdown_store(s => s.open_id === agent?.id);
    const [is_oversight_open, set_is_oversight_open] = useState(false);
    const [is_health_open, set_is_health_open] = useState(false);

    const { clusters, active_proposals } = use_workspace_store();
    
    const cluster = useMemo(() => 
        clusters.find(c => (c.collaborators || []).map(String).includes(String(agent?.id))),
    [clusters, agent?.id]);

    const has_oversight = cluster ? !!active_proposals[cluster.id] : false;

    if (!agent) return null;

    const status_styles = get_agent_status_styles(agent.status);
    const agent_color = agent.theme_color || status_styles.hex;

    return (
        <div className={`relative group transition-all duration-300 w-full ${is_any_dropdown_open ? 'z-[100]' : 'z-0'}`}>
            <div
                className={`
                    relative z-10 p-3 rounded-xl border backdrop-blur-xl transition-all duration-300
                    bg-[#1b1b1e]/60
                    ${agent.status !== 'offline' && agent.status !== 'idle' ? 'border-zinc-800' : 'border-zinc-800/30'}
                    hover:border-zinc-600 hover:scale-[1.02] active:scale-[0.98]
                    overflow-visible flex flex-col gap-3 shadow-2xl
                `}
                style={{
                    borderColor: `${agent_color}30`,
                    boxShadow: `0 0 15px ${agent_color}10`,
                    color: agent_color
                }}
            >
                {/* Header: Identity & Status */}
                <Node_Header
                    agent={agent}
                    is_alpha={is_alpha}
                    is_active={is_active}
                    available_roles={available_roles}
                    on_role_change={on_role_change}
                    on_configure_click={on_configure_click}
                    has_oversight={has_oversight}
                    is_oversight_open={is_oversight_open}
                    on_oversight_toggle={() => set_is_oversight_open(!is_oversight_open)}
                    is_health_open={is_health_open}
                    on_health_toggle={() => set_is_health_open(!is_health_open)}
                />

                {/* Metrics & Skills Row */}
                <Node_Stats agent={agent} on_skill_trigger={on_skill_trigger} />

                {/* Mission Badge area */}
                <Node_Mission agent={agent} is_active={is_active} mission_objective={mission_objective} />

                {/* Current Task Box */}
                <Node_Task_Box agent={agent} />

                {/* Model Badges (Interactive Picker) */}
                <Node_Model_Slots
                    agent={agent}
                    on_model_change={on_model_change}
                    on_model_2_change={on_model_2_change}
                    on_model_3_change={on_model_3_change}
                    on_update={on_update}
                />

                {/* Health/Security Overlay */}
                <AnimatePresence>
                    {is_health_open && (
                        <Node_Health 
                            agent={agent} 
                            on_close={() => set_is_health_open(false)} 
                        />
                    )}
                </AnimatePresence>
            </div>

            {/* Connection Point Indicators */}
            {!is_root && (
                <div className="absolute -top-1 left-1/2 -translate-x-1/2 w-2 h-2 rounded-full border border-current bg-zinc-950 opacity-0 group-hover:opacity-100 transition-opacity" />
            )}
            <div className="absolute -bottom-1 left-1/2 -translate-x-1/2 w-2 h-2 rounded-full border border-current bg-zinc-950 opacity-0 group-hover:opacity-100 transition-opacity" />

            {/* Neural Glow (Active Mission) */}
            <AnimatePresence>
                {(is_active || agent.status === 'active') && (
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
                        style={{ backgroundColor: `${agent_color}30` }}
                    />
                )}
            </AnimatePresence>

            {/* Swarm Oversight Integration (Alpha Only) */}
            <AnimatePresence>
                {is_oversight_open && is_alpha && has_oversight && cluster && (
                    <motion.div 
                        initial={{ opacity: 0, x: -20, scale: 0.95 }}
                        animate={{ opacity: 1, x: 0, scale: 1 }}
                        exit={{ opacity: 0, x: -20, scale: 0.95 }}
                        transition={{ type: "spring", damping: 20, stiffness: 300 }}
                        className="absolute top-0 -right-[420px] pointer-events-none z-[110]"
                    >
                        <div className="pointer-events-auto">
                            <Swarm_Oversight_Node 
                                cluster_id={cluster.id} 
                                on_close={() => set_is_oversight_open(false)}
                            />
                        </div>
                    </motion.div>
                )}
            </AnimatePresence>
        </div >
    );
};

export const Hierarchy_Node = React.memo(Hierarchy_Node_Base, (prev, next) => {
    const agent_prev = prev.agent;
    const agent_next = next.agent;

    if (!agent_prev || !agent_next) return false;

    // Optimization: Skip re-render if core visual state is identical
    return (
        agent_prev.status === agent_next.status &&
        agent_prev.current_task === agent_next.current_task &&
        agent_prev.cost_usd === agent_next.cost_usd &&
        agent_prev.tokens_used === agent_next.tokens_used &&
        agent_prev.model === agent_next.model &&
        agent_prev.model_2 === agent_next.model_2 &&
        agent_prev.model_3 === agent_next.model_3 &&
        agent_prev.active_model_slot === agent_next.active_model_slot &&
        agent_prev.role === agent_next.role &&
        agent_prev.valence === agent_next.valence &&
        prev.is_active === next.is_active &&
        prev.mission_objective === next.mission_objective &&
        prev.is_alpha === next.is_alpha &&
        agent_prev.active_mission?.is_degraded === agent_next.active_mission?.is_degraded &&
        agent_prev.theme_color === agent_next.theme_color &&
        agent_prev.failure_count === agent_next.failure_count &&
        agent_prev.last_failure_at === agent_next.last_failure_at
    );
});

