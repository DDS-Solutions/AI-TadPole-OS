/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **Barrel Module**: Unified export portal for the OS UI primitive layer. 
 * Facilitates streamlined component ingestion for high-level layouts and feature modules.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Circular dependency if a primitive imports a high-level UI component, or missing export during new component onboarding.
 * - **Telemetry Link**: Check this file if `import { ... } from './ui'` fails in feature modules.
 */

// Barrel export for all UI primitives
export * from './Confirm_Dialog';
export * from './Empty_State';
export * from './Page_Header';
export * from './Portal_Window';
export * from './Section_Header';
export * from './Status_Badge';
export * from './Tooltip';
export * from './Tw_Empty_State';
export * from './Header_Ticker';
export * from './Connection_Banner';

