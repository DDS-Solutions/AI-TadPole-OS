/**
 * @docs ARCHITECTURE:UI-Pages
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Pages / Standups
 * - **Primary Entrypoints**: `Standups`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { useStandups } from '../hooks/useStandups';
import { Standup_Meeting_Area } from '../components/voice/Standup_Meeting_Area';
import { Transcript_Viewer } from '../components/voice/Transcript_Viewer';
import { Live_Voice_Hub } from '../components/voice/Live_Voice_Hub';
import { i18n } from '../i18n';

/**
 * Standups
 * Voice communication and activity hub for real-time mission sequences.
 * Converted to a container-presentational pattern.
 */
export default function Standups() {
    const {
        is_live,
        is_muted,
        sync_error,
        can_start_sync,
        transcript_history,
        active_speaker,
        agents,
        target_type,
        selected_target_id,
        live_seconds,
        active_agent,
        clusters,
        input_devices,
        selected_device_id,
        execution_mode,
        can_launch_swarm,
        set_is_live,
        set_target_type,
        set_selected_target_id,
        set_selected_device_id,
        set_execution_mode,
        launch_swarm_handoff,
        toggle_live,
        toggle_mute
    } = useStandups();

    return (
        <div className="h-full grid grid-cols-1 lg:grid-cols-3 gap-6 p-6">
            {/* SEO Optimization: Structured Data & Semantic Header */}
            <h1 className="sr-only">{i18n.t('common.standups_title')}</h1>
            <script type="application/ld+json">
                {JSON.stringify({
                    "@context": "https://schema.org",
                    "@type": "Report",
                    "name": "Tadpole OS Mission Standup",
                    "description": "Real-time synchronization and status reporting hub for sovereign agent clusters.",
                    "author": { "@type": "Person", "name": "Agent of Nine" },
                    "datePublished": new Date().toISOString()
                })}
            </script>

            <Standup_Meeting_Area
                is_live={is_live}
                is_muted={is_muted}
                sync_error={sync_error}
                can_start_sync={can_start_sync}
                live_seconds={live_seconds}
                target_type={target_type}
                selected_target_id={selected_target_id}
                agents={agents}
                clusters={clusters}
                input_devices={input_devices}
                selected_device_id={selected_device_id}
                execution_mode={execution_mode}
                can_launch_swarm={can_launch_swarm}
                set_target_type={set_target_type}
                set_selected_target_id={set_selected_target_id}
                set_selected_device_id={set_selected_device_id}
                set_execution_mode={set_execution_mode}
                launch_swarm_handoff={launch_swarm_handoff}
                toggle_live={toggle_live}
                toggle_mute={toggle_mute}
            />

            <Transcript_Viewer
                is_live={is_live}
                transcript_history={transcript_history}
                active_speaker={active_speaker}
            />

            {/* Gemini Live HUD Overlay */}
            {is_live && target_type === 'agent' && active_agent?.voice_engine === 'gemini-live' && (
                <Live_Voice_Hub
                    agent_id={selected_target_id}
                    theme_color={active_agent?.theme_color || '#10b981'}
                    on_close={() => set_is_live(false)}
                />
            )}
        </div>
    );
}
