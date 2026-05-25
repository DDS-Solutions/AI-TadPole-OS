/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Registry view for all discovered MCP tools. 
 * Orchestrates categorized tool rendering and facilitates rapid capability discovery for swarm operators.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Tool category mismatch, or empty registry flash during background re-scan.
 * - **Telemetry Link**: Search for `[Mcp_Tool_List]` or `tool_registry_update` in service logs.
 */

import React from 'react';
import { Terminal, Plus } from 'lucide-react';
import { type Mcp_Tool_Hub_Definition } from '../../stores/skill_store';
import { Mcp_Tool_Card } from './Mcp_Tool_Card';
import { Tw_Empty_State } from '../ui';
import { i18n } from '../../i18n';

interface Mcp_Tool_List_Props {
    tools: Mcp_Tool_Hub_Definition[];
    on_edit: (tool: Mcp_Tool_Hub_Definition) => void;
}

export const Mcp_Tool_List: React.FC<Mcp_Tool_List_Props> = ({ tools, on_edit }) => {
    if (!tools?.length) {
        return (
            <div className="py-20 animate-in fade-in zoom-in-95 duration-500">
                <Tw_Empty_State
                    icon={<Terminal size={32} className="text-zinc-700" />}
                    title={i18n.t('skills.mcp_empty_title')}
                    description={i18n.t('skills.mcp_empty_description')}
                    action={
                        <button
                            className="inline-flex items-center gap-2 px-4 py-2 bg-green-600 hover:bg-green-500 text-white rounded-lg text-sm font-bold transition-all shadow-lg shadow-green-500/20"
                        >
                            <Plus size={16} />
                            {i18n.t('skills.discover_mcp_plugins')}
                        </button>
                    }
                />
            </div>
        );
    }

    return (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 animate-in fade-in slide-in-from-bottom-5">
            {tools?.map((tool) => (
                <Mcp_Tool_Card
                    key={tool.name}
                    tool={tool}
                    on_edit={on_edit}
                />
            ))}
        </div>
    );
};


// Metadata: [Mcp_Tool_List]
