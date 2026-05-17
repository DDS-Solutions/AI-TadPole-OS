/**
 * @docs ARCHITECTURE:UI-Pages
 * 
 * ### AI Assist Note
 * **Settings Page**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Settings]` in observability traces.
 */

import React from 'react';
import { Save } from 'lucide-react';
import { useNavigate } from 'react-router-dom';
import { use_model_store } from '../stores/model_store';
import { useSettingsManager } from '../hooks/useSettingsManager';
import { Tooltip } from '../components/ui';
import { i18n } from '../i18n';

// Modular Components
import { Connection_Settings } from '../components/settings/Connection_Settings';
import { Appearance_Settings } from '../components/settings/Appearance_Settings';
import { Agent_Defaults_Settings } from '../components/settings/Agent_Defaults_Settings';
import { Governance_Settings } from '../components/settings/Governance_Settings';
import { Architecture_Settings } from '../components/settings/Architecture_Settings';
import { Intelligence_Store_Banner } from '../components/settings/Intelligence_Store_Banner';
import { Template_Store_Banner } from '../components/settings/Template_Store_Banner';

/**
 * Settings Page
 * 
 * ### ⚙️ System Configuration
 * The central orchestration hub for environmental variables and engine tuning. 
 * Refactored to Container-Presentational pattern.
 */
export default function Settings(): React.ReactElement {
    const navigate = useNavigate();
    const models = use_model_store(state => state.models);
    const {
        settings,
        is_saved,
        validation_error,
        handle_change,
        handle_numeric_change,
        handle_save
    } = useSettingsManager();

    return (
        <div className="max-w-4xl mx-auto p-6 space-y-8 h-full overflow-y-auto custom-scrollbar relative">
            {/* GEO Optimization: Structured Data & Semantic Header */}
            <script type="application/ld+json">
                {JSON.stringify({
                    "@context": "https://schema.org",
                    "@type": "SoftwareApplication",
                    "name": "Tadpole OS System Settings",
                    "description": "Global configuration and environmental tuning hub for the Tadpole OS. Manage neural preferences, API vault synchronization, and governance caps.",
                    "author": { "@type": "Organization", "name": "Sovereign Engineering" },
                    "applicationCategory": "Configuration Tool",
                    "operatingSystem": "Tadpole OS"
                })}
            </script>
            <h1 className="sr-only">Tadpole OS System Configuration & Environmental Governance</h1>

            {/* Sticky Header Actions */}
            <div className="flex justify-end pr-2">
                <Tooltip content={i18n.t('settings.tooltip_save')} position="left">
                    <button
                        onClick={handle_save}
                        className={`flex items-center gap-2 px-4 py-2 rounded-lg font-bold transition-all shadow-sm ${is_saved ? 'bg-emerald-500 text-white' : validation_error ? 'bg-red-600 text-white' : 'bg-zinc-100 text-zinc-900 hover:bg-white'}`}
                    >
                        <Save size={16} />
                        {is_saved ? i18n.t('settings.saved') : validation_error ? i18n.t('settings.fix_errors') : i18n.t('settings.save_changes')}
                    </button>
                </Tooltip>
            </div>

            <Connection_Settings 
                settings={settings}
                handle_change={handle_change}
                validation_error={validation_error}
            />

            <Appearance_Settings 
                settings={settings}
                handle_change={handle_change}
            />

            <Agent_Defaults_Settings 
                settings={settings}
                models={models}
                handle_change={handle_change}
            />

            <Governance_Settings 
                settings={settings}
                handle_change={handle_change}
            />

            <Architecture_Settings 
                settings={settings}
                handle_numeric_change={handle_numeric_change}
            />

            <Intelligence_Store_Banner 
                on_browse={() => navigate('/infra/model-store')}
                is_scanning={false}
            />

            <Template_Store_Banner 
                on_open_store={() => navigate('/store')}
            />
        </div>
    );
}


// Metadata: [Settings]
