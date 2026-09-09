/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Agent-Config / index
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

export * from './AgentConfigHeader';
export * from './CognitionSection';
export * from './VoiceSection';
export * from './GovernanceSection';
export * from './MemorySection';
export * from './DirectMessageConsole';
export * from './useAgentConfig';
export * from './useAgentMemory';
