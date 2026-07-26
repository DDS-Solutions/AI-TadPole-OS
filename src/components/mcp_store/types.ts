/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **@docs ARCHITECTURE:Types**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[types]` in observability traces.
 */

/**
 * @docs ARCHITECTURE:Types
 */

export interface MCP_Connector {
    id: string;
    name: string;
    description: string;
    category: string;
    path: string;
    version: string;
    author: string;
    installed?: boolean;
}

// Metadata: [types]
