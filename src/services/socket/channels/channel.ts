/**
 * @docs ARCHITECTURE:UI-Services
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[channel]` in observability traces.
 */

import type { Incoming_Socket_Message } from '../types/events';

export interface Channel {
    readonly name: string;
    matches(message: Incoming_Socket_Message): boolean;
    handle(message: Incoming_Socket_Message): void;
    clear(): void;
}

export abstract class BaseChannel<T> implements Channel {
    abstract readonly name: string;
    protected listeners = new Set<(event: T) => void>();

    abstract matches(message: Incoming_Socket_Message): boolean;
    abstract handle(message: Incoming_Socket_Message): void;

    subscribe(listener: (event: T) => void): () => void {
        this.listeners.add(listener);
        return () => {
            this.listeners.delete(listener);
        };
    }

    emit(event: T): void {
        this.listeners.forEach(cb => {
            try {
                cb(event);
            } catch (err) {
                console.error(`[Channel: ${this.name}] Error in subscriber callback:`, err);
            }
        });
    }

    clear(): void {
        this.listeners.clear();
    }
}

// Metadata: [channel]
