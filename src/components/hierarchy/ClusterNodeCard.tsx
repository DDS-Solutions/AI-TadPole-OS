/**
 * @docs ARCHITECTURE:UI
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Hierarchy / ClusterNodeCard
 * - **Primary Entrypoints**: `ClusterNodeCard`, `AgentChain`, `ClusterNodeCardProps`, `SharedNodeProps`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import React, { memo } from 'react';
import type { Agent } from '../../types';
import { Hierarchy_Node } from '../Hierarchy_Node';
import { getThemeClasses, type ThemeKey } from './theme';

export type { ThemeKey };

export type SharedNodeProps = {
    available_roles: string[];
    on_role_change: (id: string, role: string) => void;
    on_skill_trigger: (id: string, skill: string, slot?: 1 | 2 | 3) => void;
    on_configure_click: (id: string) => void;
    on_model_change: (id: string, m: string) => void;
    on_model_2_change: (id: string, m: string) => void;
    on_model_3_change: (id: string, m: string) => void;
    on_update: (id: string, updates: Partial<Agent>) => void;
};

export interface AgentChain {
    id: string;
    name: string;
    theme: string;
    alpha_id?: string;
    objective?: string;
    is_active?: boolean;
    agents: Agent[];
}

export interface ClusterNodeCardProps {
    chain: AgentChain;
    dropdown_open_id: string | null;
    clusters: { id: string; name: string; theme: string; alpha_id?: string; collaborators: string[]; is_active?: boolean }[];
    shared_node_props: SharedNodeProps;
}

export const ClusterNodeCard: React.FC<ClusterNodeCardProps> = memo(({
    chain,
    dropdown_open_id,
    clusters,
    shared_node_props
}) => {
    const chainTheme = chain.theme || 'zinc';
    const currentThemeClasses = getThemeClasses(chainTheme);

    return (
        <div className="flex flex-col items-center gap-12 relative">
            <div className="mb-4 text-center">
                <h3 className={`text-[10px] font-bold uppercase tracking-[0.2em] mb-1 ${currentThemeClasses.heading}`}>
                    Chain {chain.id}
                </h3>
                <p className="text-[9px] text-zinc-500 font-medium">{chain.name}</p>
            </div>

            <div className="flex flex-col gap-12 relative">
                {chain.agents.map((agent: Agent, idx: number) => (
                    <div key={agent.id} className="relative w-[350px]" style={{ zIndex: dropdown_open_id === agent.id ? 110 : (100 - idx) }}>
                        <Hierarchy_Node
                            agent={agent}
                            theme_color={chainTheme}
                            is_alpha={agent.id === chain.alpha_id}
                            is_active={clusters.find(c => c.id === chain.id)?.is_active}
                            mission_objective={chain.objective}
                            {...shared_node_props}
                        />

                        {idx < chain.agents.length - 1 && (
                            <div
                                aria-hidden="true"
                                className={`absolute top-full left-1/2 -translate-x-1/2 h-12 w-px 
                                ${currentThemeClasses.bg}
                                ${chain.is_active || (chain.agents[idx].status !== 'offline' && chain.agents[idx].status !== 'idle') || (chain.agents[idx + 1].status !== 'offline' && chain.agents[idx + 1].status !== 'idle') ? currentThemeClasses.pulse : ''}`}
                            />
                        )}
                    </div>
                ))}
            </div>
        </div>
    );
});

ClusterNodeCard.displayName = 'ClusterNodeCard';
