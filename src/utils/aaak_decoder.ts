/**
 * @docs ARCHITECTURE:Infrastructure
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Utilities / aaak_decoder
 * - **Primary Entrypoints**: `decodeAAAK`, `isAAAK`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

const AAAK_MAP: Record<string, string> = {
  "\\*ok\\*": "✅ Status: Success",
  "\\*err\\*": "❌ Status: Failed",
  "RES:": "🔍 Result:",
  "FND:": "💡 Finding:",
  "SRC:": "🌐 Source:",
  "LOC:": "📍 Location:",
  "GOAL:": "🎯 Primary Goal:",
  "WTR\\|": "🌤️ Weather Data: ",
  "\\bdeg\\b": "degrees",
  "\\btemp\\b": "temperature",
  "\\*done\\*": "🏁 Mission Complete",
  "\\*busy\\*": "🐝 Task in progress"
};

const PRECOMPILED_AAAK_RULES: Array<[RegExp, string]> = Object.entries(AAAK_MAP).map(
  ([pattern, replacement]) => [new RegExp(pattern, "g"), replacement]
);

/**
 * Transforms a compressed AAAK string into professional human-readable text.
 * @param text The raw AAAK string from the backend.
 * @returns Expanded, human-friendly text.
 */
export function decodeAAAK(text: string): string {
  if (!text) return "";
  let decoded = text;

  // Apply each pre-compiled mapping expansion
  for (const [regex, replacement] of PRECOMPILED_AAAK_RULES) {
    decoded = decoded.replace(regex, replacement);
  }

  return decoded;
}

/**
 * Checks if a string contains any AAAK dialect markers.
 */
export function isAAAK(text: string): boolean {
  return /[*|]|RES:|FND:|SRC:|LOC:|GOAL:|WTR\|/.test(text);
}
