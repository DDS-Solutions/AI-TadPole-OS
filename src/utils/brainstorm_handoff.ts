/**
 * @docs ARCHITECTURE:Logic
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Utilities / brainstorm_handoff
 * - **Primary Entrypoints**: `build_brainstorm_context`, `Brainstorm_Entry`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Pure deterministic transcript formatting bounded to prevent prompt blowup.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

export interface Brainstorm_Entry {
    speaker: string;
    text: string;
    is_system?: boolean;
}

const DEFAULT_MAX_TURNS = 12;
const DEFAULT_MAX_CHARS = 2400;

/**
 * Formats recent conversational turns from Chat/Plan mode into a compact
 * bounded context block passed via `recent_findings` across turns and on
 * Brainstorm -> Swarm Mission handoff.
 */
export function build_brainstorm_context(
    entries: Brainstorm_Entry[],
    max_turns: number = DEFAULT_MAX_TURNS,
    max_chars: number = DEFAULT_MAX_CHARS
): string | undefined {
    const conversational = entries
        .filter((e) => !e.is_system && Boolean(e.text?.trim()))
        .slice(-max_turns);

    if (conversational.length === 0) return undefined;

    const lines = conversational.map((e) => {
        const cleaned = e.text.replace(/\s+/g, ' ').trim();
        return `${e.speaker}: ${cleaned}`;
    });

    const header = '### Prior Brainstorm Context (Chat/Plan Session)\n';
    let body = lines.join('\n');
    const available = Math.max(120, max_chars - header.length);
    if (body.length > available) {
        body = `...${body.slice(body.length - (available - 3))}`;
    }

    return `${header}${body}`;
}
