/**
 * @docs ARCHITECTURE:Telemetry
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / telemetry_buffer
 * - **Primary Entrypoints**: `telemetry_buffer`, `telemetryBuffer`, `BufferedTelemetryEvent`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

export interface BufferedTelemetryEvent {
    id?: number;
    mission_id: string;
    event_type: 'handoff' | 'swarm_pulse' | 'log' | 'trace';
    timestamp: number;
    payload: Record<string, unknown>;
}

const DB_NAME = 'TadpoleTelemetryBuffer';
const DB_VERSION = 2; // Incremented for compound index support
const STORE_NAME = 'events';

class TelemetryBufferService {
    private db_promise: Promise<IDBDatabase> | null = null;

    private async get_db(): Promise<IDBDatabase> {
        if (this.db_promise) return this.db_promise;

        this.db_promise = new Promise((resolve, reject) => {
            const request = indexedDB.open(DB_NAME, DB_VERSION);

            request.onupgradeneeded = (event) => {
                const db = (event.target as IDBOpenDBRequest).result;
                if (!db.objectStoreNames.contains(STORE_NAME)) {
                    const store = db.createObjectStore(STORE_NAME, { keyPath: 'id', autoIncrement: true });
                    store.createIndex('mission_id', 'mission_id', { unique: false });
                    store.createIndex('timestamp', 'timestamp', { unique: false });
                    store.createIndex('mission_timestamp', ['mission_id', 'timestamp'], { unique: false });
                } else {
                    const store = (event.target as IDBOpenDBRequest).transaction?.objectStore(STORE_NAME);
                    if (store && !store.indexNames.contains('mission_timestamp')) {
                        store.createIndex('mission_timestamp', ['mission_id', 'timestamp'], { unique: false });
                    }
                }
            };

            request.onsuccess = () => resolve(request.result);
            request.onerror = () => reject(request.error);
        });

        return this.db_promise;
    }

    public async append_event(mission_id: string, event_type: BufferedTelemetryEvent['event_type'], payload: Record<string, unknown>): Promise<void> {
        const db = await this.get_db();
        return new Promise((resolve, reject) => {
            const tx = db.transaction(STORE_NAME, 'readwrite');
            const store = tx.objectStore(STORE_NAME);

            const event: BufferedTelemetryEvent = {
                mission_id: mission_id || 'global',
                event_type,
                timestamp: Date.now(),
                payload
            };

            store.add(event);

            tx.oncomplete = () => resolve();
            tx.onerror = () => reject(tx.error || new Error('[TelemetryBuffer] Transaction failed during append_event'));
            tx.onabort = () => reject(tx.error || new Error('[TelemetryBuffer] Transaction aborted during append_event'));
        });
    }

    public async appendEvent(missionId: string, eventType: BufferedTelemetryEvent['event_type'], payload: Record<string, unknown>): Promise<void> {
        return this.append_event(missionId, eventType, payload);
    }

    public async query_events(mission_id: string, start_time?: number, end_time?: number): Promise<BufferedTelemetryEvent[]> {
        const db = await this.get_db();
        return new Promise((resolve, reject) => {
            const tx = db.transaction(STORE_NAME, 'readonly');
            const store = tx.objectStore(STORE_NAME);

            // Use compound index ['mission_id', 'timestamp'] for zero-heap memory filtering
            if (store.indexNames.contains('mission_timestamp')) {
                const index = store.index('mission_timestamp');
                const range = IDBKeyRange.bound(
                    [mission_id, start_time || 0],
                    [mission_id, end_time || Number.MAX_SAFE_INTEGER]
                );

                const request = index.getAll(range);
                request.onsuccess = () => resolve(request.result as BufferedTelemetryEvent[]);
                request.onerror = () => reject(request.error);
            } else {
                const index = store.index('mission_id');
                const request = index.getAll(IDBKeyRange.only(mission_id));
                request.onsuccess = () => {
                    let results = request.result as BufferedTelemetryEvent[];
                    if (start_time) results = results.filter(e => e.timestamp >= start_time);
                    if (end_time) results = results.filter(e => e.timestamp <= end_time);
                    resolve(results);
                };
                request.onerror = () => reject(request.error);
            }
        });
    }

    public async queryEvents(missionId: string, startTime?: number, endTime?: number): Promise<BufferedTelemetryEvent[]> {
        return this.query_events(missionId, startTime, endTime);
    }

    public async clear_mission(mission_id: string): Promise<void> {
        const db = await this.get_db();
        return new Promise((resolve, reject) => {
            const tx = db.transaction(STORE_NAME, 'readwrite');
            const store = tx.objectStore(STORE_NAME);
            const index = store.index('mission_id');
            const request = index.openCursor(IDBKeyRange.only(mission_id));

            request.onsuccess = () => {
                const cursor = request.result;
                if (cursor) {
                    cursor.delete();
                    cursor.continue();
                }
            };

            request.onerror = () => reject(request.error);
            tx.oncomplete = () => resolve();
            tx.onerror = () => reject(tx.error || new Error('[TelemetryBuffer] Failed to clear mission events'));
        });
    }

    public async clearMission(missionId: string): Promise<void> {
        return this.clear_mission(missionId);
    }

    /**
     * Prunes events older than max_age_ms (default: 7 days) to bound IndexedDB storage.
     */
    public async prune_stale_events(max_age_ms = 7 * 24 * 60 * 60 * 1000): Promise<number> {
        const cutoff = Date.now() - max_age_ms;
        const db = await this.get_db();
        return new Promise((resolve, reject) => {
            const tx = db.transaction(STORE_NAME, 'readwrite');
            const store = tx.objectStore(STORE_NAME);
            const index = store.index('timestamp');
            const range = IDBKeyRange.upperBound(cutoff);
            const request = index.openCursor(range);
            let deleted_count = 0;

            request.onsuccess = () => {
                const cursor = request.result;
                if (cursor) {
                    cursor.delete();
                    deleted_count++;
                    cursor.continue();
                }
            };

            request.onerror = () => reject(request.error);
            tx.oncomplete = () => resolve(deleted_count);
            tx.onerror = () => reject(tx.error);
        });
    }
}

export const telemetry_buffer = new TelemetryBufferService();
export const telemetryBuffer = telemetry_buffer;
