/**
 * @docs ARCHITECTURE:Telemetry
 * @docs OPERATIONS_MANUAL:Telemetry
 * 
 * ### AI Assist Note
 * **UI Component**: Real-time Force-Directed Graph for swarm intelligence visualization. 
 * Orchestrates high-speed telemetry ingestion (10Hz binary pulse) and renders neural node identity, halos, and data-flow pulses via custom Canvas operations.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Pulse starvation if `swarm_pulse` events drop, node collision desync due to force engine instability, or memory leak during high-frequency WebGL/Canvas context updates.
 * - **Telemetry Link**: Search for `[Swarm_Visualizer]` or `swarm_pulse` in UI tracing.
 */

import React, { useEffect, useMemo, useRef } from 'react';
import ForceGraph2D, { type ForceGraphMethods } from 'react-force-graph-2d';
import { ExternalLink } from 'lucide-react';
import { use_agent_store } from '../stores/agent_store';
import { use_sovereign_store } from '../stores/sovereign_store';
import { THEME_COLORS, GRAPH_THEME } from '../constants/theme';
import { i18n } from '../i18n';
import { tadpole_os_socket } from '../services/socket';
import { type Swarm_Pulse } from '../types';
import { SWARM_NODE_STATUS } from '../constants/swarm';

/**
 * Swarm_Visualizer
 * High-performance real-time swarm intelligence visualizer.
 * Integrates with high-speed binary telemetry (MessagePack) at 10Hz.
 */

interface GraphNode {
    id: string;
    name: string;
    status: number;
    battery: number;
    signal: number;
    progress: number;
    x?: number;
    y?: number;
    vx?: number;
    vy?: number;
}

interface GraphLink {
    source: string | GraphNode;
    target: string | GraphNode;
}

interface ExtendedForceGraphMethods {
    d3AlphaTarget(target: number): void;
    refresh(): void;
}

export const Swarm_Visualizer: React.FC<{ is_detached?: boolean, on_detach?: () => void }> = ({ is_detached = false, on_detach }) => {
    const fg_ref = useRef<ForceGraphMethods<GraphNode, GraphLink> | undefined>(undefined);
    const [graph_data, set_graph_data] = React.useState<{ nodes: GraphNode[], links: GraphLink[] }>({ nodes: [], links: [] });
    
    // Sovereign Actions for Focus
    const set_selected_agent_id = use_sovereign_store(s => s.set_selected_agent_id);
    const set_scope = use_sovereign_store(s => s.set_scope);
    const set_target_agent = use_sovereign_store(s => s.set_target_agent);

    // ### 🧠 State Synchronization: Telemetry Ingestion
    useEffect(() => {
        const unsubscribe = tadpole_os_socket.subscribe_swarm_pulse((pulse: Swarm_Pulse) => {
            // Get current agents state
            const current_agents = use_agent_store.getState().agents;
            
            set_graph_data(prev => {
                const prev_nodes = prev.nodes;
                const prev_links = prev.links;

                // 🛡️ [Robustness] Use ID fingerprint for structural change detection
                const pulse_node_ids = pulse.nodes.map(n => n.id).sort().join(',');
                const prev_node_ids = prev_nodes.map(n => n.id).sort().join(',');
                const has_structural_change = pulse_node_ids !== prev_node_ids || 
                                             pulse.edges.length !== prev_links.length;

                // 1. Map Nodes (Preserve D3 references to avoid simulation resets)
                const new_nodes = pulse.nodes.map(pulse_node => {
                    const agent = Array.from(current_agents.values()).find(a => a.id === pulse_node.id);
                    const existing = prev_nodes.find(n => n.id === pulse_node.id);

                    // 🛡️ Telemetry Sanitization & "Grip" Logic
                    const sanitized_metrics = {
                        status: pulse_node.status ?? SWARM_NODE_STATUS.IDLE,
                        battery: Math.max(0, Math.min(100, pulse_node.battery || 0)),
                        signal: Math.max(0, Math.min(100, pulse_node.signal || 0)),
                        progress: Math.max(0, Math.min(100, pulse_node.progress || 0)),
                    };

                    if (existing) {
                        Object.assign(existing, {
                            ...sanitized_metrics,
                            name: agent?.name || (pulse_node.status === SWARM_NODE_STATUS.MISSION_HUB ? "Mission Hub" : pulse_node.id),
                        });
                        return existing;
                    }

                    return {
                        ...pulse_node,
                        ...sanitized_metrics,
                        name: agent?.name || (pulse_node.status === SWARM_NODE_STATUS.MISSION_HUB ? "Mission Hub" : pulse_node.id),
                        x: (Math.random() - 0.5) * 100,
                        y: (Math.random() - 0.5) * 100,
                    };
                });

                // 2. Map Links
                const new_links = (pulse.edges || []).map(edge => ({
                    source: edge.source,
                    target: edge.target,
                    value: 1
                }));

                if (has_structural_change) {
                    // Force a new object for structural changes
                    setTimeout(() => {
                        (fg_ref.current as unknown as ExtendedForceGraphMethods)?.d3AlphaTarget(0.1);
                        setTimeout(() => (fg_ref.current as unknown as ExtendedForceGraphMethods)?.d3AlphaTarget(0), 1000);
                    }, 0);
                    return { nodes: new_nodes, links: new_links };
                } else {
                    // Return same object but refresh canvas
                    (fg_ref.current as unknown as ExtendedForceGraphMethods)?.refresh();
                    return prev;
                }
            });
        });

        return () => unsubscribe();
    }, []);

    // ### 🎨 High-Performance Rendering: Fast-Path Canvas Pipeline
    const node_canvas_object = useMemo(() => (node: GraphNode, ctx: CanvasRenderingContext2D, global_scale: number) => {
        ctx.save(); // Isolate context state
        
        const label = node.name;
        const font_size = 11 / global_scale;
        const radius = GRAPH_THEME.NODE_RADIUS;
        
        // Extended Status Colors for Mission Hubs
        const status_color = node.status === SWARM_NODE_STATUS.BUSY ? THEME_COLORS.BUSY : 
                           (node.status === SWARM_NODE_STATUS.ERROR ? THEME_COLORS.ERROR : 
                           (node.status === SWARM_NODE_STATUS.DEGRADED ? THEME_COLORS.DEGRADED : 
                           (node.status === SWARM_NODE_STATUS.MISSION_HUB ? '#A855F7' : THEME_COLORS.IDLE)));

        // 🛡️ Drawing Layer 1: Glow Halo (Busy, Error, or Hub)
        if (node.status === SWARM_NODE_STATUS.BUSY || node.status === SWARM_NODE_STATUS.ERROR || node.status === SWARM_NODE_STATUS.MISSION_HUB) {
             ctx.beginPath();
             ctx.arc(node.x ?? 0, node.y ?? 0, radius * 1.6, 0, 2 * Math.PI, false);
             ctx.fillStyle = node.status === SWARM_NODE_STATUS.BUSY ? THEME_COLORS.GLOW_CYAN : 
                            (node.status === SWARM_NODE_STATUS.MISSION_HUB ? 'rgba(168, 85, 247, 0.15)' : THEME_COLORS.GLOW_ROSE);
             ctx.fill();
        }

        // 🛡️ Drawing Layer 2: Core Neural Identity
        ctx.beginPath();
        ctx.arc(node.x ?? 0, node.y ?? 0, radius, 0, 2 * Math.PI, false);
        ctx.fillStyle = status_color;
        ctx.fill();
        
        // 🛡️ Drawing Layer 3: Busy Pulse Animation
        if (node.status === SWARM_NODE_STATUS.BUSY) {
             const pulse = (Math.sin(Date.now() / 150) + 1) / 2;
             ctx.beginPath();
             ctx.arc(node.x ?? 0, node.y ?? 0, radius + (pulse * 3), 0, 2 * Math.PI, false);
             ctx.strokeStyle = `rgba(34, 211, 238, ${0.8 - pulse})`;
             ctx.lineWidth = 1 / global_scale;
             ctx.stroke();
        }

        // 🛡️ Drawing Layer 4: Semantic Labels
        if (global_scale > 1.2) {
            ctx.font = `${font_size}px ${GRAPH_THEME.LABEL_FONT}`;
            ctx.textAlign = 'center';
            ctx.textBaseline = 'top';
            ctx.fillStyle = 'white';
            // 🛡️ Sanitization: Prevent text-based overflows
            const safe_label = label.length > 24 ? label.substring(0, 21) + '...' : label;
            ctx.fillText(safe_label, node.x ?? 0, (node.y ?? 0) + radius + 4);
        }
        
        // 🛡️ Drawing Layer 5: Resource Telemetry (Bars)
        const bar_w = 12 / global_scale;
        const bar_h = 2 / global_scale;
        if (global_scale > 1.5 && node.status !== SWARM_NODE_STATUS.MISSION_HUB) { // Hubs don't show battery
            ctx.fillStyle = GRAPH_THEME.TELEMETRY_BG;
            ctx.fillRect((node.x ?? 0) - bar_w/2, (node.y ?? 0) - radius - 5, bar_w, bar_h);
            ctx.fillStyle = node.battery > 20 ? THEME_COLORS.SUCCESS : THEME_COLORS.ERROR;
            ctx.fillRect((node.x ?? 0) - bar_w/2, (node.y ?? 0) - radius - 5, bar_w * (node.battery / 100), bar_h);
        }

        ctx.restore(); // Restore context state
    }, []);

    const current_node_count = graph_data.nodes.length;

    return (
        <div className="w-full h-full relative bg-zinc-950 rounded-[2.5rem] border border-zinc-900/50 shadow-2xl overflow-hidden group">
            <ForceGraph2D
                ref={fg_ref}
                graphData={graph_data}
                nodeCanvasObject={node_canvas_object}
                nodePointerAreaPaint={(node: GraphNode, color, ctx) => {
                    ctx.fillStyle = color;
                    ctx.beginPath(); ctx.arc(node.x ?? 0, node.y ?? 0, 8, 0, 2 * Math.PI, false); ctx.fill();
                }}
                linkColor={() => THEME_COLORS.NEURAL_GRID}
                linkWidth={GRAPH_THEME.LINK_WIDTH}
                linkDirectionalParticles={2}
                linkDirectionalParticleSpeed={GRAPH_THEME.PARTICLE_SPEED}
                backgroundColor="rgba(0,0,0,0)"
                d3AlphaDecay={0.02}
                d3VelocityDecay={0.3}
                cooldownTicks={100}
                onNodeClick={(node: GraphNode) => {
                    if (node.status === SWARM_NODE_STATUS.MISSION_HUB) return; // Mission Hubs aren't clickable
                    set_selected_agent_id(node.id);
                    set_scope('agent');
                    set_target_agent(node.name || node.id);
                    
                    if (node.x !== undefined && node.y !== undefined) {
                        fg_ref.current?.centerAt(node.x, node.y, 400);
                        fg_ref.current?.zoom(2.5, 400);
                    }
                }}
            />
            
            {/* HUD Overlay */}
            <div className="absolute top-8 left-8 pointer-events-none select-none">
                 <div className="flex flex-col gap-1.5">
                    <div className="flex items-center gap-3">
                        <div className="w-2.5 h-2.5 rounded-full bg-cyan-500 animate-pulse shadow-[0_0_15px_#06b6d4]" />
                        <h2 className="text-xs font-black text-white uppercase tracking-[0.4em]">{i18n.t('swarm_visualizer.title')}</h2>
                    </div>
                    <div className="flex items-center gap-4 ml-7">
                        <p className="text-[9px] text-zinc-500 font-bold uppercase tracking-wider">{i18n.t('swarm_visualizer.telemetry_10hz')}</p>
                        <div className="w-px h-2 bg-zinc-800" />
                        <p className="text-[9px] text-zinc-500 font-bold uppercase tracking-wider">
                            {i18n.t('swarm_visualizer.nodes_online', { count: current_node_count })}
                        </p>
                    </div>
                </div>
            </div>

            {/* Detach Window Button */}
            {!is_detached && (
                <div className="absolute top-8 right-8 flex gap-2">
                    <button 
                        onClick={() => {
                            if (on_detach) {
                                on_detach();
                            } else {
                                window.open(window.location.origin + '/detached/swarm-pulse', 'SwarmPulse', 'width=1000,height=800,noopener,noreferrer');
                            }
                        }}
                        className="p-2.5 bg-zinc-900/40 backdrop-blur-md border border-zinc-800 rounded-xl text-zinc-400 hover:bg-zinc-800 hover:text-white transition-all shadow-lg group/detach"
                        title={i18n.t('swarm_visualizer.detach_tooltip')}
                    >
                        <ExternalLink size={16} className="group-hover/detach:scale-110 transition-transform" />
                    </button>
                </div>
            )}

            {/* Bottom Controls Panel */}
            <div className="absolute bottom-8 right-8 flex gap-2">
                 <button 
                    onClick={() => fg_ref.current?.zoomToFit(400, 50)}
                    className="px-4 py-2 bg-zinc-900/40 backdrop-blur-md border border-zinc-800 rounded-xl text-[9px] font-bold text-zinc-400 uppercase tracking-widest hover:bg-zinc-800 hover:text-white transition-all shadow-lg overflow-hidden group/btn"
                 >
                    <span className="relative z-10">{i18n.t('swarm_visualizer.recenter_swarm')}</span>
                 </button>
            </div>
        </div>
    );
};
