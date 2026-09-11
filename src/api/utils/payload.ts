/**
 * @docs ARCHITECTURE:Core
 *
 * ### AI Context Alignment
 * - **Subsystem**: System Core / payload
 * - **Primary Entrypoints**: `truncate_payload`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

export function truncate_payload(data: string, max_length = 1024): string {
    if (data.length <= max_length) {
        return data;
    }
    return `${data.substring(0, max_length)}... [TRUNCATED ${data.length} bytes]`;
}
