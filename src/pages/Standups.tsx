/**
 * @docs ARCHITECTURE:UI-Pages
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[Standups]` in observability traces.
 */

import { useStandups } from '../hooks/useStandups';
import { Standup_Meeting_Area } from '../components/voice/Standup_Meeting_Area';
import { Transcript_Viewer } from '../components/voice/Transcript_Viewer';
import { Live_Voice_Hub } from '../components/voice/Live_Voice_Hub';

/**
 * Standups
 * Voice communication and activity hub for real-time mission sequences.
 * Converted to a container-presentational pattern.
 */
export default function Standups() {
    const {
        is_live,
        transcript_history,
        active_speaker,
        agents,
        target_type,
        selected_target_id,
        live_seconds,
        active_agent,
        clusters,
        set_is_live,
        set_target_type,
        set_selected_target_id,
        toggle_live
    } = useStandups();

    return (
        <div className="h-full grid grid-cols-1 lg:grid-cols-3 gap-6">
            {/* SEO Optimization: Structured Data & Semantic Header */}
            <h1 className="sr-only">Tadpole OS Swarm Standups: Real-time Coordination Hub</h1>
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
                live_seconds={live_seconds}
                target_type={target_type}
                selected_target_id={selected_target_id}
                agents={agents}
                clusters={clusters}
                set_target_type={set_target_type}
                set_selected_target_id={set_selected_target_id}
                toggle_live={toggle_live}
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

// Metadata: [Standups]

// Metadata: [Standups]

// Metadata: [Standups]
