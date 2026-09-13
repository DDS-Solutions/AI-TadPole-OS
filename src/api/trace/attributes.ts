/**
 * @docs ARCHITECTURE:Core
 *
 * ### AI Context Alignment
 * - **Subsystem**: System Core / attributes
 * - **Primary Entrypoints**: `build_trace_attributes`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { get_response_header } from './response-headers';

export const build_trace_attributes = (
    response: Response,
    extra: Record<string, string | number | boolean> = {},
): Record<string, string | number | boolean> => {
    const attributes: Record<string, string | number | boolean> = {
        'http.status_code': response.status,
        ...extra,
    };
    const request_id = get_response_header(response, 'x-request-id');
    const traceparent = get_response_header(response, 'traceparent');
    if (request_id) attributes['resp.x_request_id'] = request_id;
    if (traceparent) attributes['resp.traceparent'] = traceparent;
    return attributes;
};
