/**
 * @docs ARCHITECTURE:UI-Components
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Template_Store / mcp_placeholders
 * - **Primary Entrypoints**: `extractMcpPlaceholders`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Pure helper function parsing MCP server env blocks into placeholder descriptor objects.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import type { McpPlaceholderVariable } from './types';

export function extractMcpPlaceholders(
    mcps: Record<string, unknown> | null | undefined
): McpPlaceholderVariable[] {
    if (!mcps || typeof mcps !== 'object') return [];

    const placeholders: McpPlaceholderVariable[] = [];
    const seen = new Set<string>();
    const servers = (mcps.mcp_servers || mcps.mcpServers || mcps) as Record<string, unknown>;

    if (typeof servers !== 'object' || servers === null) return [];

    for (const [serverName, serverVal] of Object.entries(servers)) {
        if (!serverVal || typeof serverVal !== 'object') continue;
        const serverObj = serverVal as Record<string, unknown>;
        const env = serverObj.env as Record<string, unknown> | undefined;

        if (env && typeof env === 'object') {
            for (const envVal of Object.values(env)) {
                if (typeof envVal === 'string') {
                    const matches = envVal.match(/\$\{([A-Z0-9_]+)\}/g);
                    if (matches) {
                        for (const match of matches) {
                            const varName = match.slice(2, -1);
                            const key = `${serverName}::${varName}`;
                            if (!seen.has(key)) {
                                seen.add(key);
                                placeholders.push({
                                    server: serverName,
                                    variable: varName,
                                    description: `Required API key or token for ${serverName}`
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    return placeholders;
}
