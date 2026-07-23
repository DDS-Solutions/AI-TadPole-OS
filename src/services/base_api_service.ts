/**
 * @docs ARCHITECTURE:UI-Services
 * 
 * ### AI Assist Note
 * **@docs ARCHITECTURE:Services**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[base_api_service]` in observability traces.
 */

/**
 * @docs ARCHITECTURE:Services
 * 
 * ### AI Assist Note
 * **Infrastructure Service**: Standardized HTTP client and OpenTelemetry pipeline.
 * Gutted monolith: implementation decomposed to src/api/; this file serves as a
 * backward-compatibility barrel shim.
 */

export * from '../api';

// Metadata: [base_api_service]
