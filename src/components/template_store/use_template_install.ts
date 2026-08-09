/**
 * @docs ARCHITECTURE:Hooks
 * 
 * ### AI Assist Note
 * **Custom Hook**: Encapsulates template installation orchestration and event dispatching.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: System installation API errors.
 * - **Telemetry Link**: Search `[Template_Store]` in service logs.
 */

import { useState, useCallback } from 'react';
import { system_api_service } from '../../services/system_api_service';
import { REPO_URL, APP_REFRESH_AGENTS_EVENT } from './constants';
import type { Template } from './types';

export function useTemplateInstall(onSuccess?: (templateId: string) => void) {
    const [isInstalling, setIsInstalling] = useState<string | null>(null);

    const install = useCallback(async (template: Template, customConfig?: Record<string, unknown>) => {
        try {
            setIsInstalling(template.id);
            // Route all installations through the proven engine.install_template endpoint.
            // For local/downloaded swarms, use the template ID as the path identifier.
            await system_api_service.engine.install_template(
                REPO_URL,
                template.path === 'local' ? template.id : template.path
            );

            // Dispatch global refresh event
            window.dispatchEvent(new Event(APP_REFRESH_AGENTS_EVENT));

            if (onSuccess) {
                onSuccess(template.id);
            }
            return true;
        } catch (err) {
            const msg = err instanceof Error ? err.message : String(err);
            window.alert(`Error installing template: ${msg}`);
            return false;
        } finally {
            setIsInstalling(null);
        }
    }, [onSuccess]);

    return {
        isInstalling,
        install
    };
}

// Metadata: [Template_Store]


// [Template_Store]
