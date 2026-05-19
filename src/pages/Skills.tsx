/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **Root View**: Capability Forge control center. 
 * Orchestrates the registry and management of Skills, Workflows, Hooks, and MCP laboratory integrations via `useSkillsManager`.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Store hydration delay (blank lists), or MCP discovery timeout for external servers.
 * - **Telemetry Link**: Search for `[Skills_View]` or `FORGE_SYNC` in service logs.
 */

import { 
    Terminal, 
    Workflow, 
    Link,
    Box
} from 'lucide-react';
import { useSkillsManager, type Tab_Type } from '../hooks/useSkillsManager';
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

export default function Skills() {
    const {
        manifests, scripts, workflows, hooks, mcp_tools,
        agents, active_tab, search_query, active_category,
        selected_tool, is_lab_open,
        import_modal_open, preview_data, preview_text, preview_type,
        editing_skill, editing_workflow,
        assignment_modal_open, assigning_item,
        is_saving, save_error, schema_error, confirm_dialog, store_error,
        
        set_active_tab, set_search_query, set_active_category,
        set_selected_tool, set_is_lab_open, set_import_modal_open, set_editing_skill, set_editing_workflow,
        set_assignment_modal_open, set_schema_error, set_confirm_dialog,
        
        handle_import_click, on_confirm_import,
        handle_delete_skill, handle_delete_workflow,
        handle_save_skill, handle_save_workflow,
        handle_toggle_assignment, handle_assign
    } = useSkillsManager();

    const tabs: { id: Tab_Type; label: string; icon: React.ElementType; count: number; tooltip: string }[] = [
        { id: 'all', label: i18n.t('skills.tab_all', { defaultValue: 'All Abilities' }), icon: Terminal, count: manifests.length + scripts.length + workflows.length, tooltip: 'Unified swarm capabilities' },
        { id: 'scripts', label: i18n.t('skills.tab_skills'), icon: Terminal, count: scripts.length, tooltip: i18n.t('skills.tooltip_skills') },
        { id: 'workflows', label: i18n.t('skills.tab_workflows'), icon: Workflow, count: workflows.length, tooltip: i18n.t('skills.tooltip_workflows') },
        { id: 'hooks', label: i18n.t('skills.tab_hooks'), icon: Link, count: hooks.length, tooltip: i18n.t('skills.tooltip_hooks') },
        { id: 'mcp', label: i18n.t('skills.tab_mcp'), icon: Box, count: mcp_tools.length, tooltip: i18n.t('skills.tooltip_mcp') }
    ];

    return (
        <div className="p-6 space-y-6 max-w-7xl mx-auto">
            {/* GEO Optimization: Structured Data & Semantic Header */}
            <script type="application/ld+json">
            {JSON.stringify({
              "@context": "https://schema.org",
              "@type": "SoftwareApplication",
              "name": "Tadpole OS Capability Forge",
              "description": "Registry and management system for autonomous agent skills and multi-step workflows. Features integrated MCP bridging and skill-chaining diagnostics.",
              "author": { "@type": "Organization", "name": "Sovereign Engineering" },
              "applicationCategory": "System Configuration",
              "operatingSystem": "Tadpole OS"
            })}
            </script>
            <h1 className="sr-only">Tadpole OS Capability Forge: Skill & Workflow Registry</h1>

            {store_error && (
                <div className="bg-red-500/10 border border-red-500/50 text-red-500 px-4 py-3 rounded-xl text-sm font-medium animate-pulse">
                    {store_error}
                </div>
            )}

            <Skill_Header 
                stats={{
                    user_registry_count: scripts.filter(s => s.category === 'user').length + workflows.filter(w => w.category === 'user').length,
                    ai_services_count: scripts.filter(s => s.category === 'ai').length + workflows.filter(w => w.category === 'ai').length
                }}
                handlers={{
                    set_active_category,
                    set_search_query,
                    handle_import_click,
                    on_create_skill: () => set_editing_skill({ name: '', description: '', execution_command: '', schema: {}, category: 'user' }),
                    on_create_workflow: () => set_editing_workflow({ name: '', content: '', category: 'user' })
                }}
                state={{
                    active_category,
                    search_query,
                    is_saving
                }}
            />

            <div className="flex items-center gap-1 bg-[color:var(--color-background)] p-1 rounded-2xl border border-[color:var(--color-surface)] overflow-x-auto no-scrollbar">
                {tabs.map((tab) => (
                    <Tooltip key={tab.id} content={tab.tooltip} position="bottom">
                        <button
                            onClick={() => set_active_tab(tab.id)}
                            className={`flex items-center gap-3 px-6 py-3 rounded-xl text-sm font-bold transition-all whitespace-nowrap group ${
                                active_tab === tab.id
                                    ? 'bg-zinc-800 text-green-400 shadow-xl'
                                    : 'text-zinc-500 hover:text-zinc-300 hover:bg-[color:var(--color-surface)]/50'
                            }`}
                        >
                            <tab.icon size={18} className={active_tab === tab.id ? 'text-green-500' : 'text-zinc-600 group-hover:text-zinc-400'} />
                            {tab.label}
                            <span className={`text-[10px] px-1.5 py-0.5 rounded-full font-mono ${
                                active_tab === tab.id ? 'bg-green-500/20 text-green-400' : 'bg-[color:var(--color-surface)] text-zinc-600'
                            }`}>
                                {tab.count}
                            </span>
                        </button>
                    </Tooltip>
                ))}
            </div>

            <div className="min-h-[400px]">
                {active_tab === 'all' && (
                    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
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
                        {scripts.map(script => (
                            <Skill_Card 
                                key={`s-${script.name}`} 
                                skill={script} 
                                on_edit={() => set_editing_skill(script)} 
                                on_delete={() => handle_delete_skill(script.name)}
                                on_assign={() => handle_assign('skill', script.name)}
                            />
                        ))}
                        {workflows.map(wf => (
                            <Workflow_Card 
                                key={`w-${wf.name}`} 
                                workflow={wf} 
                                on_edit={() => set_editing_workflow(wf)} 
                                on_delete={() => handle_delete_workflow(wf.name)}
                                on_assign={() => handle_assign('workflow', wf.name)}
                            />
                        ))}
                    </div>
                )}

                {active_tab === 'scripts' && (
                    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                        {scripts.map(script => (
                            <Skill_Card 
                                key={script.name} 
                                skill={script} 
                                on_edit={() => set_editing_skill(script)} 
                                on_delete={() => handle_delete_skill(script.name)}
                                on_assign={() => handle_assign('skill', script.name)}
                            />
                        ))}
                        {scripts.length === 0 && (
                            <div className="col-span-full">
                                <Tw_Empty_State title={i18n.t('skills.empty_scripts_title')} description={i18n.t('skills.empty_scripts_desc')} />
                            </div>
                        )}
                    </div>
                )}

                {active_tab === 'workflows' && (
                    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                        {workflows.map(wf => (
                            <Workflow_Card 
                                key={wf.name} 
                                workflow={wf} 
                                on_edit={() => set_editing_workflow(wf)} 
                                on_delete={() => handle_delete_workflow(wf.name)}
                                on_assign={() => handle_assign('workflow', wf.name)}
                            />
                        ))}
                        {workflows.length === 0 && (
                            <div className="col-span-full">
                                <Tw_Empty_State title={i18n.t('skills.empty_workflows_title')} description={i18n.t('skills.empty_workflows_desc')} />
                            </div>
                        )}
                    </div>
                )}

                {active_tab === 'hooks' && (
                    <Hook_List 
                        hooks={hooks} 
                        on_edit={() => {}} 
                        on_delete={() => {}}
                        on_create={() => {}}
                    />
                )}

                {active_tab === 'mcp' && (
                    <Mcp_Tool_List 
                        tools={mcp_tools} 
                        on_edit={(tool) => { set_selected_tool(tool); set_is_lab_open(true); }} 
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

// Metadata: [Skills]

// Metadata: [Skills]
