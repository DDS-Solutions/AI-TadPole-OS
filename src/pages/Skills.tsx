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
    Box
} from 'lucide-react';
import { use_skill_store, type Skill_Definition, type Workflow_Definition, type Hook_Definition, type Mcp_Tool_Hub_Definition } from '../stores/skill_store';
import { use_agent_store } from '../stores/agent_store';
import type { Agent } from '../types';
import { 
    Skill_Header, 
    Skill_Card, 
    Workflow_Card, 
    Hook_List, 
    Mcp_Tool_List, 
    Mcp_Lab_Modal, 
    Import_Preview_Modal,
    Skill_Edit_Modal,
    Workflow_Edit_Modal,
    Assignment_Modal
} from '../components/skills';
import { Tw_Empty_State, Tooltip, Confirm_Dialog } from '../components/ui';
import { i18n } from '../i18n';
import { SkillParser } from '../utils/skill_parser';

type Tab_Type = 'all' | 'scripts' | 'workflows' | 'hooks' | 'mcp';

export default function Skills() {
    const { 
        manifests,
        scripts, 
        workflows, 
        hooks, 
        mcp_tools,
        fetch_skills, 
        fetch_mcp_tools,
        save_skill_script,
        delete_skill_script,
        save_workflow,
        delete_workflow,
        error: store_error
    } = use_skill_store();

    const { agents, fetch_agents, update_agent } = use_agent_store();

    const [active_tab, set_active_tab] = useState<Tab_Type>('all');
    const [search_query, set_search_query] = useState('');
    const [selected_tool, set_selected_tool] = useState<Mcp_Tool_Hub_Definition | null>(null);
    const [is_lab_open, set_is_lab_open] = useState(false);

    // Import State
    const [import_modal_open, set_import_modal_open] = useState(false);
    const [preview_data, set_preview_data] = useState<Skill_Definition | Workflow_Definition | Hook_Definition | null>(null);
    const [preview_text, set_preview_text] = useState('');
    const [preview_type, set_preview_type] = useState<string>('skill');

    // Edit State
    const [editing_skill, set_editing_skill] = useState<Partial<Skill_Definition> | null>(null);
    const [editing_workflow, set_editing_workflow] = useState<Partial<Workflow_Definition> | null>(null);
    const [is_saving, set_is_saving] = useState(false);
    const [save_error, set_save_error] = useState<string | null>(null);
    const [schema_error, set_schema_error] = useState<string | null>(null);

    // Assignment State
    const [assignment_modal_open, set_assignment_modal_open] = useState(false);
    const [assigning_item, set_assigning_item] = useState<{ type: 'skill' | 'workflow' | 'mcp', name: string } | null>(null);
    const [active_category, set_active_category] = useState<'user' | 'ai'>('user');

    // Confirm State
    const [confirm_dialog, set_confirm_dialog] = useState<{
        is_open: boolean;
        title: string;
        message: string;
        on_confirm: () => void;
    }>({
        is_open: false,
        title: '',
        message: '',
        on_confirm: () => {}
    });

    const handle_import_click = () => {
        const input = document.createElement('input');
        input.type = 'file';
        input.accept = '.md';
        input.onchange = async (e) => {
            const file = (e.target as HTMLInputElement).files?.[0];
            if (file) {
                const text = await file.text();
                const parsed = SkillParser.parse_markdown(text);
                if (parsed) {
                    set_preview_data(parsed.data);
                    set_preview_text(text);
                    set_preview_type(parsed.type);
                    set_import_modal_open(true);
                }
            }
        };
        input.click();
    };

    const on_confirm_import = async (data: Skill_Definition | Workflow_Definition | Hook_Definition, category: 'user' | 'ai') => {
        set_is_saving(true);
        try {
            if (preview_type === 'skill') {
                await save_skill_script({ ...data as Skill_Definition, category });
            } else if (preview_type === 'workflow') {
                await save_workflow({ ...data as Workflow_Definition, category });
            }
            set_import_modal_open(false);
            fetch_skills();
        } catch (err) {
            set_save_error((err as Error).message || "Failed to save imported capability");
        } finally {
            set_is_saving(false);
        }
    };

    const handle_edit_skill = (skill: Skill_Definition) => {
        set_editing_skill(skill);
        set_save_error(null);
    };

    const handle_edit_workflow = (workflow: Workflow_Definition) => {
        set_editing_workflow(workflow);
        set_save_error(null);
    };

    const handle_delete_skill = (name: string) => {
        set_confirm_dialog({
            is_open: true,
            title: i18n.t('skills.confirm_delete_skill_title'),
            message: i18n.t('skills.confirm_delete_skill_msg', { name }),
            on_confirm: async () => {
                await delete_skill_script(name);
                fetch_skills();
                set_confirm_dialog(prev => ({ ...prev, is_open: false }));
            }
        });
    };

    const handle_delete_workflow = (name: string) => {
        set_confirm_dialog({
            is_open: true,
            title: i18n.t('skills.confirm_delete_workflow_title'),
            message: i18n.t('skills.confirm_delete_workflow_msg', { name }),
            on_confirm: async () => {
                await delete_workflow(name);
                fetch_skills();
                set_confirm_dialog(prev => ({ ...prev, is_open: false }));
            }
        });
    };

    const handle_assign = (type: 'skill' | 'workflow' | 'mcp', name: string) => {
        set_assigning_item({ type, name });
        set_assignment_modal_open(true);
    };

    const handle_toggle_assignment = async (agent_id: string) => {
        if (!assigning_item) return;
        const agent = agents.find(a => a.id === agent_id);
        if (!agent) return;

        const updates: Partial<Agent> = {};
        if (assigning_item.type === 'skill') {
            const current = agent.skills || [];
            updates.skills = current.includes(assigning_item.name)
                ? current.filter((s: string) => s !== assigning_item.name)
                : [...current, assigning_item.name];
        } else if (assigning_item.type === 'workflow') {
            const current = agent.workflows || [];
            updates.workflows = current.includes(assigning_item.name)
                ? current.filter((w: string) => w !== assigning_item.name)
                : [...current, assigning_item.name];
        }

        await update_agent(agent_id, updates);
    };

    const handle_save_skill = async () => {
        if (!editing_skill) return;
        set_is_saving(true);
        try {
            await save_skill_script(editing_skill as Skill_Definition);
            set_editing_skill(null);
            fetch_skills();
        } catch (err) {
            set_save_error((err as Error).message);
        } finally {
            set_is_saving(false);
        }
    };

    const handle_save_workflow = async () => {
        if (!editing_workflow) return;
        set_is_saving(true);
        try {
            await save_workflow(editing_workflow as Workflow_Definition);
            set_editing_workflow(null);
            fetch_skills();
        } catch (err) {
            set_save_error((err as Error).message);
        } finally {
            set_is_saving(false);
        }
    };

    useEffect(() => {
        fetch_skills();
        fetch_mcp_tools();
        fetch_agents();
    }, [fetch_skills, fetch_mcp_tools, fetch_agents]);



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

    const tabs: { id: Tab_Type; label: string; icon: React.ElementType; count: number; tooltip: string }[] = [
        { id: 'all', label: i18n.t('skills.tab_all', { defaultValue: 'All Abilities' }), icon: Terminal, count: manifests.length + scripts.length + workflows.length, tooltip: 'Unified swarm capabilities' },
        { id: 'scripts', label: i18n.t('skills.tab_skills'), icon: Terminal, count: scripts.length, tooltip: i18n.t('skills.tooltip_skills') },
        { id: 'workflows', label: i18n.t('skills.tab_workflows'), icon: Workflow, count: workflows.length, tooltip: i18n.t('skills.tooltip_workflows') },
        { id: 'hooks', label: i18n.t('skills.tab_hooks'), icon: Link, count: hooks.length, tooltip: i18n.t('skills.tooltip_hooks') },
        { id: 'mcp', label: i18n.t('skills.tab_mcp'), icon: Box, count: mcp_tools.length, tooltip: i18n.t('skills.tooltip_mcp') }
    ];

    return (
        <div className="p-6 space-y-6 max-w-7xl mx-auto">
            {store_error && (
                <div className="bg-red-500/10 border border-red-500/50 text-red-500 px-4 py-3 rounded-xl text-sm font-medium animate-pulse">
                    {store_error}
                </div>
            )}
            {/* Page Header */}
            <Skill_Header 
            active_category={active_category}
            set_active_category={set_active_category}
            search_query={search_query}
            set_search_query={set_search_query}
            handle_import_click={handle_import_click}
            is_saving={is_saving}
            user_registry_count={scripts.filter(s => s.category === 'user').length + workflows.filter(w => w.category === 'user').length}
            ai_services_count={scripts.filter(s => s.category === 'ai').length + workflows.filter(w => w.category === 'ai').length}
            on_create_skill={() => set_editing_skill({ name: '', description: '', execution_command: '', schema: {}, category: 'user' })}
            on_create_workflow={() => set_editing_workflow({ name: '', content: '', category: 'user' })}
        />

        {/* Tabs Navigation */}
        <div className="flex items-center gap-1 bg-zinc-950 p-1 rounded-2xl border border-zinc-900 overflow-x-auto no-scrollbar">
            {tabs.map((tab) => (
                <Tooltip key={tab.id} content={tab.tooltip} position="bottom">
                    <button
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
                </Tooltip>
            ))}
        </div>

        {/* Tab Content */}
        <div className="min-h-[400px]">
            {active_tab === 'all' && (
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                    {/* Native Engine Skills */}
                    {manifests.filter(m => m.name.toLowerCase().includes(search_query.toLowerCase())).map(m => (
                        <Skill_Card 
                            key={`m-${m.name}`}
                            skill={{
                                name: m.name,
                                description: m.description,
                                execution_command: `Native :: ${m.toolset_group || 'Core'}`,
                                schema: {},
                                category: m.category
                            }} 
                            on_edit={() => {}} 
                            on_delete={() => {}}
                            on_assign={() => handle_assign('skill', m.name)}
                        />
                    ))}
                    {/* Custom Script Skills */}
                    {filtered_scripts.map(script => (
                        <Skill_Card 
                            key={`s-${script.name}`} 
                            skill={script} 
                            on_edit={() => handle_edit_skill(script)} 
                            on_delete={() => handle_delete_skill(script.name)}
                            on_assign={() => handle_assign('skill', script.name)}
                        />
                    ))}
                    {/* Workflows */}
                    {filtered_workflows.map(wf => (
                        <Workflow_Card 
                            key={`w-${wf.name}`} 
                            workflow={wf} 
                            on_edit={() => handle_edit_workflow(wf)} 
                            on_delete={() => handle_delete_workflow(wf.name)}
                            on_assign={() => handle_assign('workflow', wf.name)}
                        />
                    ))}
                </div>
            )}

            {active_tab === 'scripts' && (
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                    {filtered_scripts.map(script => (
                        <Skill_Card 
                            key={script.name} 
                            skill={script} 
                            on_edit={() => handle_edit_skill(script)} 
                            on_delete={() => handle_delete_skill(script.name)}
                            on_assign={() => handle_assign('skill', script.name)}
                        />
                    ))}
                    {filtered_scripts.length === 0 && (
                        <div className="col-span-full">
                            <Tw_Empty_State title={i18n.t('skills.empty_scripts_title')} description={i18n.t('skills.empty_scripts_desc')} />
                        </div>
                    )}
                </div>
            )}

            {active_tab === 'workflows' && (
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                    {filtered_workflows.map(wf => (
                        <Workflow_Card 
                            key={wf.name} 
                            workflow={wf} 
                            on_edit={() => handle_edit_workflow(wf)} 
                            on_delete={() => handle_delete_workflow(wf.name)}
                            on_assign={() => handle_assign('workflow', wf.name)}
                        />
                    ))}
                    {filtered_workflows.length === 0 && (
                        <div className="col-span-full">
                            <Tw_Empty_State title={i18n.t('skills.empty_workflows_title')} description={i18n.t('skills.empty_workflows_desc')} />
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

        {/* Modals */}
        <Import_Preview_Modal 
            is_open={import_modal_open}
            on_close={() => set_import_modal_open(false)}
            data={preview_data}
            preview={preview_text}
            type={preview_type}
            on_confirm={on_confirm_import}
        />

        <Skill_Edit_Modal 
            is_open={!!editing_skill}
            on_close={() => set_editing_skill(null)}
            editing_skill={editing_skill || {}}
            set_editing_skill={set_editing_skill}
            is_saving={is_saving}
            on_save={handle_save_skill}
            schema_error={schema_error}
            set_schema_error={set_schema_error}
            skill_save_error={save_error}
        />

        <Workflow_Edit_Modal 
            is_open={!!editing_workflow}
            on_close={() => set_editing_workflow(null)}
            editing_wf={editing_workflow || {}}
            set_editing_wf={(wf) => set_editing_workflow(wf)}
            is_saving={is_saving}
            on_save={handle_save_workflow}
            wf_save_error={save_error}
        />

        <Assignment_Modal 
            is_open={assignment_modal_open}
            on_close={() => set_assignment_modal_open(false)}
            assign_target={assigning_item}
            agents={agents}
            on_toggle_assignment={handle_toggle_assignment}
        />

        <Mcp_Lab_Modal
            tool={selected_tool}
            open={is_lab_open}
            on_close={() => set_is_lab_open(false)}
        />

        <Confirm_Dialog 
            is_open={confirm_dialog.is_open}
            title={confirm_dialog.title}
            message={confirm_dialog.message}
            on_confirm={confirm_dialog.on_confirm}
            on_cancel={() => set_confirm_dialog(prev => ({ ...prev, is_open: false }))}
            confirm_label="PURGE"
            variant="danger"
        />
        </div>
    );
}

