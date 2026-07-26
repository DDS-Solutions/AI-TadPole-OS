/**
 * @docs ARCHITECTURE:Types
 * 
 * ### AI Assist Note
 * - **Purpose**: Declares global types and environment variables for the frontend build system.
 * 
 * ### 🔍 Debugging & Observability
 * - **Telemetry Link**: Essential for type-checking during compile time.
 */
/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_NEURAL_TOKEN?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}

// Metadata: [vite_env]
