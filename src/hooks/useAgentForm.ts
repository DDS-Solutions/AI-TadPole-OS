/**
 * @docs ARCHITECTURE:Logic
 * @docs OPERATIONS_MANUAL:Agents
 * 
 * ### AI Assist Note
 * **Complex Form Hook**: Manages the multi-tab state for agent configuration (Cognition, Memory, Governance). 
 * Orchestrates tri-slot model configuration, skill/workflow toggling, and voice engine selection.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Reducer state mismatch (invalid action type), model slot collision (overwriting slot 1 with slot 2 data), or stale role defaults if `role_store` is uninitialized.
 * - **Telemetry Link**: Check `config_reducer` trace points or search `[useAgentForm]` in component logs.
 */

import { use_role_store } from '../stores/role_store';
import type {
    Agent_Connector_Config,
    Agent_Model_Slot_Key,
    Agent_Model_Slot_State,
    Agent_Stt_Engine,
    Agent_Voice_Engine,
} from '../types';

export interface Agent_Config_State {
    main_tab: 'cognition' | 'memory' | 'governance';
    active_tab: Agent_Model_Slot_Key;
    identity: {
        name: string;
        role: string;
        department: string;
    };
    voice: {
        voice_id: string;
        voice_engine: Agent_Voice_Engine;
        stt_engine?: Agent_Stt_Engine;
    };
    slots: Record<Agent_Model_Slot_Key, Agent_Model_Slot_State>;
    mcp_tools: string[];
    governance: {
        budget_usd: number;
        requires_oversight: boolean;
    };
    ui: {
        direct_message: string;
        saving: boolean;
        theme_color: string;
        new_role_name: string;
        show_promote: boolean;
    };
    connector_configs: Agent_Connector_Config[];
}

export type Agent_Config_Action =
    | { type: 'SET_MAIN_TAB'; payload: 'cognition' | 'memory' | 'governance' }
    | { type: 'SET_TAB'; payload: Agent_Model_Slot_Key }
    | { type: 'UPDATE_IDENTITY'; field: 'name' | 'role' | 'department'; value: string }
    | { type: 'UPDATE_SLOT'; slot: Agent_Model_Slot_Key; field: 'model' | 'provider' | 'system_prompt'; value: string }
    | { type: 'UPDATE_SLOT'; slot: Agent_Model_Slot_Key; field: 'temperature' | 'reasoning_depth' | 'act_threshold'; value: number }
    | { type: 'UPDATE_SLOT'; slot: Agent_Model_Slot_Key; field: 'skills' | 'workflows'; value: string[] }
    | { type: 'TOGGLE_SKILL'; slot: Agent_Model_Slot_Key; kind: 'skills' | 'workflows'; value: string }
    | { type: 'RESET_ROLE'; role: string }
    | { type: 'UPDATE_GOVERNANCE'; field: 'budget_usd'; value: number }
    | { type: 'UPDATE_GOVERNANCE'; field: 'requires_oversight'; value: boolean }
    | { type: 'UPDATE_VOICE'; field: 'voice_id' | 'voice_engine' | 'stt_engine'; value: string }
    | { type: 'TOGGLE_MCP_TOOL'; value: string }
    | { type: 'ADD_CONNECTOR'; payload: Agent_Connector_Config }
    | { type: 'REMOVE_CONNECTOR'; uri: string }
    | { type: 'SET_UI'; field: 'direct_message' | 'saving' | 'theme_color' | 'new_role_name' | 'show_promote'; value: string | boolean };

/**
 * config_reducer
 * Managed state for the agent configuration form.
 * Handles identity updates, model slot orchestration, and capability toggling.
 */
export function config_reducer(state: Agent_Config_State, action: Agent_Config_Action): Agent_Config_State {
    switch (action.type) {
        case 'SET_MAIN_TAB':
            return { ...state, main_tab: action.payload };
        case 'SET_TAB':
            return { ...state, active_tab: action.payload };
        case 'UPDATE_IDENTITY':
            return {
                ...state,
                identity: { ...state.identity, [action.field]: action.value }
            };
        case 'UPDATE_SLOT':
            return {
                ...state,
                slots: {
                    ...state.slots,
                    [action.slot]: {
                        ...state.slots[action.slot],
                        [action.field]: action.value
                    }
                }
            };
        case 'TOGGLE_SKILL': {
            const current_list = state.slots[action.slot][action.kind];
            const new_list = current_list.includes(action.value)
                ? current_list.filter(item => item !== action.value)
                : [...current_list, action.value];
            return {
                ...state,
                slots: {
                    ...state.slots,
                    [action.slot]: {
                        ...state.slots[action.slot],
                        [action.kind]: new_list
                    }
                }
            };
        }
        case 'RESET_ROLE': {
            const defaults = use_role_store.getState().roles[action.role] || { skills: [], workflows: [] };
            return {
                ...state,
                identity: { ...state.identity, role: action.role },
                slots: {
                    ...state.slots,
                    primary: { ...state.slots.primary, skills: defaults.skills, workflows: defaults.workflows },
                    secondary: { ...state.slots.secondary, skills: [], workflows: [] },
                    tertiary: { ...state.slots.tertiary, skills: [], workflows: [] }
                }
            };
        }
        case 'UPDATE_VOICE':
            return {
                ...state,
                voice: { ...state.voice, [action.field]: action.value as string }
            };
        case 'TOGGLE_MCP_TOOL': {
            const new_list = state.mcp_tools.includes(action.value)
                ? state.mcp_tools.filter(t => t !== action.value)
                : [...state.mcp_tools, action.value];
            return { ...state, mcp_tools: new_list };
        }
        case 'SET_UI':
            return {
                ...state,
                ui: { ...state.ui, [action.field]: action.value }
            };
        case 'UPDATE_GOVERNANCE':
            return {
                ...state,
                governance: { ...state.governance, [action.field]: action.value }
            } as Agent_Config_State;
        case 'ADD_CONNECTOR':
            return {
                ...state,
                connector_configs: [...state.connector_configs, action.payload]
            };
        case 'REMOVE_CONNECTOR':
            return {
                ...state,
                connector_configs: state.connector_configs.filter(c => c.uri !== action.uri)
            };
        default:
            return state;
    }
}


// Metadata: [use_agent_form]

// Metadata: [use_agent_form]

// Metadata: [useAgentForm]
