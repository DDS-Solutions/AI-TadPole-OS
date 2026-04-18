/**
 * @docs ARCHITECTURE:UI-Hooks
 * 
 * ### AI Assist Note
 * **Sovereign Interface component for the Tadpole OS dashboard.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[useErrorRemediation.ts]` in observability traces.
 */

/**
 * @docs ARCHITECTURE:UI-Hooks
 * 
 * ### AI Assist Note
 * **Autonomous Remediation Hook**: Interprets high-fidelity error codes (RFC 9457)
 * from the Tadpole engine and suggests actionable "Sovereign Repairs."
 */

import { useCallback } from 'react';

export interface RemediationAction {
    label: string;
    action: () => void;
    severity: 'info' | 'warning' | 'error';
    help_link?: string;
}

interface EngineError {
    error_code?: string;
    help_link?: string;
}

export function useErrorRemediation() {
    const get_remediation = useCallback((error: EngineError): RemediationAction | null => {
        const code = error?.error_code || '';
        const help_link = error?.help_link || undefined;

        if (code.startsWith('OOM_QUANTIZATION_FALLBACK')) {
            const model_id = code.split(':')[1] || 'current model';
            return {
                label: `Switch ${model_id} to efficient quantization (q4_K_M)`,
                action: () => {
                    // Logic to navigate to settings or trigger model update
                    console.debug("Triggering OOM Remediation for:", model_id);
                },
                severity: 'warning',
                help_link
            };
        }

        if (code.startsWith('RECRUITMENT_FAILED')) {
            const role = code.split(':')[1] || 'specialist';
            return {
                label: `Recruit ${role} from alternate Bunker node`,
                action: () => {
                    console.debug("Triggering Recruitment Remediation for:", role);
                },
                severity: 'error',
                help_link
            };
        }

        if (code.startsWith('INFRA_ERROR')) {
            const provider_id = code.split(':')[1] || 'provider';
            return {
                label: `Verify API Token for ${provider_id}`,
                action: () => {
                    console.debug("Triggering Connectivity Remediation for:", provider_id);
                },
                severity: 'error',
                help_link
            };
        }

        // Default to generic help if only help_link exists
        if (help_link) {
            return {
                label: 'View Resolution Guide',
                action: () => window.open(help_link, '_blank'),
                severity: 'info',
                help_link
            };
        }

        return null;
    }, []);

    return { get_remediation };
}
