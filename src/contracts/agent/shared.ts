/**
 * @docs ARCHITECTURE:Contracts
 * 
 * ### AI Assist Note
 * **Shared Primitives**: Authoritative enums and unions for the Agent contract.
 * Prevents string-literal drift across Wire, Domain, and Form layers.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Status mismatch between frontend unions and backend `EngineAgent` strings in Rust.
 * - **Telemetry Link**: Not tracked (Static Primitives).
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

// Metadata: [shared]

// Metadata: [shared]
