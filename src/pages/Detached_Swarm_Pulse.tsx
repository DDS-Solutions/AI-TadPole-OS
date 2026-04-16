/**
 * @docs ARCHITECTURE:Pages
 * 
 * ### AI Assist Note
 * **Detached Swarm Pulse**: Standalone telemetry visualizer optimized for 
 * multi-window operations. Decouples the pulse graph from the main dashboard 
 * to allow dedicated monitor tracking of swarm health. Inherits 
 * `Swarm_Visualizer` logic with the `is_detached` flag to enable 
 * fullscreen vertex rendering and dedicated WebSocket synchronization.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: WebSocket disconnection causing empty graph frames, 
 *   Canvas context loss on window resize, or state lag in multi-agent bursts.
 * - **Telemetry Link**: Look for `[Pulse:Detached]` in tracing logs.
 * - **Trace Scope**: `src/pages/Detached_Swarm_Pulse`
 */

import { Swarm_Visualizer } from '../components/Swarm_Visualizer';


/**
 * Detached_Swarm_Pulse
 * A standalone view for the Swarm Pulse telemetry graph, 
 * optimized for multi-window setups.
 */
export default function Detached_Swarm_Pulse() {
    return (
        <div className="w-screen h-screen bg-zinc-950 p-4">
            <div className="w-full h-full">
                <Swarm_Visualizer is_detached={true} />
            </div>
        </div>
    );
}
