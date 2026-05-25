/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[types]` in observability traces.
 */

export interface SymbolNode {
    name: string;
    path: string;
    kind: string;
    signature: string;
    start_line: number;
    end_line: number;
}

export interface ExtendedGraphNode extends SymbolNode {
    id: string;
    is_affected: boolean;
    is_path_highlighted?: boolean;
    x?: number;
    y?: number;
    vx?: number;
    vy?: number;
    fx?: number;
    fy?: number;
}

export interface ForceGraphLink {
    source: string | any; 
    target: string | any;
    is_path_highlighted?: boolean;
}

// Metadata: [types]
