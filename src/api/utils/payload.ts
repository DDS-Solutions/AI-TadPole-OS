/**
 * @docs ARCHITECTURE:Core
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[payload]` in observability traces.
 */

export function truncate_payload(data: string, max_length = 1024): string {
    if (data.length <= max_length) {
        return data;
    }
    return `${data.substring(0, max_length)}... [TRUNCATED ${data.length} bytes]`;
}

// Metadata: [payload]
