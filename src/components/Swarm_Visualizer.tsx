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

import React, { useEffect, useRef, useState, useMemo } from 'react';
import ForceGraph2D from 'react-force-graph-2d';
import { tadpole_os_socket } from '../services/socket';
import type { Swarm_Pulse } from '../types';
import { use_agent_store } from '../stores/agent_store';
import { use_sovereign_store } from '../stores/sovereign_store';

/**
 * Swarm_Visualizer
 * High-performance real-time swarm intelligence visualizer.
 * Integrates with high-speed binary telemetry (MessagePack) at 10Hz.
 */

const COLORS = {
    IDLE: '#71717a',    // zinc-500
    BUSY: '#22d3ee',    // cyan-400
    ERROR: '#f43f5e',   // rose-500
    DEGRADED: '#f59e0b', // amber-500
    GLOW_CYAN: 'rgba(34, 211, 238, 0.4)',
    GLOW_ROSE: 'rgba(244, 63, 94, 0.4)',
};

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

export const Swarm_Visualizer: React.FC = () => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const fg_ref = useRef<any>(null);
    const [graph_data, set_graph_data] = useState<{ nodes: GraphNode[], links: GraphLink[] }>({ nodes: [], links: [] });
    const { agents } = use_agent_store();
    
    // Sovereign Actions for Focus
    const set_selected_agent_id = use_sovereign_store(s => s.set_selected_agent_id);
    const set_scope = use_sovereign_store(s => s.set_scope);
    const set_target_agent = use_sovereign_store(s => s.set_target_agent);

    useEffect(() => {
        // Subscribe to high-speed binary pulses
        const unsubscribe = tadpole_os_socket.subscribe_swarm_pulse((pulse: Swarm_Pulse) => {
            set_graph_data(prev => {
                // 1. Map Nodes
                const nodes = pulse.nodes.map(node => {
                    const agent = agents.find(a => a.id === node.id);
                    const existing = prev.nodes.find(n => n.id === node.id);
                    
                    return {
                        ...node,
                        name: agent?.name || node.id,
                        // Preserve position if already calculated by force engine
                        x: existing?.x ?? (Math.random() - 0.5) * 200,
                        y: existing?.y ?? (Math.random() - 0.5) * 200,
                    };
                });

                // 2. Map Links
                const links = pulse.edges.map(edge => ({
                    source: edge.source,
                    target: edge.target
                }));

                // Avoid unnecessary react state updates if nothing changed
                return { nodes, links };
            });
        });

        return () => unsubscribe();
    }, [agents]);

    // Fast-path Canvas Rendering for Node Aesthetics
    const node_canvas_object = useMemo(() => (node: GraphNode, ctx: CanvasRenderingContext2D, global_scale: number) => {
        const label = node.name;
        const font_size = 11 / global_scale;
        const radius = 5;
        
        const status_color = node.status === 1 ? COLORS.BUSY : (node.status === 2 ? COLORS.ERROR : (node.status === 3 ? COLORS.DEGRADED : COLORS.IDLE));

        // 1. Draw Glow Halo (Only for active/error states)
        if (node.status === 1 || node.status === 2) {
             ctx.beginPath();
             ctx.arc(node.x ?? 0, node.y ?? 0, radius * 1.6, 0, 2 * Math.PI, false);
             ctx.fillStyle = node.status === 1 ? COLORS.GLOW_CYAN : COLORS.GLOW_ROSE;
             ctx.fill();
        }

        // 2. Draw Main Node
        ctx.beginPath();
        ctx.arc(node.x ?? 0, node.y ?? 0, radius, 0, 2 * Math.PI, false);
        ctx.fillStyle = status_color;
        ctx.fill();
        
        // 3. Busy Pulse Animation
        if (node.status === 1) {
             const pulse = (Math.sin(Date.now() / 150) + 1) / 2;
             ctx.beginPath();
             ctx.arc(node.x ?? 0, node.y ?? 0, radius + (pulse * 3), 0, 2 * Math.PI, false);
             ctx.strokeStyle = `rgba(34, 211, 238, ${0.8 - pulse})`;
             ctx.lineWidth = 1 / global_scale;
             ctx.stroke();
        }

        // 4. Labels (Optimized for visibility)
        if (global_scale > 2) {
            ctx.font = `${font_size}px Inter, system-ui`;
            ctx.textAlign = 'center';
            ctx.textBaseline = 'top';
            ctx.fillStyle = 'white';
            ctx.fillText(label, node.x ?? 0, (node.y ?? 0) + radius + 4);
        }
        
        // 5. Telemetry Bars (Battery/Signal)
        const bar_w = 12 / global_scale;
        const bar_h = 2 / global_scale;
        if (global_scale > 1.5) {
            ctx.fillStyle = 'rgba(255, 255, 255, 0.1)';
            ctx.fillRect((node.x ?? 0) - bar_w/2, (node.y ?? 0) - radius - 5, bar_w, bar_h);
            ctx.fillStyle = node.battery > 20 ? '#10b981' : '#f43f5e';
            ctx.fillRect((node.x ?? 0) - bar_w/2, (node.y ?? 0) - radius - 5, bar_w * (node.battery / 100), bar_h);
        }
    }, []);

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
                linkColor={() => 'rgba(255, 255, 255, 0.05)'}
                linkWidth={1}
                linkDirectionalParticles={2}
                linkDirectionalParticleSpeed={0.005}
                linkDirectionalParticleColor={() => COLORS.BUSY}
                backgroundColor="rgba(0,0,0,0)"
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
                        <h2 className="text-xs font-black text-white uppercase tracking-[0.4em]">Swarm Pulse</h2>
                    </div>
                    <div className="flex items-center gap-4 ml-7">
                        <p className="text-[9px] text-zinc-500 font-bold uppercase tracking-wider">10Hz Telemetry</p>
                        <div className="w-px h-2 bg-zinc-800" />
                        <p className="text-[9px] text-zinc-500 font-bold uppercase tracking-wider">{graph_data.nodes.length} Nodes Online</p>
                    </div>
                </div>
            </div>

            {/* Bottom Controls Panel */}
            <div className="absolute bottom-8 right-8 flex gap-2">
                 <button 
                    onClick={() => fg_ref.current?.zoomToFit(400, 50)}
                    className="px-4 py-2 bg-zinc-900/40 backdrop-blur-md border border-zinc-800 rounded-xl text-[9px] font-bold text-zinc-400 uppercase tracking-widest hover:bg-zinc-800 hover:text-white transition-all shadow-lg overflow-hidden group/btn"
                 >
                    <span className="relative z-10">Recenter Swarm</span>
                 </button>
            </div>
        </div>
    );
};

