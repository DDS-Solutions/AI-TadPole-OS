/**
 * @docs ARCHITECTURE:UI-Services
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / events
 * - **Primary Entrypoints**: `Agent_Update_Event`, `Engine_Health_Event`, `Handoff_Event`, `Mcp_Pulse_Event`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import type { Agent, Trace_Span } from '../../../types';

/** Payload for agent update/status events from the WebSocket. */
export interface Agent_Update_Event {
    type: 'agent:create' | 'agent:update' | 'agent:status' | 'engine:ui_invalidate';
    agent_id?: string;
    agentId?: string;
    status?: string;
    data?: Record<string, unknown> | Partial<Agent>;
    resource?: 'agents' | 'missions' | 'system';
    id?: string;
    source_id?: string;
}

/** Payload for engine health broadcast events. */
export interface Engine_Health_Event {
    type: 'engine:health';
    uptime?: number;
    agent_count?: number;
    active_missions?: number;
    active_agents?: number;
    max_depth?: number;
    tpm?: number;
    recruit_count?: number;
    cpu?: number;
    memory?: number;
    latency?: number;
    [key: string]: unknown;
}

/** Payload for inter-cluster handoff events. */
export interface Handoff_Event {
    type: 'agent:handoff';
    from_cluster: string;
    to_cluster: string;
    agent_id: string;
    payload?: Record<string, unknown>;
}

/** Payload for MCP tool pulse events. */
export interface Mcp_Pulse_Event {
    type: 'engine:mcp_pulse';
    tool: string;
    status: 'success' | 'error';
    latency: number;
}

export interface Socket_Log_Event {
    type: 'log' | 'thought';
    id?: string;
    request_id?: string;
    requestId?: string;
    agent_id?: string;
    agent_name?: string;
    text?: string;
    message?: string;
    level?: string;
    message_id?: string;
    messageId?: string;
    [key: string]: unknown;
}

export interface Socket_Agent_Message_Event {
    type: 'agent:message';
    id?: string;
    message_id?: string;
    messageId?: string;
    agent_id?: string;
    agent_name?: string;
    text?: string;
    message?: string;
    content?: string;
    [key: string]: unknown;
}

export interface Socket_Trace_Span_Event {
    type: 'trace:span';
    span: Trace_Span;
}

export interface Socket_Trace_Span_Update_Event {
    type: 'trace:span_update';
    span_id?: string;
    spanId?: string;
    update: Partial<Trace_Span>;
}

export interface Socket_Scheduled_Job_Complete_Event {
    type: 'engine:scheduled_job_complete';
    job_name: string;
    cost_usd?: number;
    status?: string;
}

export interface Socket_Auth_Ok_Event {
    type: 'auth_ok';
}

export interface Socket_Auth_Error_Event {
    type: 'auth_error';
    message?: string;
}

export interface Socket_Heartbeat_Event {
    type: 'heartbeat';
    [key: string]: unknown;
}

export interface Socket_Pong_Event {
    type: 'pong';
    [key: string]: unknown;
}

export type Incoming_Socket_Message =
    | Socket_Log_Event
    | Socket_Agent_Message_Event
    | Agent_Update_Event
    | Engine_Health_Event
    | Handoff_Event
    | Mcp_Pulse_Event
    | Socket_Trace_Span_Event
    | Socket_Trace_Span_Update_Event
    | Socket_Scheduled_Job_Complete_Event
    | Socket_Auth_Ok_Event
    | Socket_Auth_Error_Event
    | Socket_Heartbeat_Event
    | Socket_Pong_Event;
