/**
 * @docs ARCHITECTURE:Infrastructure
 * 
 * ### AI Assist Note
 * **UI Utility**: Swarm Dialect Translator (AAAK Decoder). 
 * Decodes the high-efficiency AAAK sequence dialect used for swarm findings into human-friendly natural language for the UI.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Regex collision with user-generated text (false positive detection) or unmapped AAAK codes (remains raw).
 * - **Telemetry Link**: Search for `decodeAAAK` in UI logs or dashboard tracing.
 */

export const AAAK_MAP: Record<string, string> = {
  "\\*ok\\*": "✅ Status: Success",
  "\\*err\\*": "❌ Status: Failed",
  "RES:": "🔍 Result:",
  "FND:": "💡 Finding:",
  "SRC:": "🌐 Source:",
  "LOC:": "📍 Location:",
  "GOAL:": "🎯 Primary Goal:",
  "WTR\\|": "🌤️ Weather Data: ",
  "deg": "degrees",
  "temp": "temperature",
  "\\*done\\*": "🏁 Mission Complete",
  "\\*busy\\*": "🐝 Task in progress"
};

/**
 * Transforms a compressed AAAK string into professional human-readable text.
 * @param text The raw AAAK string from the backend.
 * @returns Expanded, human-friendly text.
 */
export function decodeAAAK(text: string): string {
  if (!text) return "";
  let decoded = text;

  // Apply each mapping expansion
  Object.entries(AAAK_MAP).forEach(([pattern, replacement]) => {
    const regex = new RegExp(pattern, "g");
    decoded = decoded.replace(regex, replacement);
  });

  return decoded;
}

/**
 * Checks if a string contains any AAAK dialect markers.
 */
export function isAAAK(text: string): boolean {
  return /[*|]|RES:|FND:|SRC:|GOAL:/.test(text);
}
