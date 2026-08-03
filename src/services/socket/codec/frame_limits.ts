/**
 * @docs ARCHITECTURE:UI-Services
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[frame_limits]` in observability traces.
 */

export const MAX_BINARY_FRAME_SIZE = 1 * 1024 * 1024; // 1MB
export const MAX_TEXT_FRAME_SIZE = 5 * 1024 * 1024; // 5MB

// Metadata: [frame_limits]
