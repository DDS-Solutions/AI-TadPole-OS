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
import React, { useState, useMemo, useRef, useLayoutEffect } from 'react';
import { createPortal } from 'react-dom';
import { motion, AnimatePresence } from 'framer-motion';
import { Swarm_Oversight_Node } from './Swarm_Oversight_Node';
import { get_agent_status_styles } from '../utils/agent_uiutils';
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
    const [is_oversight_open, set_is_oversight_open] = useState(false);
    const [is_health_open, set_is_health_open] = useState(false);
    const node_ref = useRef<HTMLDivElement>(null);
    const [portal_pos, set_portal_pos] = useState({ top: 0, left: 0 });

    useLayoutEffect(() => {
        if (is_oversight_open && node_ref.current) {
            const rect = node_ref.current.getBoundingClientRect();
            set_portal_pos({ top: rect.top, left: rect.right + 20 });
        }
    }, [is_oversight_open]);

    const { clusters, active_proposals } = use_workspace_store();
    
    const cluster = useMemo(() => 
        clusters.find(c => (c.collaborators || []).includes(agent?.id || '')),
    [clusters, agent?.id]);

    const has_oversight = cluster ? !!active_proposals[cluster.id] : false;

    if (!agent) return null;

    const status_styles = get_agent_status_styles(agent.status);
    const agent_color = agent.theme_color || status_styles.hex;

    return (
        <div ref={node_ref} className="relative group transition-all duration-300 w-full z-0">
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

            {/* Swarm Oversight Integration (Alpha Only) - Rendered via Portal to bypass stacking context issues */}
            {is_oversight_open && is_alpha && has_oversight && cluster && createPortal(
                <AnimatePresence>
                    <motion.div 
                        initial={{ opacity: 0, x: -20, scale: 0.95 }}
                        animate={{ opacity: 1, x: 0, scale: 1 }}
                        exit={{ opacity: 0, x: -20, scale: 0.95 }}
                        transition={{ type: "spring", damping: 20, stiffness: 300 }}
                        className="fixed z-[9999] pointer-events-none"
                        style={{
                            top: portal_pos.top,
                            left: portal_pos.left,
                        }}
                    >
                        <div className="pointer-events-auto">
                            <Swarm_Oversight_Node 
                                cluster_id={cluster.id} 
                                on_close={() => set_is_oversight_open(false)}
                            />
                        </div>
                    </motion.div>
                </AnimatePresence>,
                document.body
            )}
        </div >
    );
};

export const Hierarchy_Node = React.memo(Hierarchy_Node_Base, (prev, next) => {
    // Optimization: Leverage immutable state reference checks for the agent object
    // and shallow comparison for primitive props to avoid brittle manual property listing.
    return (
        prev.agent === next.agent &&
        prev.is_active === next.is_active &&
        prev.mission_objective === next.mission_objective &&
        prev.is_alpha === next.is_alpha &&
        prev.is_root === next.is_root &&
        prev.available_roles === next.available_roles &&
        prev.theme_color === next.theme_color
    );
});


// Metadata: [Hierarchy_Node]
