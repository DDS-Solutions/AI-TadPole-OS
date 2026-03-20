import { useState, useEffect, useMemo } from 'react';
import { Users, Search, Filter, Cpu, Sliders, Shield, Activity, Zap, UserPlus, Info, DollarSign } from 'lucide-react';
import { Tooltip } from '../components/ui';
import { i18n } from '../i18n';
import { useAgentStore } from '../stores/agentStore';
import type { Agent, Department } from '../types';
import AgentConfigPanel from '../components/AgentConfigPanel';
import { useRoleStore } from '../stores/roleStore';
import { getSettings } from '../stores/settingsStore';

/**
 * @page AgentManager
 * Interface for managing the agent swarm.
 * 
 * FEATURES:
 * - Agent Grid: Visual overview of all agents.
 * - Filtering: Search by name/role and filter by role type.
 * - Configuration: Access to AgentConfigPanel for deep editing.
 * - State Sync: Real-time UI updates reflecting active/paused statuses.
 */
export default function AgentManager() {
    const { agents, fetchAgents, updateAgent, addAgent } = useAgentStore();
    const [selectedAgent, setSelectedAgent] = useState<Agent | null>(null);
    const [isCreating, setIsCreating] = useState(false);
    const [searchQuery, setSearchQuery] = useState('');
    const [filterRole, setFilterRole] = useState<string>('all');

    useEffect(() => {
        if (agents.length === 0) {
            fetchAgents();
        }
    }, [agents.length, fetchAgents]);

    /**
     * Synchronizes agent state with the parent store to maintain panel feedback.
     * If in 'creation' mode, invokes the persistent registration flow.
     * 
     * @param id - The unique identifier of the target node.
     * @param updates - Incremental state changes to apply.
     */
    const handleUpdateAgent = (id: string, updates: Partial<Agent>) => {
        if (isCreating) {
            // If we're creating, we use addAgent instead of update
            addAgent({ ...selectedAgent!, ...updates } as Agent);
            setIsCreating(false);
            setSelectedAgent(null);
        } else {
            updateAgent(id, updates);
            // Update selection if needed to maintain panel feedback
            if (selectedAgent && selectedAgent.id === id) {
                setSelectedAgent(prev => prev ? { ...prev, ...updates } : null);
            }
        }
    };

    /**
     * Initializes a new agent template and opens the configuration panel.
     * Enforces the hard-coded 25-node capacity limit per sector.
     * 
     * @throws Alert if sector capacity is exceeded.
     */
    const handleAddNewClick = () => {
        if (agents.length >= 25) {
            // Notification is handled via generic browser alert for now, 
            // but we can use EventBus if we want more flair.
            alert(i18n.t('agent_manager.error_limit_reached'));
            return;
        }

        const newAgent: Agent = {
            id: `node_${Math.random().toString(36).substr(2, 9)} `,
            name: i18n.t('agent_manager.placeholder_name'),
            role: i18n.t('agent_manager.placeholder_role'),
            status: "idle",
            model: getSettings().defaultModel,
            capabilities: [],
            workflows: [],
            tokensUsed: 0,
            costUsd: 0,
            budgetUsd: 1.0,
            department: i18n.t('agent_manager.default_dept') as Department,
            themeColor: "#10b981",
            modelConfig: {
                modelId: getSettings().defaultModel,
                provider: "anthropic", // The config panel will handle provider matching based on modelId
                temperature: getSettings().defaultTemperature,
                systemPrompt: "",
                skills: [],
                workflows: []
            }
        };

        setIsCreating(true);
        setSelectedAgent(newAgent);
    };

    const filteredAgents = agents.filter(agent => {
        const matchesSearch = agent.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
            agent.role.toLowerCase().includes(searchQuery.toLowerCase());
        const matchesRole = filterRole === 'all' || agent.role === filterRole;
        return matchesSearch && matchesRole;
    });

    const roles = useRoleStore(s => s.roles);
    const allRoleNames = useMemo(() => Object.keys(roles).sort(), [roles]);

    const filterRoles = useMemo(() => Array.from(new Set([
        ...allRoleNames,
        ...agents.map(a => a.role)
    ])).sort(), [allRoleNames, agents]);

    return (
        <div className="h-full flex flex-col bg-zinc-950">
            {/* Standard Sticky Header */}
            <div className="py-4 px-6 border-b border-zinc-900 bg-zinc-950/80 backdrop-blur-md sticky top-0 z-40 flex items-center justify-between">
                <div>
                    <h2 className="text-xl font-bold text-zinc-100 flex items-center gap-2 uppercase tracking-tight">
                        <Users className="text-emerald-500" /> {i18n.t('agent_manager.title')}
                    </h2>
                    <div className="mt-1">
                    </div>
                </div>
                <div className="flex items-center gap-4">
                    <div className="flex flex-col items-center gap-1.5">
                        <Tooltip content={i18n.t('agent_manager.tooltip_add')} position="bottom">
                            <button
                                onClick={handleAddNewClick}
                                className="flex items-center gap-2 px-4 py-2 bg-emerald-600 hover:bg-emerald-500 text-white font-bold rounded-lg text-xs transition-all shadow-[0_0_15px_rgba(16,185,129,0.2)] hover:shadow-[0_0_20px_rgba(16,185,129,0.4)]"
                            >
                                <UserPlus size={14} />
                                {i18n.t('agent_manager.btn_add')}
                            </button>
                        </Tooltip>
                        <div className="hidden">
                            <Tooltip content={i18n.t('agent_manager.tooltip_capacity')} position="bottom">
                                <div className="flex items-center gap-1.5 opacity-80 cursor-help">
                                    <Info size={10} className="text-blue-400" />
                                    <span className="text-[10px] text-zinc-500 font-bold uppercase tracking-tighter">{i18n.t('agent_manager.label_max_capacity', { current: agents.length })}</span>
                                </div>
                            </Tooltip>
                        </div>
                    </div>

                    <div className="h-6 w-px bg-zinc-800 mx-1" />

                    {/* Role Filter */}
                    <div className="relative">
                                <Tooltip content={i18n.t('agent_manager.tooltip_filter')} position="bottom">
                                    <div className="relative">
                                        <Filter className="absolute left-3 top-1/2 -translate-y-1/2 text-zinc-500" size={14} />
                                        <select
                                            value={filterRole}
                                            onChange={(e) => setFilterRole(e.target.value)}
                                            className="bg-zinc-900 border border-zinc-800 text-zinc-200 text-xs rounded-lg pl-9 pr-8 py-2 appearance-none focus:outline-none focus:border-emerald-500/50 cursor-pointer uppercase font-bold tracking-wider"
                                        >
                                            <option value="all">{i18n.t('agent_manager.filter_all')}</option>
                                    {filterRoles.map(role => (
                                        <option key={role} value={role}>{role}</option>
                                    ))}
                                </select>
                                <div className="absolute right-3 top-1/2 -translate-y-1/2 pointer-events-none text-zinc-500">
                                    <svg width="10" height="6" viewBox="0 0 10 6" fill="currentColor"><path d="M0 0.5L5 5.5L10 0.5H0Z" /></svg>
                                </div>
                            </div>
                        </Tooltip>
                    </div>

                    <div className="relative">
                        <Tooltip content={i18n.t('agent_manager.tooltip_search')} position="bottom">
                            <div className="relative">
                                <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-zinc-500" size={14} />
                                <input
                                    type="text"
                                    placeholder={i18n.t('agent_manager.search_placeholder')}
                                    value={searchQuery}
                                    onChange={(e) => setSearchQuery(e.target.value)}
                                    className="bg-zinc-900 border border-zinc-800 text-zinc-200 text-xs rounded-lg pl-9 pr-3 py-2 w-64 focus:outline-none focus:border-emerald-500/50 transition-colors placeholder:text-zinc-600"
                                />
                            </div>
                        </Tooltip>
                    </div>
                </div>
            </div>

            <div className="flex-1 overflow-y-auto custom-scrollbar p-6">
                <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
                    {filteredAgents.map(agent => (
                        <div
                            key={agent.id}
                            onClick={() => setSelectedAgent(agent)}
                            className="group bg-zinc-900 border border-zinc-800 hover:border-emerald-500/30 rounded-xl p-4 cursor-pointer transition-all hover:shadow-lg hover:shadow-emerald-900/5 relative overflow-hidden"
                        >
                            <div className="absolute top-0 right-0 p-3 opacity-0 group-hover:opacity-100 transition-opacity">
                                <Tooltip content={i18n.t('agent_manager.tooltip_configure')} position="left">
                                    <Sliders size={16} className="text-emerald-500" />
                                </Tooltip>
                            </div>

                            <div className="flex items-start gap-4">
                                <div
                                    className="w-10 h-10 rounded-lg flex items-center justify-center font-bold text-lg shrink-0"
                                    style={{
                                        backgroundColor: agent.status === 'active' ? `${agent.themeColor || '#10b981'}15` : '#27272a',
                                        color: agent.status === 'active' ? (agent.themeColor || '#10b981') : '#71717a',
                                        border: `1px solid ${agent.status === 'active' ? `${agent.themeColor || '#10b981'}30` : '#3f3f46'}`
                                    }}
                                >
                                    {(agent.name || '?')[0]}
                                </div>
                                <div className="flex-1 min-w-0">
                                    <div className="flex items-center gap-2 mb-1">
                                        <h3
                                            className="text-sm font-bold text-zinc-200 truncate group-hover:text-emerald-400 transition-colors"
                                            style={agent.status === 'active' ? { transition: 'color 0.2s' } : {}}
                                        >
                                            {agent.name}
                                        </h3>
                                        {agent.status === 'active' && (
                                            <Tooltip content={i18n.t('agent_manager.tooltip_active')} position="top">
                                                <span className="flex items-center gap-1 text-[9px] font-bold text-emerald-500 uppercase tracking-wider bg-emerald-500/10 px-1.5 py-0.5 rounded cursor-help">
                                                    <Activity size={8} /> {i18n.t('agent_manager.badge_active')}
                                                </span>
                                            </Tooltip>
                                        )}
                                    </div>
                                    <Tooltip content={i18n.t('agent_manager.tooltip_role')} position="top">
                                        <p className="text-[10px] text-zinc-500 font-mono truncate uppercase flex items-center gap-1.5 cursor-help">
                                            <Shield size={10} /> {agent.role}
                                        </p>
                                    </Tooltip>

                                    <div className="mt-4 flex flex-wrap gap-2">
                                        <Tooltip content={`LLM: ${agent.model} `} position="top">
                                            <div className="bg-zinc-900 border border-zinc-800 rounded px-2 py-1 flex items-center gap-1.5 cursor-help">
                                                <Cpu size={10} className="text-blue-400" />
                                                <span className="text-[10px] text-zinc-400 font-mono">{agent.model.split(' ').pop()}</span>
                                            </div>
                                        </Tooltip>
                                        <Tooltip content={i18n.t('agent_manager.tooltip_temp')} position="top">
                                            <div className="bg-zinc-900 border border-zinc-800 rounded px-2 py-1 flex items-center gap-1.5 cursor-help">
                                                <Zap size={10} className="text-amber-400" />
                                                <span className="text-[10px] text-zinc-400 font-mono">{(agent.modelConfig?.temperature || 0.7).toFixed(1)} TEMP</span>
                                            </div>
                                        </Tooltip>
                                        <Tooltip content={i18n.t('agent_manager.tooltip_credits')} position="top">
                                            <div className="bg-zinc-900 border border-zinc-800 rounded px-2 py-1 flex items-center gap-1.5 cursor-help">
                                                <DollarSign size={10} className="text-emerald-400" />
                                                <span className="text-[10px] text-zinc-400 font-mono">
                                                    ${(agent.costUsd || 0).toFixed(3)} / ${agent.budgetUsd || '0'}
                                                </span>
                                            </div>
                                        </Tooltip>
                                    </div>
                                </div>
                            </div>
                        </div>
                    ))}
                </div>
            </div>

            {selectedAgent && (
                <AgentConfigPanel
                    agent={selectedAgent}
                    onClose={() => {
                        setSelectedAgent(null);
                        setIsCreating(false);
                    }}
                    onUpdate={handleUpdateAgent}
                    isNew={isCreating}
                />
            )}
        </div>
    );
}
