/**
 * High-fidelity validation utilities for the Tadpole OS registry.
 * Prevents malformed inputs from reaching the Rust backend.
 */

export const ValidationUtils = {
    /**
     * Sanitizes a string for use as a system identifier or label.
     */
    sanitizeId: (id: string): string => {
        return id.toLowerCase().replace(/[^a-z0-9-]/g, '-').replace(/-+/g, '-').replace(/^-|-$/g, '');
    },

    /**
     * Validates a display name (required, non-empty, trimmed).
     */
    isValidName: (name: string): boolean => {
        return !!name && name.trim().length >= 2 && name.trim().length <= 64;
    },

    /**
     * Validates numeric limits for API governance.
     */
    isValidLimit: (val: number | undefined, min = 0, max = 1000000000): boolean => {
        if (val === undefined) return true;
        return val >= min && val <= max;
    },

    /**
     * Validates temperature (0.0 to 2.0).
     */
    isValidTemperature: (val: number | undefined): boolean => {
        if (val === undefined) return true;
        return val >= 0 && val <= 2.0;
    },

    /**
     * Validates a backend URL / Endpoint.
     */
    isValidUrl: (url: string | undefined): boolean => {
        if (!url) return true;
        try {
            const parsed = new URL(url);
            return parsed.protocol === 'http:' || parsed.protocol === 'https:';
        } catch {
            return false;
        }
    }
};
