/**
 * @docs ARCHITECTURE:UI-Services
 * 
 * ### AI Assist Note
 * **is_allowed_origin**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[origin_guard]` in observability traces.
 */

const cache = new Map<string, boolean>();

function run_origin_check(url_string: string, allowed_origins?: string[]): boolean {
    try {
        const url = new URL(url_string);
        const hostname = url.hostname.toLowerCase();
        
        // Always allow local loopback
        if (hostname === 'localhost' || hostname === '127.0.0.1' || hostname === '[::1]') {
            return true;
        }

        // Also allow same-origin if running in browser
        if (typeof window !== 'undefined' && window.location && window.location.hostname.toLowerCase() === hostname) {
            return true;
        }
        
        // Build-time allowed origins
        const env_allowed = (typeof import.meta !== 'undefined' && import.meta.env?.VITE_ALLOWED_ORIGINS) || '';
        const env_origins = env_allowed
            ? env_allowed.split(',').map((x: string) => x.trim().toLowerCase())
            : [];
        
        const runtime_allowed = allowed_origins 
            ? allowed_origins.map(x => x.trim().toLowerCase()) 
            : [];
            
        const all_allowed = [...env_origins, ...runtime_allowed];
        
        return all_allowed.some(allowed => {
            if (!allowed) return false;
            if (allowed.includes('://')) {
                try {
                    const allowed_url = new URL(allowed);
                    return allowed_url.hostname === hostname;
                } catch {
                    return false;
                }
            }
            return allowed === hostname;
        });
    } catch {
        return false;
    }
}

/**
 * is_allowed_origin
 * Validates the socket target URL to prevent token exfiltration.
 */
export function is_allowed_origin(url_string: string, allowed_origins?: string[]): boolean {
    const env_allowed = (typeof import.meta !== 'undefined' && import.meta.env?.VITE_ALLOWED_ORIGINS) || '';
    const cache_key = `${url_string}::${(allowed_origins || []).join(',')}::${env_allowed}`;
    
    if (cache.has(cache_key)) {
        return cache.get(cache_key)!;
    }
    
    const result = run_origin_check(url_string, allowed_origins);
    cache.set(cache_key, result);
    return result;
}

/** Resets the internal origin validation cache (mainly for test cleanup) */
export function reset_origin_cache(): void {
    cache.clear();
}

// Metadata: [origin_guard]
