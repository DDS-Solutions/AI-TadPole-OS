/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Sub-module**: Provider-linked audio and transcription model inventory. 
 * Manages the available transcription engines for the specific provider.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Audio model ID mismatch (400), or missing model availability for specific region.
 */

import React from 'react';
import { Zap, Info } from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';
import { type Model_Entry } from '../../stores/model_store';

/**
 * Audio_Settings_Props
 * Defines the props for the Audio_Settings component.
 */
interface Audio_Settings_Props {
    /** Currently selected audio/transcription model */
    audio_model: string;
    /** Current provider ID */
    provider_id: string;
    /** Full model catalog from model_store */
    models: Model_Entry[];
    /** Update handler for form fields */
    on_change: (field: string, value: string) => void;
}

/**
 * Audio_Settings
 * Manages the selection and suggestions for transcription-specific model infrastructure.
 */
export function Audio_Settings({
    audio_model,
    provider_id,
    models,
    on_change
}: Audio_Settings_Props): React.ReactElement {
    return (
        <section className="space-y-4">
            <div className="flex items-center gap-2">
                <h3 className="text-[11px] font-bold text-zinc-500 uppercase tracking-[0.2em] flex items-center gap-2">
                    <Zap size={12} className="text-amber-400/50" />
                    {i18n.t('provider.audio_service')}
                </h3>
            </div>
            <div className="bg-zinc-950/50 border border-zinc-800 rounded-2xl p-4 space-y-4">
                <div className="space-y-1.5">
                    <label htmlFor="provider-audio-model" className="text-[10px] font-bold text-zinc-600 uppercase tracking-widest px-1 flex items-center gap-2">
                        {i18n.t('provider.field_audio_model')}
                        <Tooltip content={i18n.t('provider.audio_model_tooltip')} position="top">
                            <Info size={10} className="text-zinc-700 hover:text-blue-400 cursor-help transition-colors" />
                        </Tooltip>
                    </label>
                    <div className="space-y-2">
                        <input
                            id="provider-audio-model"
                            className="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-1.5 text-xs text-blue-400 focus:outline-none focus:border-blue-500 font-mono"
                            placeholder={i18n.t('provider.placeholder_audio_model')}
                            list="voice-inventory-suggestions"
                            value={audio_model}
                            onChange={e => on_change('audio_model', e.target.value)}
                            aria-label={i18n.t('provider.field_audio_model_label')}
                        />
                        <datalist id="voice-inventory-suggestions">
                            <option value="whisper-large-v3" />
                            {models
                                .filter(m => m.provider === provider_id && m.modality?.toLowerCase() === 'voice')
                                .map(m => (
                                    <option key={m.id} value={m.name}>
                                        {m.name.toUpperCase()} ({i18n.t('provider.label_inventory')})
                                    </option>
                                ))}
                        </datalist>
                    </div>
                    <p className="text-[9px] text-zinc-600 font-medium px-1 italic">
                        {i18n.t('provider.audio_model_hint')}
                    </p>
                </div>
            </div>
        </section>
    );
}
