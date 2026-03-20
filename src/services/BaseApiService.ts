import { getSettings } from '../stores/settingsStore';

import { useTraceStore } from '../stores/traceStore';

export const DEFAULT_TIMEOUT = 30000; // 30 seconds default
export const DEPLOY_TIMEOUT = 7200000; // 2 hours for deployment

export function withTimeout(timeoutMs: number = DEFAULT_TIMEOUT): { signal: AbortSignal; clear: () => void } {
    const controller = new AbortController();
    const id = setTimeout(() => controller.abort('TIMEOUT'), timeoutMs);
    return { signal: controller.signal, clear: () => clearTimeout(id) };
}

export function getHeaders(customRequestId?: string): { headers: HeadersInit, context: { spanId: string, traceId: string, traceparent: string, requestId: string } } {
    const { TadpoleOSApiKey } = getSettings();
    const token = TadpoleOSApiKey || 'tadpole-dev-token-2026';

    const requestId = customRequestId || ((typeof crypto !== 'undefined' && crypto.randomUUID) ? crypto.randomUUID() : `tr-${Date.now()}`);
    const traceId = requestId.replace(/-/g, '').padEnd(32, '0').slice(0, 32);

    // Generate a random span ID for the frontend request (16 hex chars)
    let spanId = '';
    if (typeof crypto !== 'undefined' && crypto.getRandomValues) {
        spanId = Array.from(crypto.getRandomValues(new Uint8Array(8)))
            .map(b => b.toString(16).padStart(2, '0'))
            .join('');
    } else {
        spanId = Math.random().toString(16).slice(2, 10) + Math.random().toString(16).slice(2, 10);
    }

    const traceparent = `00-${traceId}-${spanId}-01`;

    return {
        headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${token}`,
            'X-Request-Id': requestId,
            'traceparent': traceparent
        },
        context: { spanId, traceId, traceparent, requestId }
    };
}

export async function apiRequest<T = unknown>(
    path: string,
    options: RequestInit & { responseType?: 'json' | 'blob' | 'text'; timeout?: number } = {}
): Promise<T> {
    const { TadpoleOSUrl } = getSettings();

    if (!TadpoleOSUrl) {
        throw new Error('Neural Link Configuration Missing: TadpoleOSUrl is undefined.');
    }

    const baseUrl = TadpoleOSUrl.replace(/\/$/, '');
    const cleanPath = path.startsWith('/') ? path : `/${path}`;
    const url = `${baseUrl}${cleanPath}`;
    const { signal, clear } = withTimeout(options.timeout);

    const { headers: baseHeaders, context } = getHeaders((options.headers as Record<string, string>)?.['X-Request-Id']);

    // Emit start span
    useTraceStore.getState().addSpan({
        id: context.spanId,
        traceId: context.traceId,
        name: `ui_request: ${path.split('?')[0]}`,
        agentId: 'frontend',
        missionId: 'system',
        startTime: Date.now(),
        status: 'running',
        attributes: {}
    });

    try {
        const response = await fetch(url, {
            ...options,
            headers: {
                ...baseHeaders,
                ...options.headers
            },
            signal: options.signal || signal
        });

        if (!response.ok) {
            const errorText = await response.text();
            let errorJson: Record<string, unknown> | null = null;
            try { errorJson = JSON.parse(errorText); } catch { /* ignore */ }

            const type = (errorJson?.type as string) || 'about:blank';
            const title = (errorJson?.title as string) || response.statusText;
            const detail = (errorJson?.detail as string) || (errorJson?.message as string) || 'Unknown Infrastructure Error';
            const message = `${title}: ${detail}`;

            useTraceStore.getState().updateSpan(context.spanId, {
                endTime: Date.now(),
                status: 'error',
                attributes: { 'http.status_code': response.status }
            });

            const error = new Error(message) as Error & { type?: string; status?: number };
            error.type = type;
            error.status = response.status;
            throw error;
        }

        let result: unknown;
        if (response.status === 204) {
            result = null;
        } else if (options.responseType === 'blob') {
            result = await response.blob();
        } else if (options.responseType === 'text') {
            result = await response.text();
        } else {
            const text = await response.text();
            result = text ? JSON.parse(text) : null;
        }

        useTraceStore.getState().updateSpan(context.spanId, {
            endTime: Date.now(),
            status: 'success',
            attributes: { 'http.status_code': response.status }
        });

        clear();
        return result as T;
    } catch (err) {
        clear();
        throw err;
    }
}
