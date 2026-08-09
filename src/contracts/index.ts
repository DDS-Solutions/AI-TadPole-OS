/**
 * @docs ARCHITECTURE:Contracts
 * 
 * ### AI Assist Note
 * **Central Contracts Barrel**: Authoritative export aggregator for domain contracts,
 * skill definitions, agent types, and role specifications.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Missing re-export or circular import dependency in contract modules.
 * - **Telemetry Link**: Search `[ContractsIndex]` in build and tracing logs.
 */

// Central Barrel Exports [ContractsIndex]
export * from './skills';
export * from './agent';
export * from './role';
