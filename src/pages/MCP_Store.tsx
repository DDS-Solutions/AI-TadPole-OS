/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Pages / MCP_Store
 * - **Primary Entrypoints**: `MCP_Store`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { useEffect, useState } from 'react';
import { Template_Store_Header } from '../components/template_store';
import { fetchMCPRegistry } from '../components/mcp_store/mcp_store_api';
import { MCP_Card } from '../components/mcp_store/MCP_Card';
import type { MCP_Connector } from '../components/mcp_store/types';

function MCP_Store() {
    const [connectors, setConnectors] = useState<MCP_Connector[]>([]);
    const [isLoading, setIsLoading] = useState(true);

    useEffect(() => {
        let mounted = true;
        
        fetchMCPRegistry().then(data => {
            if (mounted) {
                setConnectors(data);
                setIsLoading(false);
            }
        });

        return () => { mounted = false; };
    }, []);

    return (
        <div className="p-6 space-y-6 max-w-7xl mx-auto w-full">
            <Template_Store_Header />
            
            {isLoading ? (
                <div className="flex items-center justify-center py-24">
                    <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-emerald-500/50 border-t-emerald-500"></div>
                </div>
            ) : (
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                    {connectors.map(c => (
                        <MCP_Card key={c.id} connector={c} />
                    ))}
                </div>
            )}
        </div>
    );
}

export default MCP_Store;
