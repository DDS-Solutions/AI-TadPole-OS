/**
 * @docs ARCHITECTURE:Services
 *
 * ### AI Assist Note
 * - Keep event contracts stable and validate cross-tab payloads before they enter application state.
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / event bus contracts
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: invalid payloads are ignored
 * - **Telemetry Targets**: none declared
 */

export type log_source = 'User' | 'System' | 'Agent';

/** Visual severity used for color-coding in the Terminal UI. */
export type log_severity = 'info' | 'success' | 'warning' | 'error';

/** A single event in the unified command timeline. */
export interface log_entry {
  /** Unique identifier (auto-generated). */
  id: string;
  /** When the event occurred (auto-generated). */
  timestamp: Date;
  /** Who produced this entry. */
  source: log_source;
  /** Human-readable message content. */
  text: string;
  /** Severity level for UI color-coding. */
  severity: log_severity;
  /** The originating agent's ID, if `source` is `'Agent'`. */
  agent_id?: string;
  /** The originating agent's friendly name, if available. */
  agent_name?: string;
  /** The associated mission (cluster) ID, if applicable. */
  mission_id?: string;
  /** RFC 9457 Error URI for machine-readable error handling. */
  type_id?: string;
  /** Flexible metadata for extended diagnostic display. */
  metadata?: Record<string, unknown>;
}

/** Unified Telemetry Message Wrapper */
export interface telemetry_message {
  topic: 'LOG' | 'TRACE' | 'PULSE' | 'OVERSIGHT' | 'SYNC_REQUEST' | 'SYNC_RESPONSE';
  payload: unknown;
  timestamp: number;
  sender_id: string;
}

export type Log_Origin = 'local' | 'remote' | 'replay';
export type Listener = (entry: log_entry, origin?: Log_Origin) => void;

export function validate_inbound_log(payload: unknown): log_entry | null {
  if (!payload || typeof payload !== 'object') return null;
  const entry = payload as Partial<log_entry>;
  const valid_sources: log_source[] = ['User', 'System', 'Agent'];
  const valid_severities: log_severity[] = ['info', 'success', 'warning', 'error'];

  if (!entry.id || typeof entry.id !== 'string') return null;
  if (!entry.source || !valid_sources.includes(entry.source as log_source)) return null;
  if (!entry.severity || !valid_severities.includes(entry.severity as log_severity)) return null;
  if (typeof entry.text !== 'string') return null;

  const text =
    entry.text.length > 50000 ? entry.text.substring(0, 50000) + '... [BUS TRUNCATED]' : entry.text;

  const metadata = entry.metadata
    ? (truncate_large_metadata(entry.metadata) as Record<string, unknown>)
    : undefined;

  return {
    id: entry.id,
    timestamp: (() => {
      const timestamp = entry.timestamp ? new Date(entry.timestamp) : new Date();
      return Number.isFinite(timestamp.getTime()) ? timestamp : new Date();
    })(),
    source: entry.source as log_source,
    text,
    severity: entry.severity as log_severity,
    agent_id: typeof entry.agent_id === 'string' ? entry.agent_id : undefined,
    agent_name: typeof entry.agent_name === 'string' ? entry.agent_name : undefined,
    mission_id: typeof entry.mission_id === 'string' ? entry.mission_id : undefined,
    type_id: typeof entry.type_id === 'string' ? entry.type_id : undefined,
    metadata,
  };
}

export function truncate_large_metadata(value: unknown, depth = 0): unknown {
  if (depth > 3) return '[MAX_DEPTH_REACHED]';
  const MAX_STRING_LEN = 10000;

  if (typeof value === 'string') {
    if (value.length > MAX_STRING_LEN) {
      return value.substring(0, MAX_STRING_LEN) + '... [TRUNCATED]';
    }
    return value;
  }

  if (Array.isArray(value)) {
    return value.map((item) => truncate_large_metadata(item, depth + 1));
  }

  if (typeof value === 'object' && value !== null) {
    if (value instanceof Date) {
      return new Date(value.getTime());
    }
    if (value instanceof RegExp) {
      return new RegExp(value);
    }

    const result = Object.create(null) as Record<string, unknown>;
    for (const [key, val] of Object.entries(value)) {
      if (key === '__proto__' || key === 'constructor' || key === 'prototype') {
        continue;
      }
      result[key] = truncate_large_metadata(val, depth + 1);
    }
    return { ...result };
  }

  return value;
}
