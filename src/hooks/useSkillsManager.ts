/**
 * @docs ARCHITECTURE:UI-Hooks
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[useSkillsManager]` in observability traces.
 */

import { useState, useEffect, useCallback } from 'react';
import { use_skill_store, type Skill_Definition, type Workflow_Definition, type Hook_Definition, type Mcp_Tool_Hub_Definition } from '../stores/skill_store';
import { use_agent_store } from '../stores/agent_store';
import { agent_service } from '../services/agent_service';
import { SkillParser } from '../utils/skill_parser';
import { i18n } from '../i18n';
import type { Agent } from '../types';

export type Tab_Type = 'all' | 'scripts' | 'workflows' | 'hooks' | 'mcp';

export function useSkillsManager() {
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

    const agents = use_agent_store(s => s.agents);

    // UI State
    const [active_tab, set_active_tab] = useState<Tab_Type>('all');
    const [search_query, set_search_query] = useState('');
    const [active_category, set_active_category] = useState<'user' | 'ai'>('user');

    // Modals & Selection
    const [selected_tool, set_selected_tool] = useState<Mcp_Tool_Hub_Definition | null>(null);
    const [is_lab_open, set_is_lab_open] = useState(false);
    
    const [import_modal_open, set_import_modal_open] = useState(false);
    const [preview_data, set_preview_data] = useState<Skill_Definition | Workflow_Definition | Hook_Definition | null>(null);
    const [preview_text, set_preview_text] = useState('');
    const [preview_type, set_preview_type] = useState<string>('skill');

    const [editing_skill, set_editing_skill] = useState<Partial<Skill_Definition> | null>(null);
    const [editing_workflow, set_editing_workflow] = useState<Partial<Workflow_Definition> | null>(null);
    
    const [assignment_modal_open, set_assignment_modal_open] = useState(false);
    const [assigning_item, set_assigning_item] = useState<{ type: 'skill' | 'workflow' | 'mcp', name: string } | null>(null);

    // Operation State
    const [is_saving, set_is_saving] = useState(false);
    const [save_error, set_save_error] = useState<string | null>(null);
    const [schema_error, set_schema_error] = useState<string | null>(null);

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

    // Lifecycle
    useEffect(() => {
        fetch_skills();
        fetch_mcp_tools();
        void agent_service.load_agents_into_store();
    }, [fetch_skills, fetch_mcp_tools]);

    // Handlers
    const handle_import_click = useCallback(() => {
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
    }, [set_import_modal_open]);

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

        await agent_service.update_agent(agent_id, updates);
    };

    // Filtered data
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

    return {
        // State
        manifests, scripts: filtered_scripts, workflows: filtered_workflows, hooks: filtered_hooks, mcp_tools: filtered_mcp,
        agents, active_tab, search_query, active_category,
        selected_tool, is_lab_open,
        import_modal_open, preview_data, preview_text, preview_type,
        editing_skill, editing_workflow,
        assignment_modal_open, assigning_item,
        is_saving, save_error, schema_error, confirm_dialog, store_error,

        // Setters
        set_active_tab, set_search_query, set_active_category,
        set_selected_tool, set_is_lab_open,
        set_import_modal_open, set_editing_skill, set_editing_workflow,
        set_assignment_modal_open, set_assigning_item, set_schema_error,
        set_confirm_dialog,

        // Handlers
        handle_import_click, on_confirm_import,
        handle_delete_skill, handle_delete_workflow,
        handle_save_skill, handle_save_workflow,
        handle_toggle_assignment,
        handle_assign: (type: 'skill' | 'workflow' | 'mcp', name: string) => {
            set_assigning_item({ type, name });
            set_assignment_modal_open(true);
        }
    };
}

// Metadata: [useSkillsManager]
