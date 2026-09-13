/**
 * @docs ARCHITECTURE:UI-Hooks
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend React Hooks / useSettingsManager
 * - **Primary Entrypoints**: `useSettingsManager`, `UseSettingsManagerHook`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` React hook lifecycle adheres to Rules of Hooks without conditional execution branches.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { useState, useEffect } from 'react';
import { get_settings, save_settings, type Tadpole_Settings } from '../stores/settings_store';
import { system_api_service } from '../services/system_api_service';
import { i18n } from '../i18n';

export interface UseSettingsManagerHook {
    settings: Tadpole_Settings;
    is_saved: boolean;
    validation_error: string | null;
    
    // Handlers
    handle_change: (e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>) => void;
    handle_numeric_change: (name: string, value: number) => void;
    handle_save: () => Promise<void>;
}

export function useSettingsManager(): UseSettingsManagerHook {
    const [settings, set_settings] = useState(() => get_settings());
    const [is_saved, set_is_saved] = useState(false);
    const [validation_error, set_validation_error] = useState<string | null>(null);

    useEffect(() => {
        let isMounted = true;
        void system_api_service.oversight.get_governance_settings().then((gov) => {
            if (!isMounted || !gov) return;
            set_settings(prev => ({
                ...prev,
                ...(gov.privacy_mode !== undefined ? { privacy_mode: gov.privacy_mode } : {}),
                ...(gov.auto_approve_safe_skills !== undefined ? { auto_approve_safe_skills: gov.auto_approve_safe_skills } : {}),
                ...(gov.max_agents !== undefined ? { max_agents: gov.max_agents } : {}),
                ...(gov.max_clusters !== undefined ? { max_clusters: gov.max_clusters } : {}),
                ...(gov.max_swarm_depth !== undefined ? { max_swarm_depth: gov.max_swarm_depth } : {}),
                ...(gov.max_task_length !== undefined ? { max_task_length: gov.max_task_length } : {}),
                ...(gov.default_budget_usd !== undefined ? { default_budget_usd: gov.default_budget_usd } : {}),
                ...(gov.default_model ? { default_model: gov.default_model } : {}),
                ...(gov.default_provider ? { default_provider: gov.default_provider } : {})
            }));
        }).catch(() => {
            // Non-blocking fallback to local state
        });
        return () => { isMounted = false; };
    }, []);

    const handle_change = (e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>): void => {
        const { name, value, type } = e.target;
        const val = type === 'checkbox' ? (e.target as HTMLInputElement).checked : value;

        set_settings(prev => ({
            ...prev,
            [name]: val
        }));
        set_is_saved(false);
        set_validation_error(null);
    };

    const handle_numeric_change = (name: string, value: number): void => {
        set_settings(prev => ({ ...prev, [name]: value }));
        set_is_saved(false);
        set_validation_error(null);
    };

    const handle_save = async (): Promise<void> => {
        const error = save_settings(settings);
        if (error) {
            set_validation_error(error);
            return;
        }

        try {
            await system_api_service.oversight.update_governance_settings({
                auto_approve_safe_skills: settings.auto_approve_safe_skills,
                privacy_mode: settings.privacy_mode,
                max_agents: settings.max_agents,
                max_clusters: settings.max_clusters,
                max_swarm_depth: settings.max_swarm_depth,
                max_task_length: settings.max_task_length,
                default_budget_usd: settings.default_budget_usd,
                default_model: settings.default_model,
                default_provider: settings.default_provider
            });
        } catch (e) {
            console.error("Failed to sync governance settings with engine", e);
            set_validation_error(i18n.t('settings.error_sync_failed', { defaultValue: 'System synchronization failed. Local changes saved.' }));
            return;
        }

        // Apply appearance preferences
        document.documentElement.setAttribute('data-theme', settings.theme);
        document.documentElement.setAttribute('data-density', settings.density);

        set_is_saved(true);
        set_validation_error(null);
        setTimeout(() => set_is_saved(false), 2000);
    };

    return {
        settings,
        is_saved,
        validation_error,
        handle_change,
        handle_numeric_change,
        handle_save
    };
}
