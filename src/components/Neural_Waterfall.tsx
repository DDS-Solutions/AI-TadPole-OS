/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **@docs ARCHITECTURE:Interface**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Neural_Waterfall]` in observability traces.
 */

/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Assist Note
 * **Stable Barrel**: Re-exports Neural_Waterfall from its new location under
 * `intelligence/waterfall/`. All consumers keep `import { Neural_Waterfall } from '../components/Neural_Waterfall'`.
 *
 * Implementation lives in `src/components/intelligence/waterfall/` — edit there.
 */

export { Neural_Waterfall } from './intelligence/waterfall/Neural_Waterfall';

// Metadata: [Neural_Waterfall]
