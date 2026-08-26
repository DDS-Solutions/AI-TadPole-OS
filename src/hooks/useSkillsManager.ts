/**
 * @docs ARCHITECTURE:UI-Hooks
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend React Hooks / useSkillsManager
 * - **Primary Entrypoints**: `useSkillsManager`, `SkillsManagerState`, `Tab_Type`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` React hook lifecycle adheres to Rules of Hooks without conditional execution branches.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { useReducer, useEffect, useCallback, useMemo } from 'react';
import { use_skill_store, type Skill_Definition, type Workflow_Definition, type Hook_Definition, type Mcp_Tool_Hub_Definition } from '../stores/skill_store';
import { use_agent_store } from '../stores/agent_store';
import { agent_service } from '../services/agent_service';
import { SkillParser } from '../utils/skill_parser';
import { i18n } from '../i18n';
import type { Agent } from '../types';

export type Tab_Type = 'all' | 'scripts' | 'workflows' | 'hooks' | 'mcp';

export interface SkillsManagerState {
    active_tab: Tab_Type;
    search_query: string;
    active_category: 'user' | 'ai';
    selected_tool: Mcp_Tool_Hub_Definition | null;
    is_lab_open: boolean;
    import_modal_open: boolean;
    preview_data: Skill_Definition | Workflow_Definition | Hook_Definition | null;
    preview_text: string;
    preview_type: string;
    editing_skill: Partial<Skill_Definition> | null;
    editing_workflow: Partial<Workflow_Definition> | null;
    editing_hook: Partial<Hook_Definition> | null;
    hook_form: Hook_Definition;
    assignment_modal_open: boolean;
    assigning_item: { type: 'skill' | 'workflow' | 'mcp'; name: string } | null;
    is_saving: boolean;
    save_error: string | null;
    schema_error: string | null;
    confirm_dialog: {
        is_open: boolean;
        title: string;
        message: string;
        on_confirm: () => void;
    };
}

const initial_hook_form: Hook_Definition = {
    name: '',
    description: '',
    hook_type: 'pre_validation',
    content: '',
    active: true,
    category: 'user'
};

const initial_state: SkillsManagerState = {
    active_tab: 'all',
    search_query: '',
    active_category: 'user',
    selected_tool: null,
    is_lab_open: false,
    import_modal_open: false,
    preview_data: null,
    preview_text: '',
    preview_type: 'skill',
    editing_skill: null,
    editing_workflow: null,
    editing_hook: null,
    hook_form: initial_hook_form,
    assignment_modal_open: false,
    assigning_item: null,
    is_saving: false,
    save_error: null,
    schema_error: null,
    confirm_dialog: {
        is_open: false,
        title: '',
        message: '',
        on_confirm: () => {}
    }
};

type SetFieldAction<K extends keyof SkillsManagerState> = {
    type: 'SET_FIELD';
    field: K;
    value: SkillsManagerState[K];
};

type Action =
    | { [K in keyof SkillsManagerState]: SetFieldAction<K> }[keyof SkillsManagerState]
    | { type: 'SET_FIELDS'; fields: Partial<SkillsManagerState> }
    | { type: 'UPDATE_HOOK_FORM'; value: Hook_Definition | ((prev: Hook_Definition) => Hook_Definition) }
    | { type: 'UPDATE_CONFIRM_DIALOG'; value: SkillsManagerState['confirm_dialog'] | ((prev: SkillsManagerState['confirm_dialog']) => SkillsManagerState['confirm_dialog']) }
    | { type: 'RESET_HOOK_FORM' };

function skills_manager_reducer(state: SkillsManagerState, action: Action): SkillsManagerState {
    switch (action.type) {
        case 'SET_FIELD':
            return { ...state, [action.field]: action.value };
        case 'SET_FIELDS':
            return { ...state, ...action.fields };
        case 'UPDATE_HOOK_FORM':
            return {
                ...state,
                hook_form: typeof action.value === 'function' ? action.value(state.hook_form) : action.value
            };
        case 'UPDATE_CONFIRM_DIALOG':
            return {
                ...state,
                confirm_dialog: typeof action.value === 'function' ? action.value(state.confirm_dialog) : action.value
            };
        case 'RESET_HOOK_FORM':
            return { ...state, hook_form: initial_hook_form, editing_hook: {} };
        default:
            return state;
    }
}

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
        save_hook,
        delete_hook,
        error: store_error
    } = use_skill_store();

    const agents = use_agent_store(s => s.agents);
    const [state, dispatch] = useReducer(skills_manager_reducer, initial_state);

    // Strictly-typed stable setters
    const set_active_tab = useCallback((value: Tab_Type) => dispatch({ type: 'SET_FIELD', field: 'active_tab', value }), []);
    const set_search_query = useCallback((value: string) => dispatch({ type: 'SET_FIELD', field: 'search_query', value }), []);
    const set_active_category = useCallback((value: 'user' | 'ai') => dispatch({ type: 'SET_FIELD', field: 'active_category', value }), []);
    const set_selected_tool = useCallback((value: Mcp_Tool_Hub_Definition | null) => dispatch({ type: 'SET_FIELD', field: 'selected_tool', value }), []);
    const set_is_lab_open = useCallback((value: boolean) => dispatch({ type: 'SET_FIELD', field: 'is_lab_open', value }), []);
    const set_import_modal_open = useCallback((value: boolean) => dispatch({ type: 'SET_FIELD', field: 'import_modal_open', value }), []);
    const set_editing_skill = useCallback((value: Partial<Skill_Definition> | null) => dispatch({ type: 'SET_FIELD', field: 'editing_skill', value }), []);
    const set_editing_workflow = useCallback((value: Partial<Workflow_Definition> | null) => dispatch({ type: 'SET_FIELD', field: 'editing_workflow', value }), []);
    const set_editing_hook = useCallback((value: Partial<Hook_Definition> | null) => dispatch({ type: 'SET_FIELD', field: 'editing_hook', value }), []);
    const set_assignment_modal_open = useCallback((value: boolean) => dispatch({ type: 'SET_FIELD', field: 'assignment_modal_open', value }), []);
    const set_assigning_item = useCallback((value: { type: 'skill' | 'workflow' | 'mcp'; name: string } | null) => dispatch({ type: 'SET_FIELD', field: 'assigning_item', value }), []);
    const set_schema_error = useCallback((value: string | null) => dispatch({ type: 'SET_FIELD', field: 'schema_error', value }), []);

    // Stable callback updaters without state dependency loops
    const set_hook_form = useCallback((value: Hook_Definition | ((prev: Hook_Definition) => Hook_Definition)) => {
        dispatch({ type: 'UPDATE_HOOK_FORM', value });
    }, []);

    const set_confirm_dialog = useCallback((value: SkillsManagerState['confirm_dialog'] | ((prev: SkillsManagerState['confirm_dialog']) => SkillsManagerState['confirm_dialog'])) => {
        dispatch({ type: 'UPDATE_CONFIRM_DIALOG', value });
    }, []);

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
                    dispatch({
                        type: 'SET_FIELDS',
                        fields: {
                            preview_data: parsed.data,
                            preview_text: text,
                            preview_type: parsed.type,
                            import_modal_open: true
                        }
                    });
                }
            }
        };
        input.click();
    }, []);

    const on_confirm_import = useCallback(async (data: Skill_Definition | Workflow_Definition | Hook_Definition, category: 'user' | 'ai') => {
        dispatch({ type: 'SET_FIELD', field: 'is_saving', value: true });
        try {
            if (state.preview_type === 'skill') {
                await save_skill_script({ ...data as Skill_Definition, category });
            } else if (state.preview_type === 'workflow') {
                await save_workflow({ ...data as Workflow_Definition, category });
            }
            dispatch({ type: 'SET_FIELD', field: 'import_modal_open', value: false });
            fetch_skills();
        } catch (err) {
            dispatch({
                type: 'SET_FIELD',
                field: 'save_error',
                value: (err as Error).message || 'Failed to save imported capability'
            });
        } finally {
            dispatch({ type: 'SET_FIELD', field: 'is_saving', value: false });
        }
    }, [state.preview_type, save_skill_script, save_workflow, fetch_skills]);

    // DRY Save Helper
    const execute_save = useCallback(async (save_fn: () => Promise<void>, reset_field: 'editing_skill' | 'editing_workflow' | 'editing_hook') => {
        dispatch({ type: 'SET_FIELD', field: 'is_saving', value: true });
        try {
            await save_fn();
            dispatch({ type: 'SET_FIELD', field: reset_field, value: null });
            fetch_skills();
        } catch (err) {
            dispatch({ type: 'SET_FIELD', field: 'save_error', value: (err as Error).message || 'Save operation failed' });
        } finally {
            dispatch({ type: 'SET_FIELD', field: 'is_saving', value: false });
        }
    }, [fetch_skills]);

    const handle_save_skill = useCallback(async () => {
        if (!state.editing_skill) return;
        await execute_save(() => save_skill_script(state.editing_skill as Skill_Definition), 'editing_skill');
    }, [state.editing_skill, execute_save, save_skill_script]);

    const handle_save_workflow = useCallback(async () => {
        if (!state.editing_workflow) return;
        await execute_save(() => save_workflow(state.editing_workflow as Workflow_Definition), 'editing_workflow');
    }, [state.editing_workflow, execute_save, save_workflow]);

    const handle_save_hook = useCallback(async () => {
        if (!state.editing_hook) return;
        await execute_save(() => save_hook(state.hook_form), 'editing_hook');
    }, [state.editing_hook, state.hook_form, execute_save, save_hook]);

    // DRY Delete Helper
    const confirm_and_delete = useCallback((title: string, message: string, delete_fn: () => Promise<void>) => {
        dispatch({
            type: 'SET_FIELD',
            field: 'confirm_dialog',
            value: {
                is_open: true,
                title,
                message,
                on_confirm: async () => {
                    await delete_fn();
                    fetch_skills();
                    dispatch({
                        type: 'UPDATE_CONFIRM_DIALOG',
                        value: (prev) => ({ ...prev, is_open: false })
                    });
                }
            }
        });
    }, [fetch_skills]);

    const handle_delete_skill = useCallback((name: string) => {
        confirm_and_delete(
            i18n.t('skills.confirm_delete_skill_title'),
            i18n.t('skills.confirm_delete_skill_msg', { name }),
            () => delete_skill_script(name)
        );
    }, [confirm_and_delete, delete_skill_script]);

    const handle_delete_workflow = useCallback((name: string) => {
        confirm_and_delete(
            i18n.t('skills.confirm_delete_workflow_title'),
            i18n.t('skills.confirm_delete_workflow_msg', { name }),
            () => delete_workflow(name)
        );
    }, [confirm_and_delete, delete_workflow]);

    const handle_delete_hook = useCallback((name: string) => {
        confirm_and_delete(
            i18n.t('skills.confirm_delete_hook_title', { defaultValue: 'Decommission Hook' }),
            i18n.t('skills.confirm_delete_hook_msg', { defaultValue: `Are you sure you want to decommission hook '${name}'?` }),
            () => delete_hook(name)
        );
    }, [confirm_and_delete, delete_hook]);

    const handle_toggle_assignment = useCallback(async (agent_id: string) => {
        const assigning_item = state.assigning_item;
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
    }, [state.assigning_item, agents]);

    // Memoized filtered projections (O(1) recalculation on unrelated form renders)
    const filtered_scripts = useMemo(() => 
        scripts.filter(s => 
            s.name.toLowerCase().includes(state.search_query.toLowerCase()) || 
            s.description.toLowerCase().includes(state.search_query.toLowerCase())
        ), [scripts, state.search_query]
    );

    const filtered_workflows = useMemo(() => 
        workflows.filter(w => 
            w.name.toLowerCase().includes(state.search_query.toLowerCase())
        ), [workflows, state.search_query]
    );

    const filtered_hooks = useMemo(() => 
        hooks.filter(h => 
            h.name.toLowerCase().includes(state.search_query.toLowerCase()) ||
            h.description.toLowerCase().includes(state.search_query.toLowerCase())
        ), [hooks, state.search_query]
    );

    const filtered_mcp = useMemo(() => 
        mcp_tools.filter(t => 
            t.name.toLowerCase().includes(state.search_query.toLowerCase()) ||
            t.description.toLowerCase().includes(state.search_query.toLowerCase())
        ), [mcp_tools, state.search_query]
    );

    return {
        // State
        manifests,
        scripts: filtered_scripts,
        workflows: filtered_workflows,
        hooks: filtered_hooks,
        mcp_tools: filtered_mcp,
        agents,
        active_tab: state.active_tab,
        search_query: state.search_query,
        active_category: state.active_category,
        selected_tool: state.selected_tool,
        is_lab_open: state.is_lab_open,
        import_modal_open: state.import_modal_open,
        preview_data: state.preview_data,
        preview_text: state.preview_text,
        preview_type: state.preview_type,
        editing_skill: state.editing_skill,
        editing_workflow: state.editing_workflow,
        editing_hook: state.editing_hook,
        hook_form: state.hook_form,
        assignment_modal_open: state.assignment_modal_open,
        assigning_item: state.assigning_item,
        is_saving: state.is_saving,
        save_error: state.save_error,
        schema_error: state.schema_error,
        confirm_dialog: state.confirm_dialog,
        store_error,

        // Setters
        set_active_tab,
        set_search_query,
        set_active_category,
        set_selected_tool,
        set_is_lab_open,
        set_import_modal_open,
        set_editing_skill,
        set_editing_workflow,
        set_editing_hook,
        set_hook_form,
        set_assignment_modal_open,
        set_assigning_item,
        set_schema_error,
        set_confirm_dialog,

        // Handlers
        handle_import_click,
        on_confirm_import,
        handle_delete_skill,
        handle_delete_workflow,
        handle_delete_hook,
        handle_save_skill,
        handle_save_workflow,
        handle_save_hook,
        handle_toggle_assignment,
        handle_edit_hook: (hook: Hook_Definition) => {
            dispatch({
                type: 'SET_FIELDS',
                fields: {
                    hook_form: hook,
                    editing_hook: hook
                }
            });
        },
        handle_create_hook: () => {
            dispatch({ type: 'RESET_HOOK_FORM' });
        },
        handle_assign: (type: 'skill' | 'workflow' | 'mcp', name: string) => {
            dispatch({
                type: 'SET_FIELDS',
                fields: {
                    assigning_item: { type, name },
                    assignment_modal_open: true
                }
            });
        }
    };
}
