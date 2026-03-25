import React from 'react';
import { FlaskConical } from 'lucide-react';
import { i18n } from '../../i18n';
import type { McpToolHubDefinition } from '../../stores/skillStore';
import { McpToolCard } from './McpToolCard';

interface McpToolListProps {
    mcpTools: McpToolHubDefinition[];
    searchQuery: string;
    activeCategory: 'user' | 'ai';
    onAssignTool: (name: string) => void;
    onTestTool: (tool: McpToolHubDefinition) => void;
}

export const McpToolList: React.FC<McpToolListProps> = ({
    mcpTools,
    searchQuery,
    activeCategory,
    onAssignTool,
    onTestTool
}) => {
    const filteredTools = mcpTools.filter(t => 
        t.category === activeCategory && 
        (t.name.toLowerCase().includes(searchQuery.toLowerCase()) || 
         t.description?.toLowerCase().includes(searchQuery.toLowerCase()))
    );

    return (
        <div className="space-y-6">
            <div className="flex justify-between items-center mb-6 sticky top-0 bg-zinc-950/80 backdrop-blur-md pt-2 pb-3 border-b border-zinc-800/50 z-20">
                <h2 className="text-xl font-bold text-zinc-100 tracking-tight flex items-center gap-3">
                    <FlaskConical className="text-cyan-500" size={20} /> {i18n.t('skills.header_lab')}
                </h2>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                {filteredTools.map(tool => (
                    <McpToolCard 
                        key={tool.name} 
                        tool={tool} 
                        onAssign={onAssignTool} 
                        onTest={onTestTool} 
                    />
                ))}
            </div>
        </div>
    );
};
