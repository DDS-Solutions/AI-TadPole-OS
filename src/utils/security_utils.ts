/**
 * @docs ARCHITECTURE:Security
 * 
 * ### AI Context Alignment
 * - **Subsystem**: Sovereign Frontend / Utilities / security_utils
 * - **Primary Entrypoints**: `sanitize_telemetry`, `sanitize_payload`, `scan_and_redact_secrets`
 * 
 * ### AI Assist Note
 * **Security Utility**: Provides robust sanitization for real-time telemetry and agent-generated content.
 * Prevents XSS by stripping dangerous HTML tags and event handlers.
 * Deterministic Secret & Credential Pre-Flight Scanner (DLP) redacts sensitive keys before transmission.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

/**
 * Sanitizes a string by stripping HTML tags and potentially dangerous attributes.
 * Focused on preventing XSS in real-time logs and thought streams.
 */
export const sanitize_telemetry = (text: string): string => {
    if (!text) return '';

    // 1. Strip <script> tags and their contents
    let sanitized = text.replace(/<script\b[^>]*>([\s\S]*?)<\/script>/gim, '');

    // 2. Strip common dangerous event handlers (on* attributes)
    sanitized = sanitized.replace(/on\w+="[^"]*"/gim, '');
    sanitized = sanitized.replace(/on\w+='[^']*'/gim, '');
    sanitized = sanitized.replace(/on\w+=\S+/gim, '');

    // 3. Strip potentially dangerous tags but leave formatting if needed
    const dangerous_tags = /<(iframe|object|embed|form|frameset|frame|applet|meta|link|style|base)\b[^>]*>([\s\S]*?)<\/\1>|<(iframe|object|embed|form|frameset|frame|applet|meta|link|style|base)\b[^>]*>/gim;
    sanitized = sanitized.replace(dangerous_tags, '');

    return sanitized;
};

/**
 * Normalizes and sanitizes an entire telemetry payload recursively.
 * Ensures that all string fields are stripped of dangerous HTML.
 */
export const sanitize_payload = <T>(payload: T): T => {
    if (payload === null || payload === undefined) {
        return payload;
    }

    if (typeof payload === 'string') {
        return sanitize_telemetry(payload) as unknown as T;
    }

    if (Array.isArray(payload)) {
        return payload.map(item => sanitize_payload(item)) as unknown as T;
    }

    if (typeof payload === 'object') {
        const new_obj = { ...payload } as Record<string, unknown>;
        for (const key in new_obj) {
            if (Object.prototype.hasOwnProperty.call(new_obj, key)) {
                new_obj[key] = sanitize_payload(new_obj[key]);
            }
        }
        return new_obj as unknown as T;
    }

    return payload;
};

export interface SecretScanResult {
    sanitized: string;
    has_secrets: boolean;
    redacted_count: number;
    detected_types: string[];
}

/**
 * Deterministic Secret & Credential Pre-Flight Scanner (DLP).
 * Scans outgoing text for credentials (OpenAI/Anthropic/Google/GitHub/Bearer)
 * and redacts them in-place with zero latency and 100% deterministic precision.
 */
export const scan_and_redact_secrets = (text: string): SecretScanResult => {
    if (!text) {
        return { sanitized: '', has_secrets: false, redacted_count: 0, detected_types: [] };
    }

    let sanitized = text;
    let redacted_count = 0;
    const detected_types: string[] = [];

    // 1. Private Keys
    const private_key_regex = /-----BEGIN[ A-Z0-9_-]+PRIVATE KEY-----[\s\S]*?-----END[ A-Z0-9_-]+PRIVATE KEY-----/gi;
    if (private_key_regex.test(sanitized)) {
        sanitized = sanitized.replace(private_key_regex, '[REDACTED_PRIVATE_KEY]');
        detected_types.push('Private Key');
        redacted_count++;
    }

    // 2. OpenAI / Anthropic / Groq / XAI Keys (including sk-ant- and sk-proj-)
    const ai_key_regex = /\b(?:sk-ant-[a-zA-Z0-9_-]{20,}|sk-proj-[a-zA-Z0-9_-]{20,}|sk-[a-zA-Z0-9_-]{20,}|gsk_[a-zA-Z0-9_-]{20,}|xai-[a-zA-Z0-9_-]{20,})\b/g;
    const ai_matches = sanitized.match(ai_key_regex);
    if (ai_matches) {
        sanitized = sanitized.replace(ai_key_regex, '[REDACTED_AI_KEY]');
        detected_types.push('AI Provider Key');
        redacted_count += ai_matches.length;
    }

    // 3. Google API Keys
    const google_key_regex = /\bAIza[0-9A-Za-z-_]{35}\b/g;
    const google_matches = sanitized.match(google_key_regex);
    if (google_matches) {
        sanitized = sanitized.replace(google_key_regex, '[REDACTED_GOOGLE_KEY]');
        detected_types.push('Google API Key');
        redacted_count += google_matches.length;
    }

    // 4. GitHub Tokens (classic ghp_ and modern github_pat_)
    const github_token_regex = /\b(?:gh[pousr]_[0-9a-zA-Z]{30,}|github_pat_[0-9a-zA-Z_]{30,})\b/g;
    const github_matches = sanitized.match(github_token_regex);
    if (github_matches) {
        sanitized = sanitized.replace(github_token_regex, '[REDACTED_GITHUB_TOKEN]');
        detected_types.push('GitHub Token');
        redacted_count += github_matches.length;
    }


    // 5. Explicit Bearer Tokens
    const bearer_regex = /Bearer\s+([a-zA-Z0-9_\-.]{25,})/gi;
    if (bearer_regex.test(sanitized)) {
        sanitized = sanitized.replace(bearer_regex, 'Bearer [REDACTED_BEARER_TOKEN]');
        detected_types.push('Bearer Token');
        redacted_count++;
    }

    return {
        sanitized,
        has_secrets: redacted_count > 0,
        redacted_count,
        detected_types
    };
};
