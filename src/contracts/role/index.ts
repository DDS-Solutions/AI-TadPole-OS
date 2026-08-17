/**
 * @docs ARCHITECTURE:Contracts
 * 
 * ### AI Assist Note
 * **Role Barrel Export**: Re-exports domain role definitions and wire payloads.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Missing export in role domain or wire contracts.
 * - **Telemetry Link**: Search `[RoleIndex]` in logs.
 */

export * from './domain';
export * from './wire';
// Metadata: [RoleIndex]
