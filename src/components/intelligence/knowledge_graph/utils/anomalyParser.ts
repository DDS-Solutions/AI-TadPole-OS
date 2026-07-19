/**
 * @docs ARCHITECTURE:Intelligence
 *
 * ### AI Assist Note
 * Extracts structured anomaly data from raw anomaly strings returned by the
 * backend. Centralizes the regex parsing so it can be unit-tested independently
 * and replaced with structured backend data in the future.
 *
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Backend changes anomaly string format → regex misses → returns raw string as name.
 * - **Telemetry Link**: Search `[anomaly_parser]` in component traces.
 */

export interface ParsedAnomaly {
    /** The detected anomaly type. */
    type: 'UNUSED_SYMBOL' | 'UNKNOWN';
    /** Symbol name extracted from the anomaly string, or the full string if unparseable. */
    name: string;
    /** Raw (unredacted) file path. `null` if not parseable. */
    rawPath: string | null;
    /** Original anomaly string for display/copy fallback. */
    original: string;
    /** Stable key for React list rendering (deterministic, unique per anomaly). */
    stableKey: string;
}

// Regex for the known anomaly format from the code-graph analysis engine.
// Captures: (1) symbol name, (2) file path.
const UNUSED_SYMBOL_RE = /^Unused symbol \(0 incoming references\): (\w+) in (.+)$/;

/**
 * Parses a raw anomaly string into structured data.
 * Returns `ParsedAnomaly` with extracted fields, or a safe fallback if the
 * format is not recognized.
 */
export function parseAnomaly(anomaly: string, index: number): ParsedAnomaly {
    const match = anomaly.match(UNUSED_SYMBOL_RE);
    if (match) {
        return {
            type: 'UNUSED_SYMBOL',
            name: match[1],
            rawPath: match[2],
            original: anomaly,
            stableKey: `unused-${match[1]}-${match[2]}`,
        };
    }
    return {
        type: 'UNKNOWN',
        name: anomaly,
        rawPath: null,
        original: anomaly,
        stableKey: `unknown-${index}-${anomaly.slice(0, 64)}`,
    };
}

// Metadata: [anomaly_parser]
