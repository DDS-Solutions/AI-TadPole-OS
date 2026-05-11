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
import { tadpole_os_socket, type Tadpole_OS_Socket_Client } from '../services/socket';
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
    vy?: number;
    // Logic/Reasoning specific fields
    slot?: string;
    model?: string;
}

interface GraphLink {
    source: string | GraphNode;
    target: string | GraphNode;
}

interface ExtendedForceGraphMethods extends ForceGraphMethods<GraphNode, GraphLink> {
    d3Simulation?(): { alphaTarget(target: number): void };
    refresh?(): void;
}

export const Swarm_Visualizer: React.FC<{ is_detached?: boolean, on_detach?: () => void }> = ({ is_detached = false, on_detach }) => {
    const fg_ref = useRef<ForceGraphMethods<GraphNode, GraphLink> | undefined>(undefined);
    const [view_mode, set_view_mode] = React.useState<'swarm' | 'logic'>('swarm');
    const [graph_data, set_graph_data] = React.useState<{ nodes: GraphNode[], links: GraphLink[] }>({ nodes: [], links: [] });
    const [logic_data, set_logic_data] = React.useState<{ nodes: GraphNode[], links: GraphLink[] }>({ nodes: [], links: [] });
    
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
                        const methods = (fg_ref.current as unknown as ExtendedForceGraphMethods);
                        methods?.d3Simulation?.()?.alphaTarget(0.1);
                        setTimeout(() => methods?.d3Simulation?.()?.alphaTarget(0), 1000);
                    }, 0);
                    return { nodes: new_nodes, links: new_links };
                } else {
                    // Return same object but refresh canvas
                    (fg_ref.current as unknown as ExtendedForceGraphMethods)?.refresh?.();
                    return prev;
                }
            });
        });

        const unsubscribe_reasoning = (tadpole_os_socket as Tadpole_OS_Socket_Client).subscribe_raw((raw_event: Record<string, unknown>) => {
            const event = raw_event as unknown as { type: string, agent_id: string, step: { id: string, content: string, model: string, slot: string, parent_id?: string } };
            if (event.type === 'agent:reasoning_step') {
                const step = event.step;
                set_logic_data(prev => {
                    const node_exists = prev.nodes.some(n => n.id === step.id);
                    if (node_exists) return prev;

                    const new_node: GraphNode = {
                        id: step.id,
                        name: step.content.substring(0, 50) + '...',
                        status: SWARM_NODE_STATUS.IDLE,
                        battery: 100,
                        signal: 100,
                        progress: 0,
                        model: step.model,
                        slot: step.slot,
                        x: (Math.random() - 0.5) * 100,
                        y: (Math.random() - 0.5) * 100,
                    };

                    const new_link = step.parent_id ? {
                        source: step.parent_id,
                        target: step.id
                    } : null;

                    return {
                        nodes: [...prev.nodes, new_node],
                        links: new_link ? [...prev.links, new_link] : prev.links
                    };
                });
            }
        });

        return () => {
            unsubscribe();
            unsubscribe_reasoning();
        };
    }, []);

    // ### 🎨 High-Performance Rendering: Fast-Path Canvas Pipeline
    const logic_node_canvas_object = useMemo(() => (node: GraphNode, ctx: CanvasRenderingContext2D, global_scale: number) => {
        ctx.save();
        const radius = GRAPH_THEME.NODE_RADIUS * 0.8;
        const color = node.slot === 'planning' ? '#14B8A6' : '#22D3EE';
        
        // Glow for active slot
        ctx.beginPath();
        ctx.arc(node.x ?? 0, node.y ?? 0, radius * 1.5, 0, 2 * Math.PI);
        ctx.fillStyle = node.slot === 'planning' ? 'rgba(20, 184, 166, 0.1)' : 'rgba(34, 211, 238, 0.1)';
        ctx.fill();

        ctx.beginPath();
        ctx.arc(node.x ?? 0, node.y ?? 0, radius, 0, 2 * Math.PI);
        ctx.fillStyle = color;
        ctx.fill();

        if (global_scale > 1.2) {
            ctx.font = `bold ${9 / global_scale}px Inter`;
            ctx.textAlign = 'center';
            ctx.fillStyle = 'white';
            ctx.fillText(node.model || 'Unknown', node.x ?? 0, (node.y ?? 0) - radius - 5);
            
            ctx.font = `${10 / global_scale}px Inter`;
            ctx.fillStyle = 'rgba(255,255,255,0.6)';
            const safe_label = node.name.length > 30 ? node.name.substring(0, 27) + '...' : node.name;
            ctx.fillText(safe_label, node.x ?? 0, (node.y ?? 0) + radius + 12);
        }
        ctx.restore();
    }, []);

    // --- 🧬 Swarm Node Renderer (Premium Aesthetics) ---
    const swarm_node_canvas_object = useMemo(() => (node: GraphNode, ctx: CanvasRenderingContext2D, global_scale: number) => {
        const radius = GRAPH_THEME.NODE_RADIUS;
        const is_hub = node.status === SWARM_NODE_STATUS.MISSION_HUB;
        
        ctx.save();
        
        // 1. Halo / Glow
        const gradient = ctx.createRadialGradient(node.x ?? 0, node.y ?? 0, radius, node.x ?? 0, node.y ?? 0, radius * 3);
        gradient.addColorStop(0, is_hub ? 'rgba(236, 72, 153, 0.2)' : 'rgba(34, 211, 238, 0.2)');
        gradient.addColorStop(1, 'rgba(0, 0, 0, 0)');
        ctx.fillStyle = gradient;
        ctx.beginPath();
        ctx.arc(node.x ?? 0, node.y ?? 0, radius * 3, 0, 2 * Math.PI);
        ctx.fill();

        // 2. Main Body
        ctx.beginPath();
        ctx.arc(node.x ?? 0, node.y ?? 0, radius, 0, 2 * Math.PI);
        ctx.fillStyle = is_hub ? '#EC4899' : '#22D3EE';
        ctx.fill();
        
        // 3. Status Ring (Battery/Health)
        ctx.beginPath();
        ctx.arc(node.x ?? 0, node.y ?? 0, radius + 2, 0, 2 * Math.PI);
        ctx.strokeStyle = `rgba(255, 255, 255, ${0.1 + (node.battery / 100) * 0.3})`;
        ctx.lineWidth = 1;
        ctx.stroke();

        // 4. Labeling
        if (global_scale > 1.0) {
            const label = node.name || node.id;
            const fontSize = 12 / global_scale;
            ctx.font = `${fontSize}px Inter`;
            ctx.textAlign = 'center';
            ctx.fillStyle = 'white';
            ctx.fillText(label, node.x ?? 0, (node.y ?? 0) + radius + fontSize + 5);
        }
        
        ctx.restore();
    }, []);

    const current_node_count = view_mode === 'swarm' ? graph_data.nodes.length : logic_data.nodes.length;

    return (
        <div className="w-full h-full relative bg-zinc-950 rounded-[2.5rem] border border-zinc-900/50 shadow-2xl overflow-hidden group">
            {/* View Mode Toggle */}
            <div className="absolute top-8 left-1/2 -translate-x-1/2 flex gap-1 p-1 bg-zinc-900/60 backdrop-blur-xl border border-zinc-800/50 rounded-2xl shadow-2xl z-50 pointer-events-auto">
                <button 
                    onClick={() => set_view_mode('swarm')}
                    className={`px-5 py-2 rounded-xl text-[10px] font-black uppercase tracking-[0.2em] transition-all duration-300 ${view_mode === 'swarm' ? 'bg-cyan-500 text-white shadow-[0_0_20px_rgba(6,182,212,0.4)]' : 'text-zinc-500 hover:text-zinc-300'}`}
                >
                    Swarm
                </button>
                <button 
                    onClick={() => set_view_mode('logic')}
                    className={`px-5 py-2 rounded-xl text-[10px] font-black uppercase tracking-[0.2em] transition-all duration-300 ${view_mode === 'logic' ? 'bg-emerald-500 text-white shadow-[0_0_20px_rgba(16,185,129,0.4)]' : 'text-zinc-500 hover:text-zinc-300'}`}
                >
                    Reasoning Trace
                </button>
            </div>

            <ForceGraph2D
                ref={fg_ref}
                graphData={view_mode === 'swarm' ? graph_data : logic_data}
                nodeCanvasObject={view_mode === 'swarm' ? swarm_node_canvas_object : logic_node_canvas_object}
                nodePointerAreaPaint={(node: GraphNode, color, ctx) => {
                    ctx.fillStyle = color;
                    ctx.beginPath(); ctx.arc(node.x ?? 0, node.y ?? 0, 10, 0, 2 * Math.PI, false); ctx.fill();
                }}
                linkColor={() => view_mode === 'swarm' ? THEME_COLORS.NEURAL_GRID : 'rgba(16, 185, 129, 0.2)'}
                linkWidth={view_mode === 'swarm' ? GRAPH_THEME.LINK_WIDTH : 1.5}
                linkDirectionalParticles={view_mode === 'swarm' ? 2 : 4}
                linkDirectionalParticleWidth={view_mode === 'swarm' ? 2 : 3}
                linkDirectionalParticleColor={() => view_mode === 'swarm' ? '#22D3EE' : '#10B981'}
                linkDirectionalParticleSpeed={GRAPH_THEME.PARTICLE_SPEED}
                backgroundColor="rgba(0,0,0,0)"
                d3AlphaDecay={0.02}
                d3VelocityDecay={0.3}
                cooldownTicks={100}
                onNodeClick={(node: GraphNode) => {
                    if (view_mode === 'swarm') {
                        if (node.status === SWARM_NODE_STATUS.MISSION_HUB) return;
                        set_selected_agent_id(node.id);
                        set_scope('agent');
                        set_target_agent(node.name || node.id);
                    }
                    
                    if (node.x !== undefined && node.y !== undefined) {
                        fg_ref.current?.centerAt?.(node.x, node.y, 400);
                        fg_ref.current?.zoom?.(view_mode === 'swarm' ? 2.5 : 3.5, 400);
                    }
                }}
            />
            
            {/* HUD Overlay */}
            <div className="absolute top-8 left-8 pointer-events-none select-none">
                 <div className="flex flex-col gap-1.5">
                    <div className="flex items-center gap-3">
                        <div className={`w-2.5 h-2.5 rounded-full animate-pulse ${view_mode === 'swarm' ? 'bg-cyan-500 shadow-[0_0_15px_#06b6d4]' : 'bg-emerald-500 shadow-[0_0_15px_#10b981]'}`} />
                        <h2 className="text-xs font-black text-white uppercase tracking-[0.4em]">
                            {view_mode === 'swarm' ? i18n.t('swarm_visualizer.title') : 'Reasoning Trace v1.0'}
                        </h2>
                    </div>
                    <div className="flex items-center gap-4 ml-7">
                        <p className="text-[9px] text-zinc-500 font-bold uppercase tracking-wider">{i18n.t('swarm_visualizer.telemetry_10hz')}</p>
                        <div className="w-px h-2 bg-zinc-800" />
                        <p className="text-[9px] text-zinc-500 font-bold uppercase tracking-wider">
                            {view_mode === 'swarm' ? i18n.t('swarm_visualizer.nodes_online', { count: current_node_count }) : `${current_node_count} Reasoning Branches`}
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
                    onClick={() => fg_ref.current?.zoomToFit?.(400, 50)}
                    className="px-4 py-2 bg-zinc-900/40 backdrop-blur-md border border-zinc-800 rounded-xl text-[9px] font-bold text-zinc-400 uppercase tracking-widest hover:bg-zinc-800 hover:text-white transition-all shadow-lg overflow-hidden group/btn"
                 >
                    <span className="relative z-10">{i18n.t('swarm_visualizer.recenter_swarm')}</span>
                 </button>
            </div>
        </div>
    );
};
