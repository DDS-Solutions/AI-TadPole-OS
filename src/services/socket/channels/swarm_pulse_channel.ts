/**
 * @docs ARCHITECTURE:UI-Services
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / swarm_pulse_channel
 * - **Primary Entrypoints**: `SwarmPulseChannel`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { BaseChannel } from './channel';
import type { Swarm_Pulse } from '../../../types';

export class SwarmPulseChannel extends BaseChannel<Swarm_Pulse> {
    readonly name = 'swarm_pulse';

    matches(): boolean {
        return false; // Binary packets are routed explicitly
    }

    handle(): void {
        // No-op
    }

    handle_binary(pulse: Swarm_Pulse): void {
        this.emit(pulse);
    }
}
