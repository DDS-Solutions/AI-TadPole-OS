/**
 * @docs ARCHITECTURE:Services
 * @docs API_REFERENCE:Endpoints
 *
 * ### AI Assist Note
 * **Stable Barrel**: Re-exports the full Agent API surface from `./agent/`.
 * Implementation lives in `src/services/agent/` — edit there, not here.
 *
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: See individual sub-service files in `./agent/`.
 * - **Telemetry Link**: Search `[AgentAPI]` in backend tracing.
 */

export * from './agent/index';
