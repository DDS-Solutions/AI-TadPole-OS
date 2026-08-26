/**
 * @docs ARCHITECTURE:Infrastructure
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend Utilities / validation_utils
 * - **Primary Entrypoints**: `ValidationUtils`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

export const ValidationUtils = {
    /**
     * Sanitizes a string for use as a system identifier or label.
     */
    sanitize_id: (id: string): string => {
        return id.toLowerCase().replace(/[^a-z0-9-]/g, '-').replace(/-+/g, '-').replace(/^-|-$/g, '');
    },

    /**
     * Validates a display name (required, non-empty, trimmed).
     */
    is_valid_name: (name: string): boolean => {
        return !!name && name.trim().length >= 2 && name.trim().length <= 64;
    },

    /**
     * Validates numeric limits for API governance.
     */
    is_valid_limit: (val: number | undefined, min = 0, max = 1000000000): boolean => {
        if (val === undefined) return true;
        return val >= min && val <= max;
    },

    /**
     * Validates temperature (0.0 to 2.0).
     */
    is_valid_temperature: (val: number | undefined): boolean => {
        if (val === undefined) return true;
        return val >= 0 && val <= 2.0;
    },

    /**
     * Validates a backend URL / Endpoint.
     */
    is_valid_url: (url: string | undefined): boolean => {
        if (!url) return true;
        try {
            const parsed = new URL(url);
            return parsed.protocol === 'http:' || parsed.protocol === 'https:';
        } catch {
            return false;
        }
    }
};
