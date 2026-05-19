/**
 * @docs ARCHITECTURE:UI-Components
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Simulation_Banner]` in observability traces.
 */

import React from 'react';
import { WifiOff } from 'lucide-react';
import { i18n } from '../../i18n';

interface SimulationBannerProps {
    is_simulated: boolean;
    set_is_simulated: (simulated: boolean) => void;
}

export const Simulation_Banner: React.FC<SimulationBannerProps> = ({ is_simulated, set_is_simulated }) => {
    if (!is_simulated) return null;

    return (
        <div className="bg-amber-500/10 border border-amber-500/30 p-3 rounded-xl flex items-center justify-between animate-in fade-in slide-in-from-top-4">
            <div className="flex items-center gap-3">
                <div className="p-1.5 bg-amber-500/20 rounded-lg">
                    <WifiOff size={16} className="text-amber-500" />
                </div>
                <div>
                    <p className="text-xs font-bold text-amber-200 uppercase tracking-widest">{i18n.t('oversight.disconnected_title')}</p>
                    <p className="text-[10px] text-amber-500/70 font-mono">{i18n.t('oversight.disconnected_subtitle')}</p>
                </div>
            </div>
            <button
                onClick={() => set_is_simulated(false)}
                className="text-[10px] px-3 py-1 bg-amber-500/20 hover:bg-amber-500/30 text-amber-400 rounded-full border border-amber-500/30 transition-colors uppercase font-bold tracking-tighter"
            >
                {i18n.t('oversight.retry_connection')}
            </button>
        </div>
    );
};

// Metadata: [Simulation_Banner]
