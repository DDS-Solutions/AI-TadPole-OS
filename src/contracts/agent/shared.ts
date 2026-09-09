/**
 * @docs ARCHITECTURE:Contracts
 *
 * ### AI Context Alignment
 * - **Subsystem**: System Core / shared
 * - **Primary Entrypoints**: `Agent_Connector_Config`, `Agent_Status`, `Agent_Model_Slot_Key`, `Agent_Voice_Engine`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

export type Agent_Status = 'idle' | 'active' | 'suspended' | 'failed' | 'throttled' | 'offline' | 'thinking' | 'coding' | 'speaking';

export type Agent_Model_Slot_Key = 'primary' | 'secondary' | 'tertiary';

export type Agent_Voice_Engine = 'browser' | 'openai' | 'groq' | 'piper' | 'gemini-live';

export type Agent_Stt_Engine = 'groq' | 'whisper';

export type Agent_Metadata = Record<string, unknown>;

export interface Agent_Connector_Config {
    type: string;
    uri: string;
}

export type Department = 
    | 'Executive' 
    | 'Engineering' 
    | 'Marketing' 
    | 'Sales' 
    | 'Product' 
    | 'Operations' 
    | 'Quality Assurance' 
    | 'Design' 
    | 'Research' 
    | 'Support' 
    | 'Intelligence' 
    | 'Finance' 
    | 'Growth' 
    | 'Success';
