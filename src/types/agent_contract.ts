/**
 * @docs ARCHITECTURE:Contracts
 * 
 * ### AI Assist Note
 * **Deprecated Context**: Backward compatibility layer for the legacy agent types.
 * Redirects to `src/contracts/agent/domain.ts`. 
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Circular imports or type mismatches if new contract fields are missing from this proxy.
 * - **Telemetry Link**: Not tracked (Static exports).
 * 
 * @deprecated Use imports from '@/contracts/agent' instead.
 * This file is maintained for backward compatibility during the contract consolidation.
 */

export type { 
    Agent_Status, 
    Agent_Voice_Engine, 
    Agent_Stt_Engine, 
    Agent_Metadata, 
    Agent_Model_Slot_Key, 
    Agent_Model_Slot_State,
    Agent_Connector_Config
} from '../contracts/agent';

// Metadata: [agent_contract]

// Metadata: [agent_contract]
