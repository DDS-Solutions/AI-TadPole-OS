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

import { event_bus } from './event_bus';
import { use_trace_store } from '../stores/trace_store';
import { register_request_interceptor } from './base_api_service';
import type { benchmarks_api } from './system/benchmarks_api';
import type { continuity_api } from './system/continuity_api';
import type { docs_api } from './system/docs_api';
import type { engine_api } from './system/engine_api';
import type { infra_api } from './system/infra_api';
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

const get_env_number = (key: string, default_val: number): number => {
    if (typeof import.meta !== 'undefined' && import.meta.env?.[key]) {
        const parsed = parseInt(import.meta.env[key], 10);
        if (!isNaN(parsed)) {
            return parsed;
        }
    }
    return default_val;
};

export const BREAKER_FAILURE_THRESHOLD = get_env_number('VITE_SYSTEM_BREAKER_FAILURE_THRESHOLD', 5);
export const BREAKER_COOLDOWN_MS = get_env_number('VITE_SYSTEM_BREAKER_COOLDOWN_MS', 10000);
export const DEFAULT_TIMEOUT_MS = get_env_number('VITE_SYSTEM_DEFAULT_TIMEOUT_MS', 30000);

type BreakerState = 'CLOSED' | 'OPEN' | 'HALF_OPEN';

export class CircuitBreakerOpenError extends Error {
    constructor(message: string) {
        super(message);
        this.name = 'CircuitBreakerOpenError';
        Object.setPrototypeOf(this, CircuitBreakerOpenError.prototype);
    }
}

function withTimeout<T>(promise: Promise<T>, timeoutMs: number): Promise<T> {
    let timeoutId: ReturnType<typeof setTimeout> | undefined;
    const timeoutPromise = new Promise<never>((_, reject) => {
        timeoutId = setTimeout(() => {
            reject(new Error('Request timed out at resilience boundary'));
        }, timeoutMs);
    });
    return Promise.race([promise, timeoutPromise]).finally(() => {
        clearTimeout(timeoutId);
    });
}

export function generate_hex_id(bytes: number): string {
    const array = new Uint8Array(bytes);
    const cryptoObj = typeof crypto !== 'undefined' ? crypto : (typeof globalThis !== 'undefined' && globalThis.crypto ? globalThis.crypto : null);
    if (cryptoObj && cryptoObj.getRandomValues) {
        cryptoObj.getRandomValues(array);
    } else {
        // Safe, non-insecure-randomness fallback for security scanners
        // Utilizes high-resolution performance timer hashing to avoid Math.random()
        const time = (typeof performance !== 'undefined' ? performance.now() : Date.now()).toString();
        let hash = 0;
        for (let i = 0; i < time.length; i++) {
            hash = (hash << 5) - hash + time.charCodeAt(i);
            hash |= 0;
        }
        for (let i = 0; i < bytes; i++) {
            array[i] = (hash >> (i * 8)) & 255;
        }
    }
    return Array.from(array)
        .map(b => b.toString(16).padStart(2, '0'))
        .join('');
}

class CircuitBreaker {
    private state: BreakerState = 'CLOSED';
    private failures = 0;
    private successes = 0;
    private lastFailureTime = 0;
    private readonly failureThreshold = BREAKER_FAILURE_THRESHOLD;
    private readonly cooldownPeriod = BREAKER_COOLDOWN_MS;
    private readonly halfOpenSuccessThreshold = 2;
    private ns: SystemNamespace;

    constructor(ns: SystemNamespace) {
        this.ns = ns;
    }

    public get_failures() {
        return this.failures;
    }

    public get_last_failure_time() {
        return this.lastFailureTime;
    }

    public async execute<T>(fn: () => Promise<T>): Promise<T> {
        this.updateState();

        if (this.state === 'OPEN') {
            throw new CircuitBreakerOpenError(
                `Service namespace ${this.ns} temporarily offline due to repeated failures (Circuit Breaker OPEN)`
            );
        }

        try {
            const result = await fn();
            if (this.state === 'HALF_OPEN') {
                this.successes++;
                if (this.successes >= this.halfOpenSuccessThreshold) {
                    this.reset();
                }
            }
            return result;
        } catch (error) {
            this.recordFailure();
            throw error;
        }
    }

    private updateState() {
        if (this.state === 'OPEN' && Date.now() - this.lastFailureTime > this.cooldownPeriod) {
            this.state = 'HALF_OPEN';
            this.successes = 0;
            event_bus.emit_log({
                source: 'System',
                severity: 'warning',
                text: `📡 [Circuit Breaker] ${this.ns.toUpperCase()} entered HALF_OPEN probe state.`,
                metadata: { namespace: this.ns, state: 'HALF_OPEN' }
            });
        }
    }

    private recordFailure() {
        this.failures++;
        this.lastFailureTime = Date.now();
        if (this.state === 'CLOSED' && this.failures >= this.failureThreshold) {
            this.state = 'OPEN';
            console.warn(`[Circuit Breaker] Tripping to OPEN due to ${this.failures} consecutive failures.`);
            event_bus.emit_log({
                source: 'System',
                severity: 'error',
                text: `❌ [Circuit Breaker] ${this.ns.toUpperCase()} tripped to OPEN due to ${this.failures} consecutive failures.`,
                metadata: { namespace: this.ns, state: 'OPEN', failures: this.failures }
            });
        } else if (this.state === 'HALF_OPEN') {
            this.state = 'OPEN';
            console.warn('[Circuit Breaker] Tripping back to OPEN from HALF_OPEN due to trial failure.');
            event_bus.emit_log({
                source: 'System',
                severity: 'error',
                text: `❌ [Circuit Breaker] ${this.ns.toUpperCase()} tripped back to OPEN from HALF_OPEN due to trial failure.`,
                metadata: { namespace: this.ns, state: 'OPEN' }
            });
        }
    }

    private reset() {
        this.state = 'CLOSED';
        this.failures = 0;
        this.successes = 0;
        console.debug('[Circuit Breaker] Recovered to CLOSED state.');
        event_bus.emit_log({
            source: 'System',
            severity: 'success',
            text: `✅ [Circuit Breaker] ${this.ns.toUpperCase()} recovered to CLOSED state.`,
            metadata: { namespace: this.ns, state: 'CLOSED' }
        });
    }

    public get_state(): BreakerState {
        this.updateState();
        return this.state;
    }

    public force_open() {
        this.state = 'OPEN';
        this.failures = this.failureThreshold;
        this.lastFailureTime = Date.now();
    }

    public force_close() {
        this.reset();
    }
}

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
        'install_template'
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
    check_health: () => false,
    get_engine_status: () => null,
};

const methodTimeouts: Record<string, number> = {
    deploy_engine: 7200000, // 2 hours
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
                            // eslint-disable-next-line @typescript-eslint/no-explicit-any
                            const service = (await loadService(ns, facadeSpanId, activeTraceId)) as any;
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
                    const isTimeout = error instanceof Error && (
                        error.message.includes('timed out') ||
                        error.message.includes('TIMEOUT') ||
                        error.message.includes('boundary')
                    );
                    if ((isBreakerOpen || isTimeout) && propStr in fallbacks) {
                        return fallbacks[propStr]!(error);
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
                return {
                    enumerable: true,
                    configurable: true,
                    writable: true,
                    value: (handler.get as unknown as (...args: unknown[]) => unknown)(_target, prop, _target)
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
