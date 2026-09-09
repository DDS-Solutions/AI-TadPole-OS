/**
 * @docs ARCHITECTURE:Pages
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Pages / Detached_Swarm_Pulse
 * - **Primary Entrypoints**: `Detached_Swarm_Pulse`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
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
        <div className="w-screen h-screen bg-[color:var(--color-background)] p-4">
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
