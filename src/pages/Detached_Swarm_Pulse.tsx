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
import { i18n } from '../i18n';


/**
 * Detached_Swarm_Pulse
 * A standalone view for the Swarm Pulse telemetry graph, 
 * optimized for multi-window setups.
 */
export default function Detached_Swarm_Pulse() {
    return (
        <div className="w-screen h-screen bg-zinc-950 p-4">
            {/* GEO Optimization: Structured Data & Semantic Header */}
            <script type="application/ld+json">
            {JSON.stringify({
              "@context": "https://schema.org",
              "@type": "SoftwareApplication",
              "name": "Tadpole OS Swarm Pulse",
              "description": i18n.t('swarm_visualizer.detached_description'),
              "author": { "@type": "Organization", "name": "Sovereign Engineering" },
              "applicationCategory": "Telemetry Tool",
              "operatingSystem": "Tadpole OS"
            })}
            </script>
            <h1 className="sr-only">{i18n.t('swarm_visualizer.detached_telemetry_h1')}</h1>
            <div className="w-full h-full">
                <Swarm_Visualizer is_detached={true} />
            </div>
        </div>
    );
}

// Metadata: [Detached_Swarm_Pulse]

// Metadata: [Detached_Swarm_Pulse]
