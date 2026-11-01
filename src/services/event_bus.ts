/**
 * @docs ARCHITECTURE:Services
 *
 * ### AI Assist Note
 * - Coordinate event delivery and bounded local history while validating messages at the boundary.
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / event_bus
 * - **Primary Entrypoints**: event_bus, log_entry, telemetry_message
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: [event_bus]
 */

import { EventHistory } from './event_bus/history';
import {
  truncate_large_metadata as truncate_metadata,
  validate_inbound_log as validate_log_payload,
} from './event_bus/contracts';
import type {
  Listener,
  log_entry,
  log_severity,
  log_source,
  telemetry_message,
} from './event_bus/contracts';

export type { log_entry, telemetry_message, Log_Origin } from './event_bus/contracts';

class event_bus_service {
  private listeners: Listener[] = [];

  private history = new EventHistory();
  private channel: BroadcastChannel | null =
    typeof window !== 'undefined' && typeof BroadcastChannel !== 'undefined'
      ? new BroadcastChannel('tadpole-neural-hub')
      : null;
  private pending_sync_response: ReturnType<typeof setTimeout> | null = null;
  private trace_listeners: ((span: unknown) => void)[] = [];
  private pulse_listeners: ((pulse: unknown) => void)[] = [];
  private instance_id =
    typeof crypto !== 'undefined' && typeof crypto.getRandomValues === 'function'
      ? Array.from(crypto.getRandomValues(new Uint32Array(2)), (dec) => dec.toString(36)).join('-')
      : `${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`;

  constructor() {
    this.bind_channel_handler(this.channel);
    if (this.channel) {
      // Request initial state on startup
      setTimeout(() => this.request_sync(), 100);
    }
  }

  private bind_channel_handler(channel: BroadcastChannel | null): void {
    if (!channel) return;
    channel.onmessage = (event) => {
      try {
        const msg = event.data as telemetry_message;
        if (!msg || msg.sender_id === this.instance_id) return;

        switch (msg.topic) {
          case 'LOG': {
            const validated = this.validate_inbound_log(msg.payload);
            if (validated) {
              this.internal_emit(validated, false);
            }
            break;
          }
          case 'TRACE':
            this.internal_emit_trace(msg.payload, false);
            break;
          case 'PULSE':
            this.internal_emit_pulse(msg.payload, false);
            break;
          case 'SYNC_REQUEST':
            this.handle_sync_request();
            break;
          case 'SYNC_RESPONSE':
            // Suppress pending sync response if another tab has already replied
            if (this.pending_sync_response) {
              clearTimeout(this.pending_sync_response);
              this.pending_sync_response = null;
            }
            this.handle_sync_response(msg.payload);
            break;
        }
      } catch (error) {
        console.error('[event_bus] Error handling BroadcastChannel message:', error);
      }
    };
  }

  private validate_inbound_log(payload: unknown): log_entry | null {
    return validate_log_payload(payload);
  }

  private request_sync(): void {
    if (this.channel) {
      try {
        this.channel.postMessage({
          topic: 'SYNC_REQUEST',
          payload: null,
          timestamp: Date.now(),
          sender_id: this.instance_id,
        } as telemetry_message);
      } catch (error) {
        console.error('[event_bus] Failed to send sync request:', error);
      }
    }
  }

  private handle_sync_request(): void {
    // Only windows with history should respond
    if (!this.history.has_any()) return;

    // Cancel any existing pending sync response to avoid duplicate scheduling
    if (this.pending_sync_response) {
      clearTimeout(this.pending_sync_response);
    }

    // Apply a randomized delay (50ms - 200ms) to prevent broadcast storms.
    // If another tab broadcasts a SYNC_RESPONSE during this interval, this response is cancelled.
    // Use a simple non-cryptographic pseudo-random delay based on performance.now() to avoid Math.random()
    const jitter =
      typeof performance !== 'undefined'
        ? Math.floor(performance.now() * 1000) % 150
        : Date.now() % 150;
    this.pending_sync_response = setTimeout(() => {
      if (this.channel) {
        try {
          this.channel.postMessage({
            topic: 'SYNC_RESPONSE',
            payload: {
              logs: this.get_history().slice(-100), // Limit sync response payload size to last 100 entries to prevent DoS
            },
            timestamp: Date.now(),
            sender_id: this.instance_id,
          } as telemetry_message);
        } catch (error) {
          console.error('[event_bus] Failed to send sync response:', error);
        }
      }
      this.pending_sync_response = null;
    }, 50 + jitter);
  }

  private handle_sync_response(payload: unknown): void {
    if (!payload || typeof payload !== 'object') return;
    const p = payload as { logs?: log_entry[] };
    if (p.logs && Array.isArray(p.logs)) {
      // Limit receiver-side processing payload to last 200 logs to prevent memory/CPU pressure (EVT-010)
      const logs_to_process = p.logs.slice(-200);

      // Validate and filter out logs that are malformed or already in deduplication cache
      const new_logs: log_entry[] = [];
      for (const item of logs_to_process) {
        const validated = this.validate_inbound_log(item);
        if (validated && !this.history.has_processed(validated.id)) {
          new_logs.push(validated);
        }
      }
      if (new_logs.length === 0) return;

      // Sort logs chronologically by timestamp
      new_logs.sort((a, b) => new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime());

      const accepted_logs = this.history.append_batch(new_logs);
      if (accepted_logs.length === 0) return;

      // Deliver notifications to listeners asynchronously in chunks to prevent event loop starvation.
      // We only deliver the last 100 logs since the UI components maintain a maximum local window of 100 logs anyway.
      const logs_to_notify = accepted_logs.slice(-100);
      const chunk_size = 50;
      let index = 0;
      const deliver_chunk = () => {
        const limit = Math.min(index + chunk_size, logs_to_notify.length);
        for (; index < limit; index++) {
          const log = logs_to_notify[index];
          this.listeners.forEach((listener) => {
            try {
              listener(log, 'replay');
            } catch (error) {
              console.error('[event_bus] Error in listener during sync:', error);
            }
          });
        }
        if (index < logs_to_notify.length) {
          setTimeout(deliver_chunk, 0);
        }
      };
      setTimeout(deliver_chunk, 0);
    }
  }

  /** Subscribe to all future events. Returns an unsubscribe function. */
  subscribe_logs(listener: Listener): () => void {
    this.listeners.push(listener);
    return () => {
      this.listeners = this.listeners.filter((l) => l !== listener);
    };
  }

  subscribe_traces(listener: (span: unknown) => void): () => void {
    this.trace_listeners.push(listener);
    return () => {
      this.trace_listeners = this.trace_listeners.filter((l) => l !== listener);
    };
  }

  /** Emit an event to all subscribers. `id` and `timestamp` are auto-filled if not provided. */
  emit_log(entry: Omit<log_entry, 'id' | 'timestamp'> & { id?: string; timestamp?: Date }): void {
    const valid_sources: log_source[] = ['User', 'System', 'Agent'];
    const valid_severities: log_severity[] = ['info', 'success', 'warning', 'error'];

    if (!entry || typeof entry !== 'object') {
      throw new TypeError('[event_bus] emit_log requires a valid entry object.');
    }
    // Coerce unknown source/severity rather than throwing — a new backend enum value
    // should degrade one log line, not crash the entire ingestion path.
    let safe_source: log_source = 'System';
    if (valid_sources.includes(entry.source)) {
      safe_source = entry.source;
    } else {
      console.warn(`[event_bus] Unknown log source "${entry.source}" — coerced to 'System'.`);
    }

    let safe_severity: log_severity = 'info';
    if (valid_severities.includes(entry.severity)) {
      safe_severity = entry.severity;
    } else {
      console.warn(`[event_bus] Unknown log severity "${entry.severity}" — coerced to 'info'.`);
    }
    const raw_text =
      typeof entry.text === 'string' ? entry.text : entry.text != null ? String(entry.text) : '';

    // PERFORMANCE: Truncate extremely large text payloads in the core bus to prevent memory pressure
    // Max 50k characters for the bus (UI components can truncate further if needed)
    const text =
      raw_text.length > 50000 ? raw_text.substring(0, 50000) + '... [BUS TRUNCATED]' : raw_text;

    // Truncate large strings in metadata as well
    const metadata = entry.metadata
      ? (this.truncate_large_metadata(entry.metadata) as Record<string, unknown>)
      : undefined;

    const full_entry: log_entry = {
      id: entry.id || this.generate_secure_id(),
      timestamp: entry.timestamp || new Date(),
      source: safe_source,
      text,
      severity: safe_severity,
      agent_id: entry.agent_id,
      agent_name: entry.agent_name,
      mission_id: entry.mission_id,
      type_id: entry.type_id,
      metadata,
    };
    this.internal_emit(full_entry, true);
  }

  emit_trace(span: unknown): void {
    this.internal_emit_trace(span, true);
  }

  emit_pulse(pulse: unknown): void {
    this.internal_emit_pulse(pulse, true);
  }

  private generate_secure_id(): string {
    if (typeof crypto !== 'undefined') {
      if (typeof crypto.randomUUID === 'function') {
        return crypto.randomUUID();
      }
      if (typeof crypto.getRandomValues === 'function') {
        const array = new Uint32Array(4);
        crypto.getRandomValues(array);
        return Array.from(array, (dec) => dec.toString(36)).join('-');
      }
    }
    // Fallback for environments without crypto (e.g. legacy test setups)
    const timestamp = Date.now().toString(36);
    const counter =
      typeof performance !== 'undefined' ? Math.floor(performance.now() * 1000) % 1000000 : 0;
    return `${timestamp}-${counter.toString(36)}`;
  }

  /**
   * Recursively truncates large strings in metadata objects to prevent memory bloat,
   * while preserving native constructors (Date, RegExp), handling arrays, and preventing prototype pollution.
   */

  private truncate_large_metadata(value: unknown, depth = 0): unknown {
    return truncate_metadata(value, depth);
  }

  private internal_emit(full_entry: log_entry, broadcast: boolean): void {
    if (!this.history.append(full_entry)) return;

    this.listeners.forEach((listener) => {
      try {
        listener(full_entry, broadcast ? 'local' : 'remote');
      } catch (error) {
        console.error('[event_bus] Error in listener:', error);
      }
    });

    if (broadcast && this.channel) {
      try {
        this.channel.postMessage({
          topic: 'LOG',
          payload: full_entry,
          timestamp: Date.now(),
          sender_id: this.instance_id,
        } as telemetry_message);
      } catch (error) {
        console.error(
          '[event_bus] Failed to broadcast log over channel, attempting fallback:',
          error,
        );
        try {
          // Fallback: strip metadata to prevent DataCloneError
          const fallback_entry = { ...full_entry, metadata: undefined };
          this.channel.postMessage({
            topic: 'LOG',
            payload: fallback_entry,
            timestamp: Date.now(),
            sender_id: this.instance_id,
          } as telemetry_message);
        } catch (fallback_error) {
          console.error('[event_bus] Critical broadcast fallback failure:', fallback_error);
        }
      }
    }
  }

  private internal_emit_trace(span: unknown, broadcast: boolean): void {
    this.trace_listeners.forEach((l) => {
      try {
        l(span);
      } catch (error) {
        console.error('[event_bus] Error in trace listener:', error);
      }
    });
    if (broadcast && this.channel) {
      try {
        this.channel.postMessage({
          topic: 'TRACE',
          payload: span,
          timestamp: Date.now(),
          sender_id: this.instance_id,
        } as telemetry_message);
      } catch (error) {
        console.error('[event_bus] Failed to broadcast trace:', error);
      }
    }
  }

  private internal_emit_pulse(pulse: unknown, broadcast: boolean): void {
    this.pulse_listeners.forEach((l) => {
      try {
        l(pulse);
      } catch (error) {
        console.error('[event_bus] Error in pulse listener:', error);
      }
    });
    if (broadcast && this.channel) {
      try {
        this.channel.postMessage({
          topic: 'PULSE',
          payload: pulse,
          timestamp: Date.now(),
          sender_id: this.instance_id,
        } as telemetry_message);
      } catch (error) {
        console.error('[event_bus] Failed to broadcast pulse:', error);
      }
    }
  }

  /** Returns a chronologically ordered copy of all stored history. */
  get_history(): log_entry[] {
    return this.history.get_history();
  }

  clear_history(): void {
    this.history.clear();
    if (this.pending_sync_response) {
      clearTimeout(this.pending_sync_response);
      this.pending_sync_response = null;
    }
  }

  /** Full teardown: clears history AND removes all subscribers. Use on unmount. */
  destroy(): void {
    this.clear_history();
    this.listeners = [];
    this.trace_listeners = [];
    this.pulse_listeners = [];
    if (this.channel) {
      try {
        this.channel.close();
      } catch (error) {
        console.error('[event_bus] Error closing channel during destroy:', error);
      }
      this.channel = null;
    }
  }

  /** Reset state and re-initialize transport channel. Primarily used for test suite isolation. */
  reset(): void {
    this.clear_history();
    this.listeners = [];
    this.trace_listeners = [];
    this.pulse_listeners = [];
    if (this.channel) {
      try {
        this.channel.close();
      } catch (error) {
        console.error('[event_bus] Error closing channel during reset:', error);
      }
    }
    // Re-initialize channel for subsequent test/lifecycle usage
    this.channel =
      typeof window !== 'undefined' && typeof BroadcastChannel !== 'undefined'
        ? new BroadcastChannel('tadpole-neural-hub')
        : null;

    this.bind_channel_handler(this.channel);
  }
}

/** Singleton instance shared across the entire application. */
export const event_bus = new event_bus_service();
