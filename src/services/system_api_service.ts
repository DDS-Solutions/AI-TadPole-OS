/**
 * @docs ARCHITECTURE:UI-Services
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / system_api_service
 * - **Primary Entrypoints**: `invalidate_namespace`, `get_circuit_breakers`, `get_system_health`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: `[Proxy]`
 */

import { use_trace_store } from '../stores/trace_store';
import { register_request_interceptor } from './base_api_service';
import type { benchmarks_api } from './system/benchmarks_api';
import type { continuity_api } from './system/continuity_api';
import type { docs_api } from './system/docs_api';
import type { engine_api } from './system/engine_api';
import type { infra_api } from './system/infra_api';
import { DEPLOY_TIMEOUT } from './base_api_service';
import type { oversight_api } from './system/oversight_api';
import type { workspace_api } from './system/workspace_api';

export const NAMESPACES = ['engine', 'infra', 'benchmarks', 'continuity', 'oversight', 'docs', 'workspace'] as const;
export type SystemNamespace = typeof NAMESPACES[number];

export interface SystemApiService {
    readonly engine: typeof engine_api;
    readonly infra: typeof infra_api;
    readonly benchmarks: typeof benchmarks_api;
    readonly continuity: typeof continuity_api;
    readonly oversight: typeof oversight_api;
    readonly docs: typeof docs_api;
    readonly workspace: typeof workspace_api;
}

import {
    CircuitBreaker,
    CircuitBreakerOpenError,
    TimeoutError,
    withTimeout,
    BREAKER_FAILURE_THRESHOLD,
    BREAKER_COOLDOWN_MS,
    DEFAULT_TIMEOUT_MS,
    type BreakerState
} from './resilience/circuit_breaker';
import { generate_hex_id } from './resilience/hex_utils';

export {
    CircuitBreaker,
    CircuitBreakerOpenError,
    TimeoutError,
    BREAKER_FAILURE_THRESHOLD,
    BREAKER_COOLDOWN_MS,
    DEFAULT_TIMEOUT_MS,
    generate_hex_id,
    type BreakerState
};


const loaders = {
    engine: () => import('./system/engine_api').then(m => m.engine_api),
    infra: () => import('./system/infra_api').then(m => m.infra_api),
    benchmarks: () => import('./system/benchmarks_api').then(m => m.benchmarks_api),
    continuity: () => import('./system/continuity_api').then(m => m.continuity_api),
    oversight: () => import('./system/oversight_api').then(m => m.oversight_api),
    docs: () => import('./system/docs_api').then(m => m.docs_api),
    workspace: () => import('./system/workspace_api').then(m => m.workspace_api)
} as const;

export const NAMESPACE_KEYS: Record<SystemNamespace, readonly string[]> = {
    engine: [
        'get_engine_status',
        'check_health',
        'deploy_engine',
        'speak',
        'kill_agents',
        'shutdown_engine',
        'transcribe',
        'install_template',
        'import_template',
        'get_installed_templates',
        'uninstall_template',
        'update_environment'
    ],
    infra: [
        'test_provider',
        'get_nodes',
        'discover_nodes',
        'get_providers',
        'update_provider',
        'delete_provider',
        'sync_provider_models',
        'update_model',
        'delete_model',
        'get_models',
        'get_model_catalog',
        'pull_model'
    ],
    benchmarks: [
        'get_benchmarks',
        'run_benchmark'
    ],
    continuity: [
        'get_scheduled_jobs',
        'create_scheduled_job',
        'update_scheduled_job',
        'delete_scheduled_job',
        'get_scheduled_job_runs',
        'list_continuity_workflows',
        'create_continuity_workflows',
        'add_continuity_workflows_step',
        'delete_continuity_workflows',
        'trigger_scheduled_job',
        'get_workflow_run_steps'
    ],
    oversight: [
        'get_pending_oversight',
        'get_oversight_ledger',
        'decide_oversight',
        'get_security_quotas',
        'update_security_quota',
        'get_mission_quotas',
        'update_mission_quota',
        'get_audit_trail',
        'get_agent_health',
        'get_integrity_status',
        'get_security_snapshot',
        'get_governance_settings',
        'update_governance_settings'
    ],
    docs: [
        'get_knowledge_docs',
        'get_knowledge_doc',
        'get_operations_manual'
    ],
    workspace: [
        'get_workspaces_status',
        'get_workspace_files'
    ]
};

const fallbacks: Partial<Record<string, (error: unknown) => unknown>> = {
    'engine.check_health': () => false,
    'engine.get_engine_status': () => null,
};

const methodTimeouts: Record<string, number> = {
    deploy_engine: DEPLOY_TIMEOUT,
};

export const _cache_for_testing: { [key in SystemNamespace]?: Promise<unknown> } = {};
const breakers: { [key in SystemNamespace]?: CircuitBreaker } = {};

export function invalidate_namespace(ns: SystemNamespace) {
    delete _cache_for_testing[ns];
}

export function get_circuit_breakers(): Record<SystemNamespace, CircuitBreaker> {
    for (const ns of NAMESPACES) {
        if (!breakers[ns]) {
            breakers[ns] = new CircuitBreaker(ns);
        }
    }
    return breakers as Record<SystemNamespace, CircuitBreaker>;
}

function getBreaker(ns: SystemNamespace): CircuitBreaker {
    let breaker = breakers[ns];
    if (!breaker) {
        breaker = new CircuitBreaker(ns);
        breakers[ns] = breaker;
    }
    return breaker;
}

async function loadService(ns: SystemNamespace, parentSpanId?: string, traceId?: string) {
    if (!_cache_for_testing[ns]) {
        const spanId = generate_hex_id(8);
        const activeTraceId = traceId || use_trace_store.getState().active_trace_id || generate_hex_id(16);

        use_trace_store.getState().add_span({
            id: spanId,
            trace_id: activeTraceId,
            parent_id: parentSpanId,
            name: `system_api: load_service (${ns})`,
            agent_id: 'system',
            mission_id: 'system',
            start_time: Date.now(),
            status: 'running',
            attributes: { namespace: ns }
        });

        _cache_for_testing[ns] = loaders[ns]().then(service => {
            use_trace_store.getState().update_span(spanId, {
                end_time: Date.now(),
                status: 'success'
            });
            return service;
        }).catch(err => {
            delete _cache_for_testing[ns]; // Clear cache on load failure so subsequent calls can retry
            use_trace_store.getState().update_span(spanId, {
                end_time: Date.now(),
                status: 'error',
                attributes: {
                    error: err instanceof Error ? err.message : String(err)
                }
            });
            throw err;
        });
    }
    return _cache_for_testing[ns]!;
}

function createNamespaceProxy<T extends SystemNamespace>(ns: T): SystemApiService[T] {
    const handler: ProxyHandler<Record<string, unknown>> = {
        get(_target, prop) {
            if (typeof prop === 'symbol') {
                return undefined;
            }

            const propStr = String(prop);
            if (!NAMESPACE_KEYS[ns].includes(propStr)) {
                if (propStr.startsWith('__') || propStr === 'then' || propStr === 'toJSON') {
                    return undefined;
                }
                console.warn(`[Proxy] Accessed unknown property "${propStr}" on namespace "${ns}"`);
            }

            return async (...args: unknown[]) => {
                const breaker = getBreaker(ns);
                const timeoutMs = methodTimeouts[propStr] || DEFAULT_TIMEOUT_MS;
                
                const activeTraceId = use_trace_store.getState().active_trace_id || generate_hex_id(16);
                const facadeSpanId = generate_hex_id(8);

                use_trace_store.getState().add_span({
                    id: facadeSpanId,
                    trace_id: activeTraceId,
                    name: `system_api: ${ns}.${propStr}`,
                    agent_id: 'system',
                    mission_id: 'system',
                    start_time: Date.now(),
                    status: 'running',
                    attributes: {
                        namespace: ns,
                        method: propStr,
                    }
                });

                const breakerSpanId = generate_hex_id(8);
                use_trace_store.getState().add_span({
                    id: breakerSpanId,
                    trace_id: activeTraceId,
                    parent_id: facadeSpanId,
                    name: `circuit_breaker: execute (${ns})`,
                    agent_id: 'system',
                    mission_id: 'system',
                    start_time: Date.now(),
                    status: 'running',
                    attributes: {
                        namespace: ns,
                        breaker_state: breaker.get_state()
                    }
                });

                try {
                    const result = await withTimeout(
                        breaker.execute(async () => {
                            const service = (await loadService(ns, facadeSpanId, activeTraceId)) as Record<string, (...args: unknown[]) => unknown>;
                            const method = service[propStr];
                            if (typeof method !== 'function') {
                                throw new TypeError(`Method ${propStr} is not a function on service ${ns}`);
                            }
                            return method.apply(service, args);
                        }),
                        timeoutMs
                    );

                    use_trace_store.getState().update_span(breakerSpanId, {
                        end_time: Date.now(),
                        status: 'success',
                        attributes: {
                            breaker_state_after: breaker.get_state()
                        }
                    });

                    use_trace_store.getState().update_span(facadeSpanId, {
                        end_time: Date.now(),
                        status: 'success'
                    });

                    return result;
                } catch (error) {
                    const errorMsg = error instanceof Error ? error.message : String(error);

                    use_trace_store.getState().update_span(breakerSpanId, {
                        end_time: Date.now(),
                        status: 'error',
                        attributes: {
                            error: errorMsg,
                            breaker_state_after: breaker.get_state()
                        }
                    });

                    use_trace_store.getState().update_span(facadeSpanId, {
                        end_time: Date.now(),
                        status: 'error',
                        attributes: {
                            error: errorMsg
                        }
                    });

                    const isBreakerOpen = error instanceof CircuitBreakerOpenError;
                    const isTimeout = error instanceof TimeoutError || (error instanceof Error && (
                        error.message.includes('timed out') ||
                        error.message.includes('TIMEOUT') ||
                        error.message.includes('boundary')
                    ));
                    const fallbackKey = `${ns}.${propStr}`;
                    if ((isBreakerOpen || isTimeout) && fallbackKey in fallbacks) {
                        return fallbacks[fallbackKey]!(error);
                    }
                    throw error;
                }
            };
        },

        has(_target, prop) {
            const propStr = String(prop);
            return NAMESPACE_KEYS[ns].includes(propStr);
        },

        ownKeys() {
            return [...NAMESPACE_KEYS[ns]];
        },

        getOwnPropertyDescriptor(_target, prop) {
            const propStr = String(prop);
            if (NAMESPACE_KEYS[ns].includes(propStr)) {
                // Materialize the async method function so that `Object.getOwnPropertyDescriptor`
                // returns a proper Function value for tooling/reflection without going through
                // the fragile `handler.get` self-cast pattern. The facade is Object.freeze()'d
                // so `writable: true` here is never exercised by callers.
                const method = handler.get!({} as Record<string, unknown>, propStr, {});
                return {
                    enumerable: true,
                    configurable: true,
                    writable: true,
                    value: method
                };
            }
            return undefined;
        },


        set() {
            throw new Error(`Facade namespace "${ns}" is immutable`);
        },

        defineProperty() {
            throw new Error(`Facade namespace "${ns}" is immutable`);
        },

        deleteProperty() {
            throw new Error(`Facade namespace "${ns}" is immutable`);
        }
    };
    return new Proxy({}, handler) as unknown as SystemApiService[T];
}

const facade: SystemApiService = {
    engine: createNamespaceProxy('engine'),
    infra: createNamespaceProxy('infra'),
    benchmarks: createNamespaceProxy('benchmarks'),
    continuity: createNamespaceProxy('continuity'),
    oversight: createNamespaceProxy('oversight'),
    docs: createNamespaceProxy('docs'),
    workspace: createNamespaceProxy('workspace')
};

export const system_api_service = Object.freeze(facade);

export function get_system_health() {
    const summary: Record<string, unknown> = {};
    for (const ns of NAMESPACES) {
        const breaker = getBreaker(ns);
        summary[ns] = {
            state: breaker.get_state(),
            failures: breaker.get_failures(),
            last_failure_time: breaker.get_last_failure_time()
        };
    }
    return summary;
}

// Register virtual health endpoint interceptor
if (typeof register_request_interceptor === 'function') {
    register_request_interceptor((path) => {
        if (path === '/health/system' || path === 'health/system') {
            return Promise.resolve(get_system_health());
        }
        return null;
    });
}

export type {
    Agent_Health,
    Audit_Entry,
    Benchmark_Record,
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
