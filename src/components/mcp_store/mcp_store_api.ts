/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **@docs ARCHITECTURE:UI-Services**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[mcp_store_api]` in observability traces.
 */

/**
 * @docs ARCHITECTURE:UI-Services
 */

import type { MCP_Connector } from './types';

export const MCP_REGISTRY_RAW = 'https://dds-solutions.github.io/AI-Tadpole-OS-Industry-Templates/mcp_registry.json';

export async function fetchMCPRegistry(): Promise<MCP_Connector[]> {
    try {
        const res = await fetch(MCP_REGISTRY_RAW);
        if (!res.ok) throw new Error('Failed to load MCP Registry');
        const data = await res.json();
        return (data.connectors || []) as MCP_Connector[];
    } catch (err) {
        console.debug('[MCPStore] Failed to fetch MCP registry, using fallback data:', err);
        return [
            {
                id: "mcp-generic-crm",
                name: "Generic REST & Webhook CRM Blueprint",
                description: "Template MCP server for querying REST endpoints and receiving webhooks from SMB SaaS platforms like HubSpot, Salesforce, etc.",
                category: "CRM & SaaS",
                path: "mcp-blueprints/generic-crm",
                version: "1.0.0",
                author: "Tadpole OS Core"
            },
            {
                id: "mcp-smb-accounting",
                name: "SMB Accounting Blueprint",
                description: "Connect to lightweight SMB accounting platforms like QuickBooks and Xero. Mirrors invoicing workflows.",
                category: "Accounting",
                path: "mcp-blueprints/smb-accounting",
                version: "1.0.0",
                author: "Tadpole OS Core"
            },
            {
                id: "mcp-local-log-scanner",
                name: "Local Log Scanner & File Parser",
                description: "Parses structured logs and local flat files for continuous state observation.",
                category: "System",
                path: "mcp-blueprints/log-scanner",
                version: "1.1.2",
                author: "Community"
            }
        ];
    }
}

// Metadata: [mcp_store_api]
