/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **Root View**: Capability Forge control center. 
 * Orchestrates the registry and management of Skills, Workflows, Hooks, and MCP laboratory integrations via `skill_store`.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Store hydration delay (blank lists), or MCP discovery timeout for external servers.
 * - **Telemetry Link**: Search for `[Skills_View]` or `FORGE_SYNC` in service logs.
 */

import { useEffect, useState } from 'react';
import { 
    Terminal, 
    Workflow, 
    Link, 
    Plus, 
    RefreshCw, 
    Search,
    Shield,
    Activity,
    Box,
    Zap
} from 'lucide-react';
import { use_skill_store, type Mcp_Tool_Hub_Definition } from '../stores/skill_store';
import { Hook_List, Mcp_Tool_List, Mcp_Lab_Modal } from '../components/skills';
import { Tw_Empty_State, Tooltip } from '../components/ui';
import { i18n } from '../i18n';

type Tab_Type = 'scripts' | 'workflows' | 'hooks' | 'mcp';

export default function Skills() {
    const { 
        scripts, 
        workflows, 
        hooks, 
        mcp_tools,
        fetch_skills, 
        fetch_mcp_tools,
        is_loading,
        error 
    } = use_skill_store();

    const [active_tab, set_active_tab] = useState<Tab_Type>('scripts');
    const [search_query, set_search_query] = useState('');
    const [selected_tool, set_selected_tool] = useState<Mcp_Tool_Hub_Definition | null>(null);
    const [is_lab_open, set_is_lab_open] = useState(false);

    useEffect(() => {
        fetch_skills();
        fetch_mcp_tools();
    }, [fetch_skills, fetch_mcp_tools]);

    const handle_refresh = () => {
        fetch_skills();
        fetch_mcp_tools();
    };

    const handle_edit_tool = (tool: Mcp_Tool_Hub_Definition) => {
        set_selected_tool(tool);
        set_is_lab_open(true);
    };

    const filtered_scripts = scripts.filter(s => 
        s.name.toLowerCase().includes(search_query.toLowerCase()) || 
        s.description.toLowerCase().includes(search_query.toLowerCase())
    );

    const filtered_workflows = workflows.filter(w => 
        w.name.toLowerCase().includes(search_query.toLowerCase())
    );

    const filtered_hooks = hooks.filter(h => 
        h.name.toLowerCase().includes(search_query.toLowerCase()) ||
        h.description.toLowerCase().includes(search_query.toLowerCase())
    );

    const filtered_mcp = mcp_tools.filter(t => 
        t.name.toLowerCase().includes(search_query.toLowerCase()) ||
        t.description.toLowerCase().includes(search_query.toLowerCase())
    );

    const tabs: { id: Tab_Type; label: string; icon: React.ElementType; count: number }[] = [
        { id: 'scripts', label: i18n.t('skills.tab_scripts'), icon: Terminal, count: scripts.length },
        { id: 'workflows', label: i18n.t('skills.tab_workflows'), icon: Workflow, count: workflows.length },
        { id: 'hooks', label: i18n.t('skills.tab_hooks'), icon: Link, count: hooks.length },
        { id: 'mcp', label: i18n.t('skills.tab_mcp'), icon: Box, count: mcp_tools.length }
    ];

    return (
        <div className="p-6 space-y-6 max-w-7xl mx-auto">
            {/* Page Header */}
            <div className="flex flex-col md:flex-row justify-between items-start md:items-center gap-6">
                <div className="space-y-1">
                    <h1 className="text-3xl font-bold text-zinc-100 tracking-tight flex items-center gap-3">
                        <Zap className="text-blue-500" />
                        {i18n.t('skills.page_title')}
                    </h1>
                    <p className="text-zinc-500 max-w-lg">{i18n.t('skills.page_subtitle')}</p>
                </div>

                <div className="flex items-center gap-3 w-full md:w-auto">
                    <div className="relative flex-1 md:w-64">
                        <Search className="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-zinc-500" />
                        <input
                            type="text"
                            placeholder={i18n.t('skills.search_placeholder')}
                            className="w-full bg-zinc-900 border border-zinc-800 rounded-xl pl-9 pr-4 py-2 text-sm text-zinc-200 focus:outline-none focus:border-blue-500 transition-all font-mono"
                            value={search_query}
                            onChange={(e) => set_search_query(e.target.value)}
                        />
                    </div>
                    <button 
                        onClick={handle_refresh}
                        className="p-2.5 bg-zinc-900 border border-zinc-800 rounded-xl text-zinc-400 hover:text-zinc-100 hover:border-zinc-700 transition-all group"
                        disabled={is_loading}
                    >
                        <RefreshCw className={`w-5 h-5 ${is_loading ? 'animate-spin text-blue-500' : 'group-active:rotate-180 transition-transform duration-500'}`} />
                    </button>
                    <button className="flex items-center gap-2 px-4 py-2.5 bg-blue-600 hover:bg-blue-500 text-white rounded-xl text-sm font-bold transition-all shadow-lg shadow-blue-500/20 active:scale-95">
                        <Plus size={18} />
                        {i18n.t('skills.create_button')}
                    </button>
                </div>
            </div>

            {/* Error State */}
            {error && (
                <div className="bg-red-500/10 border border-red-500/20 p-4 rounded-xl flex items-center gap-3 animate-in fade-in slide-in-from-top-2">
                    <Shield className="text-red-500" size={18} />
                    <p className="text-sm font-medium text-red-500">{error}</p>
                </div>
            )}

            {/* Tabs Navigation */}
            <div className="flex items-center gap-1 bg-zinc-950 p-1 rounded-2xl border border-zinc-900 overflow-x-auto no-scrollbar">
                {tabs.map((tab) => (
                    <button
                        key={tab.id}
                        onClick={() => set_active_tab(tab.id)}
                        className={`flex items-center gap-3 px-6 py-3 rounded-xl text-sm font-bold transition-all whitespace-nowrap group ${
                            active_tab === tab.id
                                ? 'bg-zinc-800 text-blue-400 shadow-xl'
                                : 'text-zinc-500 hover:text-zinc-300 hover:bg-zinc-900/50'
                        }`}
                    >
                        <tab.icon size={18} className={active_tab === tab.id ? 'text-blue-500' : 'text-zinc-600 group-hover:text-zinc-400'} />
                        {tab.label}
                        <span className={`text-[10px] px-1.5 py-0.5 rounded-full font-mono ${
                            active_tab === tab.id ? 'bg-blue-500/20 text-blue-400' : 'bg-zinc-900 text-zinc-600'
                        }`}>
                            {tab.count}
                        </span>
                    </button>
                ))}
            </div>

            {/* Tab Content */}
            <div className="min-h-[400px]">
                {active_tab === 'scripts' && (
                    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                        {filtered_scripts.map(script => (
                            <div key={script.name} className="bg-zinc-900 border border-zinc-800 p-5 rounded-2xl space-y-4 hover:border-zinc-700 transition-all group">
                                <div className="flex justify-between items-center">
                                    <div className="flex items-center gap-3">
                                        <div className="p-2 bg-blue-500/10 rounded-xl">
                                            <Terminal size={20} className="text-blue-500" />
                                        </div>
                                        <h3 className="font-bold text-zinc-100 uppercase tracking-tight group-hover:text-blue-400 transition-colors">{script.name}</h3>
                                    </div>
                                    <Tooltip content={i18n.t('skills.verified_protocol')}>
                                        <Shield size={14} className="text-emerald-500 opacity-40 group-hover:opacity-100 transition-opacity" />
                                    </Tooltip>
                                </div>
                                <p className="text-sm text-zinc-400 leading-relaxed line-clamp-2">{script.description}</p>
                                <div className="flex items-center gap-2 pt-2">
                                    <div className="px-2 py-1 bg-zinc-950 border border-zinc-800 rounded-lg text-[9px] font-mono text-zinc-500 uppercase">
                                        {script.category}
                                    </div>
                                    <div className="flex-1 h-px bg-zinc-800/50" />
                                    <button className="text-[10px] font-bold text-blue-500 uppercase tracking-widest hover:text-blue-400 transition-colors">
                                        {i18n.t('skills.view_source')}
                                    </button>
                                </div>
                            </div>
                        ))}
                        {filtered_scripts.length === 0 && (
                            <div className="col-span-full">
                                <Tw_Empty_State title={i18n.t('skills.no_scripts_found')} />
                            </div>
                        )}
                    </div>
                )}

                {active_tab === 'workflows' && (
                    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                        {filtered_workflows.map(wf => (
                            <div key={wf.name} className="bg-zinc-900 border border-zinc-800 p-5 rounded-2xl space-y-4 hover:border-zinc-700 transition-all group">
                                <div className="flex justify-between items-center">
                                    <div className="flex items-center gap-3">
                                        <div className="p-2 bg-orange-500/10 rounded-xl">
                                            <Workflow size={20} className="text-orange-500" />
                                        </div>
                                        <h3 className="font-bold text-zinc-100 uppercase tracking-tight group-hover:text-orange-400 transition-colors">{wf.name}</h3>
                                    </div>
                                    <Activity size={14} className="text-zinc-600 group-hover:text-amber-500 transition-colors" />
                                </div>
                                <div className="bg-zinc-950 p-3 rounded-xl border border-zinc-900">
                                    <p className="text-[10px] font-mono text-zinc-500 leading-relaxed truncate">{wf.content}</p>
                                </div>
                            </div>
                        ))}
                        {filtered_workflows.length === 0 && (
                            <div className="col-span-full">
                                <Tw_Empty_State title={i18n.t('skills.no_workflows_found')} />
                            </div>
                        )}
                    </div>
                )}

                {active_tab === 'hooks' && (
                    <Hook_List 
                        hooks={filtered_hooks} 
                        on_edit={() => {}} 
                        on_delete={() => {}}
                        on_create={() => {}}
                    />
                )}

                {active_tab === 'mcp' && (
                    <Mcp_Tool_List 
                        tools={filtered_mcp} 
                        on_edit={handle_edit_tool} 
                    />
                )}
            </div>

            <Mcp_Lab_Modal
                tool={selected_tool}
                open={is_lab_open}
                on_close={() => set_is_lab_open(false)}
            />
        </div>
    );
}

