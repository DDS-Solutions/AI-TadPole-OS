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

/**
 * Swarm_Visualizer
 * High-performance real-time swarm intelligence visualizer.
 * Integrates with high-speed binary telemetry (MessagePack) at 10Hz.
 */

// Constants moved to src/constants/theme.ts

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

export const Swarm_Visualizer: React.FC<{ is_detached?: boolean, on_detach?: () => void }> = ({ is_detached = false, on_detach }) => {
    const fg_ref = useRef<ForceGraphMethods<GraphNode, GraphLink> | undefined>(undefined);
    const graph_data_ref = useRef<{ nodes: GraphNode[], links: GraphLink[] }>({ nodes: [], links: [] });
    // Trigger React state only when the structural topology changes (node/link count)
    const [, set_graph_structure_version] = React.useState<number>(0);
    const { agents } = use_agent_store();
    
    // Sovereign Actions for Focus
    const set_selected_agent_id = use_sovereign_store(s => s.set_selected_agent_id);
    const set_scope = use_sovereign_store(s => s.set_scope);
    const set_target_agent = use_sovereign_store(s => s.set_target_agent);

    // ### 🧠 State Synchronization: Telemetry Ingestion
    // Subscribes to the high-speed (10Hz) binary telemetry pulse from the backend.
    // Maps the incoming 'SwarmPulse' protocol buffer records into the local D3 
    // Graph representation while preserving node momentum and position clusters.
    useEffect(() => {
        // Subscribe to high-speed binary pulses
        const unsubscribe = tadpole_os_socket.subscribe_swarm_pulse((pulse: Swarm_Pulse) => {
            const current = graph_data_ref.current;
            
            // 1. Map Nodes (Mutate in place to avoid React reconciliation overhead)
            const new_nodes = (pulse.nodes || []).map(pulse_node => {
                const agent = agents.find(a => a.id === pulse_node.id);
                const existing = current.nodes.find(n => n.id === pulse_node.id);
                
                return {
                    ...pulse_node,
                    name: agent?.name || pulse_node.id,
                    // Preserve position/velocity from the D3 force engine
                    x: existing?.x ?? (Math.random() - 0.5) * 200,
                    y: existing?.y ?? (Math.random() - 0.5) * 200,
                    vx: existing?.vx,
                    vy: existing?.vy
                };
            });

            // 2. Map Links
            const new_links = (pulse.edges || []).map(edge => ({
                source: edge.source,
                target: edge.target
            }));

            // Check if structure changed (additions/deletions)
            const structure_changed = 
                new_nodes.length !== current.nodes.length || 
                new_links.length !== current.links.length;

            graph_data_ref.current = { nodes: new_nodes, links: new_links };

            if (structure_changed) {
                set_graph_structure_version((v: number) => v + 1);
            }
        });

        return () => unsubscribe();
    }, [agents]);

    // ### 🎨 High-Performance Rendering: Fast-Path Canvas Pipeline
    // To handle swarms with 100+ agents without saturating the Main Thread, 
    // we bypass the React Reconciliation loop for node aesthetics and 
    // draw directly to the Canvas context using 2D primitives.
    const node_canvas_object = useMemo(() => (node: GraphNode, ctx: CanvasRenderingContext2D, global_scale: number) => {
        ctx.save(); // Isolate context state
        
        const label = node.name;
        const font_size = 11 / global_scale;
        const radius = GRAPH_THEME.NODE_RADIUS;
        
        const status_color = node.status === 1 ? THEME_COLORS.BUSY : (node.status === 2 ? THEME_COLORS.ERROR : (node.status === 3 ? THEME_COLORS.DEGRADED : THEME_COLORS.IDLE));

        // 🛡️ Drawing Layer 1: Glow Halo
        if (node.status === 1 || node.status === 2) {
             ctx.beginPath();
             ctx.arc(node.x ?? 0, node.y ?? 0, radius * 1.6, 0, 2 * Math.PI, false);
             ctx.fillStyle = node.status === 1 ? THEME_COLORS.GLOW_CYAN : THEME_COLORS.GLOW_ROSE;
             ctx.fill();
        }

        // 🛡️ Drawing Layer 2: Core Neural Identity
        ctx.beginPath();
        ctx.arc(node.x ?? 0, node.y ?? 0, radius, 0, 2 * Math.PI, false);
        ctx.fillStyle = status_color;
        ctx.fill();
        
        // 🛡️ Drawing Layer 3: Busy Pulse Animation
        if (node.status === 1) {
             const pulse = (Math.sin(Date.now() / 150) + 1) / 2;
             ctx.beginPath();
             ctx.arc(node.x ?? 0, node.y ?? 0, radius + (pulse * 3), 0, 2 * Math.PI, false);
             ctx.strokeStyle = `rgba(34, 211, 238, ${0.8 - pulse})`;
             ctx.lineWidth = 1 / global_scale;
             ctx.stroke();
        }

        // 🛡️ Drawing Layer 4: Semantic Labels
        if (global_scale > 2) {
            ctx.font = `${font_size}px ${GRAPH_THEME.LABEL_FONT}`;
            ctx.textAlign = 'center';
            ctx.textBaseline = 'top';
            ctx.fillStyle = 'white';
            ctx.fillText(label, node.x ?? 0, (node.y ?? 0) + radius + 4);
        }
        
        // 🛡️ Drawing Layer 5: Resource Telemetry (Bars)
        const bar_w = 12 / global_scale;
        const bar_h = 2 / global_scale;
        if (global_scale > 1.5) {
            ctx.fillStyle = GRAPH_THEME.TELEMETRY_BG;
            ctx.fillRect((node.x ?? 0) - bar_w/2, (node.y ?? 0) - radius - 5, bar_w, bar_h);
            ctx.fillStyle = node.battery > 20 ? THEME_COLORS.SUCCESS : THEME_COLORS.ERROR;
            ctx.fillRect((node.x ?? 0) - bar_w/2, (node.y ?? 0) - radius - 5, bar_w * (node.battery / 100), bar_h);
        }

        ctx.restore(); // Restore context state
    }, []);

    const current_node_count = graph_data_ref.current.nodes.length;

    return (
        <div className="w-full h-full relative bg-zinc-950 rounded-[2.5rem] border border-zinc-900/50 shadow-2xl overflow-hidden group">
            <ForceGraph2D
                ref={fg_ref}
                /* eslint-disable-next-line react-hooks/refs */
                graphData={graph_data_ref.current}
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
                // ### 🧪 D3 Force Optimization
                // alphaDecay: Higher values (0.02) ensure the graph stabilizes 
                // quickly after node injections.
                // velocityDecay: (0.3) provides 'viscosity' to prevent 
                // erratic jitter during high-frequency pulse updates.
                d3AlphaDecay={0.02}
                d3VelocityDecay={0.3}
                cooldownTicks={100}
                onNodeClick={(node: GraphNode) => {
                    // Focus Agent Logs & Scope
                    set_selected_agent_id(node.id);
                    set_scope('agent');
                    set_target_agent(node.name || node.id);
                    
                    // Center View on Node
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
                            {/* eslint-disable-next-line react-hooks/refs */}
                            {i18n.t('swarm_visualizer.nodes_online', { count: current_node_count })}
                        </p>
                    </div>
                </div>
            </div>

            {/* Detach Window Button - Hidden if already detached */}
            {!is_detached && (
                <div className="absolute top-8 right-8 flex gap-2">
                    <button 
                        onClick={() => {
                            if (on_detach) {
                                on_detach();
                            } else {
                                window.open(window.location.origin + '/detached/swarm-pulse', 'SwarmPulse', 'width=1000,height=800');
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


// Metadata: [Swarm_Visualizer]
