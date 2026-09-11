/**
 * @docs ARCHITECTURE:Contracts
 *
 * ### AI Context Alignment
 * - **Subsystem**: System Core / form
 * - **Primary Entrypoints**: `Agent_Model_Slot_State`, `AgentFormState`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
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
        shadows_human_id?: string;
        economic_zone?: string;
        daily_spend_limit?: number;
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
