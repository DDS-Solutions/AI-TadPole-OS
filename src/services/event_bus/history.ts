/**
 * @docs ARCHITECTURE:Services
 *
 * ### AI Assist Note
 * - Keep event history bounded and deduplicate IDs across single events and sync batches.
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / event history
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import type { log_entry } from './contracts';

const BUFFER_SIZE = 1000;
const MAX_ID_CACHE = 500;

export class EventHistory {
  private ring: (log_entry | null)[] = new Array(BUFFER_SIZE).fill(null);
  private head = 0;
  private count = 0;
  private processed_ids = new Set<string>();

  has_any(): boolean {
    return this.count > 0 || this.processed_ids.size > 0;
  }

  has_processed(id: string): boolean {
    return this.processed_ids.has(id);
  }

  append(entry: log_entry): boolean {
    if (this.processed_ids.has(entry.id)) return false;
    this.store(entry);
    return true;
  }

  append_batch(entries: readonly log_entry[]): log_entry[] {
    const batch_ids = new Set<string>();
    const accepted: log_entry[] = [];

    for (const entry of entries) {
      if (batch_ids.has(entry.id) || this.processed_ids.has(entry.id)) continue;
      batch_ids.add(entry.id);
      this.store(entry);
      accepted.push(entry);
    }

    return accepted;
  }

  get_history(): log_entry[] {
    if (this.count === 0) return [];
    const result: log_entry[] = [];
    const start = this.count < BUFFER_SIZE ? 0 : this.head;
    for (let i = 0; i < this.count; i++) {
      const idx = (start + i) % BUFFER_SIZE;
      if (this.ring[idx]) result.push(this.ring[idx]!);
    }
    return result;
  }

  clear(): void {
    this.ring = new Array(BUFFER_SIZE).fill(null);
    this.head = 0;
    this.count = 0;
    this.processed_ids.clear();
  }

  private store(entry: log_entry): void {
    if (this.processed_ids.size >= MAX_ID_CACHE) {
      const iter = this.processed_ids.values();
      for (let i = 0; i < MAX_ID_CACHE / 2; i++) {
        const { value, done } = iter.next();
        if (done) break;
        this.processed_ids.delete(value);
      }
    }
    this.processed_ids.add(entry.id);
    this.ring[this.head] = entry;
    this.head = (this.head + 1) % BUFFER_SIZE;
    if (this.count < BUFFER_SIZE) this.count++;
  }
}
