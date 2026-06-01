/**
 * @docs ARCHITECTURE:UI-Services
 * 
 * ### AI Assist Note
 * **@docs ARCHITECTURE:Services**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[system_api_service]` in observability traces.
 */

/**
 * @docs ARCHITECTURE:Services
 * @docs API_REFERENCE:Endpoints
 *
 * Facade for system-level backend APIs. Domain-specific implementations live in
 * `src/services/system/*` so callers can keep using the stable
 * `system_api_service` import while the service layer stays small.
 */

import { benchmarks_api } from './system/benchmarks_api';
import { continuity_api } from './system/continuity_api';
import { docs_api } from './system/docs_api';
import { engine_api } from './system/engine_api';
import { infra_api } from './system/infra_api';
import { oversight_api } from './system/oversight_api';
import { workspace_api } from './system/workspace_api';

export const system_api_service = {
    ...engine_api,
    ...infra_api,
    ...benchmarks_api,
    ...continuity_api,
    ...oversight_api,
    ...docs_api,
    ...workspace_api
};

export type {
    Agent_Health,
    Audit_Entry,
    Benchmark_Record,
    Infra_Node,
    Provider_Test_Config,
    Quota_Details,
    Quotas,
    Scheduled_Job,
    Scheduled_Job_Run,
    Store_Model,
    Swarm_Node,
    Workflow_Entry,
    Workflow_Step,
    Workspace_Status
} from './system_api_types';

export type { Skill_Manifest } from './mission_api_service';

// Metadata: [system_api_service]

// Metadata: [system_api_service]
