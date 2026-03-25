import { Info } from 'lucide-react';
import { Tooltip } from '../ui';
import { i18n } from '../../i18n';

interface VoiceSectionProps {
    voice: {
        voiceId: string;
        voiceEngine: string;
    };
    sttEngine: string;
    themeColor: string;
    onUpdateVoice: (field: string, value: string) => void;
}

export function VoiceSection({ voice, sttEngine, themeColor, onUpdateVoice }: VoiceSectionProps) {
    return (
        <div className="space-y-6">
            <div className="pt-4 border-t border-zinc-800/50">
                <label className="block text-[10px] font-bold text-zinc-500 uppercase tracking-widest mb-2 flex items-center gap-2">
                    {i18n.t('agent_config.voice_integration')}
                    <Tooltip content={i18n.t('agent_config.tooltip_voice_engine')} position="top">
                        <Info size={10} className="text-zinc-700 hover:text-zinc-300 cursor-help transition-colors" />
                    </Tooltip>
                </label>
                <div 
                    className="grid grid-cols-2 gap-3 bg-zinc-900/50 border border-zinc-800 rounded-lg p-3 transition-all"
                    style={{ borderLeft: `2px solid ${themeColor}40` }}
                >
                    <div className="space-y-1.5">
                        <label className="block text-[9px] font-bold text-zinc-600 uppercase opacity-60">{i18n.t('agent_config.label_engine')}</label>
                        <select
                            value={voice.voiceEngine}
                            onChange={(e) => onUpdateVoice('voiceEngine', e.target.value)}
                            className="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-2 text-xs text-zinc-200 focus:outline-none focus:border-zinc-700 appearance-none font-bold cursor-pointer"
                        >
                            <option value="browser">{i18n.t('agent_config.label_browser_local')}</option>
                            <option value="piper">{i18n.t('agent_config.voice_piper')}</option>
                            <option value="openai">{i18n.t('agent_config.voice_openai_hifi')}</option>
                            <option value="groq">{i18n.t('agent_config.label_groq_fast')}</option>
                        </select>
                    </div>
                    <div className="space-y-1.5">
                        <label className="block text-[9px] font-bold text-zinc-600 uppercase opacity-60 flex items-center gap-1.5">
                            {i18n.t('agent_config.label_vocal_identity')}
                            <Tooltip content={i18n.t('agent_config.tooltip_vocal_identity')} position="top">
                                <Info size={9} className="text-zinc-700 hover:text-zinc-300 cursor-help transition-colors" />
                            </Tooltip>
                        </label>
                        {voice.voiceEngine === 'openai' ? (
                            <select
                                value={voice.voiceId}
                                onChange={(e) => onUpdateVoice('voiceId', e.target.value)}
                                className="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-2 text-xs text-zinc-200 focus:outline-none focus:border-zinc-700 appearance-none font-bold cursor-pointer"
                            >
                                <option value="alloy">{i18n.t('agent_config.voice_alloy')}</option>
                                <option value="echo">{i18n.t('agent_config.label_echo_deep')}</option>
                                <option value="fable">{i18n.t('agent_config.voice_fable')}</option>
                                <option value="onyx">{i18n.t('agent_config.voice_onyx')}</option>
                                <option value="nova">{i18n.t('agent_config.voice_nova')}</option>
                                <option value="shimmer">{i18n.t('agent_config.voice_shimmer')}</option>
                            </select>
                        ) : (
                            <input
                                type="text"
                                placeholder={i18n.t('agent_config.placeholder_voice_id')}
                                value={voice.voiceId}
                                onChange={(e) => onUpdateVoice('voiceId', e.target.value)}
                                className="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-2 text-xs text-zinc-200 focus:outline-none focus:border-zinc-700 font-mono"
                            />
                        )}
                    </div>
                </div>
            </div>

            <div className="pt-4 border-t border-zinc-800/50">
                <label className="block text-[10px] font-bold text-zinc-500 uppercase tracking-widest mb-2 flex items-center gap-2">
                    {i18n.t('agent_config.stt_integration')}
                    <Tooltip content={i18n.t('agent_config.tooltip_stt')} position="top">
                        <Info size={10} className="text-zinc-700 hover:text-zinc-300 cursor-help transition-colors" />
                    </Tooltip>
                </label>
                <div 
                    className="bg-zinc-900/50 border border-zinc-800 rounded-lg p-3 transition-all"
                    style={{ borderLeft: `2px solid ${themeColor}40` }}
                >
                    <div className="space-y-1.5">
                        <label className="block text-[9px] font-bold text-zinc-600 uppercase opacity-60">{i18n.t('agent_config.label_engine')}</label>
                        <select
                            value={sttEngine || 'groq'}
                            onChange={(e) => onUpdateVoice('sttEngine', e.target.value)}
                            className="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-2 text-xs text-zinc-200 focus:outline-none focus:border-zinc-700 appearance-none font-bold cursor-pointer"
                        >
                            <option value="groq">{i18n.t('agent_config.label_groq_ultra')}</option>
                            <option value="whisper">{i18n.t('agent_config.voice_whisper_private')}</option>
                        </select>
                    </div>
                </div>
            </div>
        </div>
    );
}
