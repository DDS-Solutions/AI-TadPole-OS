/**
 * @docs ARCHITECTURE:Infrastructure
 * 
 * ### AI Assist Note
 * **Validation Engine**: Sanitization and constraint enforcement for system identifiers and API payloads. 
 * Orchestrates regex-based ID normalization and numeric safety checks (temperature, limits) for the Tadpole OS registry.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Sanitization collision (two different names resulting in the same `sanitize_id`), or URL protocol rejection.
 * - **Telemetry Link**: Look for `is_valid_name` or `sanitize_id` in call stacks during "Invalid Configuration" errors.
 */

/**
 * High-fidelity validation utilities for the Tadpole OS registry.
 * Prevents malformed inputs from reaching the Rust backend.
 * Refactored for strict snake_case compliance.
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


// Metadata: [validation_utils]

// Metadata: [validation_utils]
