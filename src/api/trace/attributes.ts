/**
 * @docs ARCHITECTURE:Core
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[attributes]` in observability traces.
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

// Metadata: [attributes]
