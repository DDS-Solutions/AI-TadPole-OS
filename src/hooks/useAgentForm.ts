import { useRoleStore } from '../stores/roleStore';

export interface ModelSlotConfig {
    provider: string;
    model: string;
    temperature: number;
    systemPrompt: string;
    skills: string[];
    workflows: string[];
}

export interface AgentConfigState {
    mainTab: 'cognition' | 'memory' | 'governance';
    activeTab: 'primary' | 'secondary' | 'tertiary';
    identity: {
        name: string;
        role: string;
    };
    voice: {
        voiceId: string;
        voiceEngine: 'browser' | 'openai' | 'groq' | 'piper' | 'gemini-live';
        sttEngine?: 'groq' | 'whisper';
    };
    slots: {
        primary: ModelSlotConfig;
        secondary: ModelSlotConfig;
        tertiary: ModelSlotConfig;
    };
    mcpTools: string[];
    governance: {
        budgetUsd: number;
        requiresOversight: boolean;
    };
    ui: {
        directMessage: string;
        saving: boolean;
        themeColor: string;
        newRoleName: string;
        showPromote: boolean;
    };
    connectorConfigs: { type: string; uri: string }[];
}

export type Action =
    | { type: 'SET_MAIN_TAB'; payload: 'cognition' | 'memory' | 'governance' }
    | { type: 'SET_TAB'; payload: 'primary' | 'secondary' | 'tertiary' }
    | { type: 'UPDATE_IDENTITY'; field: 'name' | 'role'; value: string }
    | { type: 'UPDATE_SLOT'; slot: 'primary' | 'secondary' | 'tertiary'; field: 'model' | 'provider' | 'systemPrompt'; value: string }
    | { type: 'UPDATE_SLOT'; slot: 'primary' | 'secondary' | 'tertiary'; field: 'temperature'; value: number }
    | { type: 'UPDATE_SLOT'; slot: 'primary' | 'secondary' | 'tertiary'; field: 'skills' | 'workflows'; value: string[] }
    | { type: 'TOGGLE_SKILL'; slot: 'primary' | 'secondary' | 'tertiary'; kind: 'skills' | 'workflows'; value: string }
    | { type: 'RESET_ROLE'; role: string }
    | { type: 'UPDATE_GOVERNANCE'; field: 'budgetUsd'; value: number }
    | { type: 'UPDATE_GOVERNANCE'; field: 'requiresOversight'; value: boolean }
    | { type: 'UPDATE_VOICE'; field: 'voiceId' | 'voiceEngine' | 'sttEngine'; value: string }
    | { type: 'TOGGLE_MCP_TOOL'; value: string }
    | { type: 'ADD_CONNECTOR'; payload: { type: string; uri: string } }
    | { type: 'REMOVE_CONNECTOR'; uri: string }
    | { type: 'SET_UI'; field: 'directMessage' | 'saving' | 'themeColor' | 'newRoleName' | 'showPromote'; value: string | boolean };

export function configReducer(state: AgentConfigState, action: Action): AgentConfigState {
    switch (action.type) {
        case 'SET_MAIN_TAB':
            return { ...state, mainTab: action.payload };
        case 'SET_TAB':
            return { ...state, activeTab: action.payload };
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
            const currentList = state.slots[action.slot][action.kind as keyof ModelSlotConfig] as string[];
            const newList = currentList.includes(action.value as string)
                ? currentList.filter(item => item !== action.value)
                : [...currentList, action.value as string];
            return {
                ...state,
                slots: {
                    ...state.slots,
                    [action.slot]: {
                        ...state.slots[action.slot],
                        [action.kind]: newList
                    }
                }
            };
        }
        case 'RESET_ROLE': {
            const defaults = useRoleStore.getState().roles[action.role] || { skills: [], workflows: [] };
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
            const newList = state.mcpTools.includes(action.value)
                ? state.mcpTools.filter(t => t !== action.value)
                : [...state.mcpTools, action.value];
            return { ...state, mcpTools: newList };
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
            } as AgentConfigState;
        case 'ADD_CONNECTOR':
            return {
                ...state,
                connectorConfigs: [...state.connectorConfigs, action.payload]
            };
        case 'REMOVE_CONNECTOR':
            return {
                ...state,
                connectorConfigs: state.connectorConfigs.filter(c => c.uri !== action.uri)
            };
        default:
            return state;
    }
}
