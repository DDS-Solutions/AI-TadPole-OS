/**
 * @docs ARCHITECTURE:Contracts
 * 
 * ### AI Assist Note
 * **Form Layer**: UI-optimized state for the Agent Configuration form.
 * Decouples the flat domain model from the hierarchical tabbed-UI state.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Validation errors when mapping nested form fields back to the flat domain model in `agent_mappers`.
 * - **Telemetry Link**: Not tracked (Static Form Type).
 */

import type { 
    Agent_Model_Slot_Key, 
    Agent_Voice_Engine, 
    Agent_Stt_Engine, 
    Agent_Connector_Config 
} from './shared';

export interface Agent_Model_Slot_State {
    provider: string;
    model: string;
    temperature: number;
    system_prompt: string;
    reasoning_depth: number;
    act_threshold: number;
    skills: string[];
    workflows: string[];
}

export interface AgentFormState {
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

// Metadata: [form]

// Metadata: [form]
