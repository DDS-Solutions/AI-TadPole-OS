/**
 * @docs ARCHITECTURE:UI-Services
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Service Layer / origin_guard
 * - **Primary Entrypoints**: `is_allowed_origin`, `is_private_network_origin`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

const cache = new Map<string, boolean>();

/**
 * is_private_network_origin
 * Identifies if a hostname belongs to local loopback, RFC 1918 private IPv4 subnets, or mDNS/local domains.
 */
export function is_private_network_origin(hostname: string): boolean {
    const clean = hostname.toLowerCase().trim();
    if (clean === 'localhost' || clean === '127.0.0.1' || clean === '[::1]' || clean === '::1') {
        return true;
    }
    if (clean.endsWith('.local') || clean.endsWith('.lan') || clean.endsWith('.internal')) {
        return true;
    }

    // Check RFC 1918 IPv4 ranges (10.0.0.1/8, 10.0.0.1/12, 10.0.0.1/16) and 10.0.0.1/8
    const ipv4_match = clean.match(/^(\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})$/);
    if (ipv4_match) {
        const octets = ipv4_match.slice(1, 5).map(Number);
        if (octets.some(o => o < 0 || o > 255)) return false;
        const [o1, o2] = octets;
        if (o1 === 10) return true;
        if (o1 === 172 && o2 >= 16 && o2 <= 31) return true;
        if (o1 === 192 && o2 === 168) return true;
        if (o1 === 127) return true;
    }
    return false;
}

function run_origin_check(url_string: string, allowed_origins?: string[], allow_private_network = true): boolean {
    try {
        const url = new URL(url_string);
        const hostname = url.hostname.toLowerCase();
        const port = url.port || (url.protocol === 'https:' || url.protocol === 'wss:' ? '443' : '80');
        
        // 1. Always allow local loopback
        if (hostname === 'localhost' || hostname === '127.0.0.1' || hostname === '[::1]' || hostname === '::1') {
            return true;
        }

        // 2. Allow same-origin if running in browser
        if (typeof window !== 'undefined' && window.location && window.location.hostname.toLowerCase() === hostname) {
            return true;
        }

        // 3. Allow private network origins when enabled (e.g. LAN self-hosting mode)
        if (allow_private_network && is_private_network_origin(hostname)) {
            return true;
        }
        
        // 4. Build-time allowed origins
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
            
            // If the pattern includes protocol or slashes, parse as URL
            if (allowed.includes('://')) {
                try {
                    const allowed_url = new URL(allowed);
                    const allowed_port = allowed_url.port || (allowed_url.protocol === 'https:' || allowed_url.protocol === 'wss:' ? '443' : '80');
                    if (allowed_url.port) {
                        return allowed_url.hostname.toLowerCase() === hostname && allowed_port === port;
                    }
                    return allowed_url.hostname.toLowerCase() === hostname;
                } catch {
                    return false;
                }
            }
            
            // Check host:port format
            if (allowed.includes(':')) {
                const [allowed_host, allowed_p] = allowed.split(':');
                return allowed_host === hostname && allowed_p === port;
            }
            
            // Check wildcard domains (*.example.com)
            if (allowed.startsWith('*.')) {
                const root = allowed.slice(2);
                return hostname === root || hostname.endsWith(`.${root}`);
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
export function is_allowed_origin(url_string: string, allowed_origins?: string[], allow_private_network = true): boolean {
    const env_allowed = (typeof import.meta !== 'undefined' && import.meta.env?.VITE_ALLOWED_ORIGINS) || '';
    const cache_key = `${url_string}::${(allowed_origins || []).join(',')}::${env_allowed}::${allow_private_network}`;
    
    if (cache.has(cache_key)) {
        return cache.get(cache_key)!;
    }
    
    const result = run_origin_check(url_string, allowed_origins, allow_private_network);
    cache.set(cache_key, result);
    return result;
}
