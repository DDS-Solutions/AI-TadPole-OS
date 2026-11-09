/**
 * @docs ARCHITECTURE:Core
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Utils / mission_utils
 * - **Primary Entrypoints**: `clean_mission_title`, `is_mission_flight_event`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Strips injected Socratic envelope comments and raw markdown artifacts for clean UI presentation.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

/**
 * Sanitizes and cleans mission titles.
 * Injected Socratic context envelopes (`<!-- SOCRATIC_GATE_ENVELOPE: ... -->`) or markdown comments
 * are stripped so headers and catalog items display human-readable mission directives.
 */
export function clean_mission_title(title?: string | null, fallback = 'Untitled Sovereign Mission'): string {
    if (!title || typeof title !== 'string') return fallback;

    let cleaned = title;

    // 1. Remove closed envelope / HTML comments
    cleaned = cleaned.replace(/<!--[\s\S]*?-->/g, ' ');

    // 2. Remove unclosed / truncated HTML comments (e.g. `<!-- SOCRATIC_GATE_ENVELOPE: PRE-CLEARED ...`)
    cleaned = cleaned.replace(/<!--[\s\S]*$/g, ' ');

    // 3. Remove leading markdown heading symbols (e.g. `# `, `## `)
    cleaned = cleaned.replace(/^#+\s*/gm, '');

    // 4. Remove leading punctuation, colons, or ellipsis remnants
    cleaned = cleaned.replace(/^[\s.:,-]+/, '').trim();

    // 5. Collapse internal repeated whitespace and newlines to single space
    cleaned = cleaned.replace(/\s+/g, ' ').trim();

    // 6. Check if string is empty or just punctuation
    if (!cleaned || cleaned === '...' || cleaned === '---') {
        return fallback;
    }

    return cleaned;
}

/**
 * Determines whether an incoming WebSocket event pertains to active mission telemetry.
 * Captures mission_update broadcasts, discrete LogEntry frames with mission_id,
 * and legacy mission category/tags events (S-08).
 */
export function is_mission_flight_event(data: unknown): boolean {
    const evt = data as { type?: string; mission_id?: string; category?: string; tags?: string[] };
    return Boolean(
        evt?.type === 'mission_update' ||
        evt?.mission_id ||
        evt?.category === 'mission' ||
        (Array.isArray(evt?.tags) && evt.tags.includes('mission')) ||
        (evt?.type === 'log' && (Boolean(evt?.mission_id) || evt?.category === 'mission'))
    );
}

