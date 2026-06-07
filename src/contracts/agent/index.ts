/**
 * @docs ARCHITECTURE:Contracts
 * 
 * ### AI Assist Note
 * **Agent Contract Barrel**: Single entry point for all Agent-related types.
 * Organized by layer: Wire (DTO), Domain (Model), Form (State).
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Circular dependencies if barrels are imported cross-layer (avoid importing barrel in sub-files).
 * - **Telemetry Link**: Not tracked (Index Barrel).
 */

export * from './shared';
export * from './wire';
export * from './domain';
export * from './form';

// Metadata: [index]

// Metadata: [index]
