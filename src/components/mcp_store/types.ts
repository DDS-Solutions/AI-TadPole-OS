/**
 * @docs ARCHITECTURE:UI-Components
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Mcp_Store / types
 * - **Primary Entrypoints**: `MCP_Connector`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
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
