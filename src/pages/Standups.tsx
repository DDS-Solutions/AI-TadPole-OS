import { useState, useEffect, useRef } from 'react';
import { Mic, Users, Play, Pause, BarChart3, Target, ChevronDown } from 'lucide-react';
import { Tooltip } from '../components/ui';
import { EventBus } from '../services/eventBus';
import { voiceClient } from '../services/voiceClient';
import { TadpoleOSService } from '../services/tadpoleosService';
import { useWorkspaceStore } from '../stores/workspaceStore';
import { loadAgents } from '../services/agentService';
import { i18n } from '../i18n';
import { type Agent } from '../types';
import { resolveAgentModelConfig } from '../utils/modelUtils';

const AudioVisualizer = ({ isActive }: { isActive: boolean }) => {
    return (
        <div className="flex items-end justify-center gap-1 h-12 w-32">
            {[...Array(8)].map((_, i) => (
                <div
                    key={i}
                    className={`w-2 bg-emerald-500 rounded-t transition-all duration-150 ${isActive ? 'animate-pulse' : 'h-1 bg-zinc-800'}`}
                    style={{ height: isActive ? `${20 + (Math.sin(i * 1.5) * 30 + 30)}%` : '4px' }}
                ></div>
            ))}
        </div>
    );
};

/**
 * Voice Communication / Standup page.
 * Subscribes to EventBus for live transcript updates.
 * Integrates with VoiceClient for speech-to-text and text-to-speech.
 */

export default function Standups() {
    const [isLive, setIsLive] = useState(false);
    const [transcript, setTranscript] = useState<string[]>([i18n.t('standups.msg_online')]);
    const [activeSpeaker, setActiveSpeaker] = useState<string | null>(null);
    const [agents, setAgents] = useState<Agent[]>([]);
    const clusters = useWorkspaceStore(state => state.clusters);
    const [targetType, setTargetType] = useState<'agent' | 'cluster'>('agent');
    const [selectedTargetId, setSelectedTargetId] = useState<string>('');
    const lastSpokenRef = useRef<string | null>(null);

    // Initial setup
    useEffect(() => {
        loadAgents().then(data => {
            setAgents(data);
            if (data.length > 0) setSelectedTargetId(data[0].id);
        });
    }, []);

    // ── Voice & Transcript Subscriptions ──────────────────────────────────
    useEffect(() => {
        // 1. Subscribe to EventBus for visual transcript
        const unsubscribeBus = EventBus.subscribe((entry) => {
            if (entry.source === 'User' || entry.source === 'Agent') {
                const speakerName = entry.source === 'User' ? 'User' : (entry.agentId || 'Agent');
                const line = `${speakerName}: ${entry.text}`;

                setTranscript(prev => [...prev, line]);
                setActiveSpeaker(speakerName);
                setTimeout(() => setActiveSpeaker(null), 3000);

                // 2. Speak Agent responses (if they change and it's not the same as last spoken)
                if (entry.source === 'Agent' && entry.text !== lastSpokenRef.current) {
                    voiceClient.speak(entry.text);
                    lastSpokenRef.current = entry.text;
                }
            }
        });

        // 3. Neural Handoff (Option B): High-Fidelity Backend Transcription
        const handleNeuralSync = async () => {
            if (isLive) {
                console.debug(`🎙️ [Sovereignty] ${i18n.t('standups.debug_start_recording')}`);
                await voiceClient.startRecording();
            } else {
                console.debug(`🎙️ [Sovereignty] ${i18n.t('standups.debug_ending_sync')}`);
                const audioBlob = await voiceClient.stopRecording();
                if (audioBlob && selectedTargetId) {
                    // Show a temporary "processing" message
                    setTranscript(prev => [...prev, i18n.t('standups.msg_transcribing')]);

                    const text = await TadpoleOSService.transcribe(audioBlob);
                    if (text) {
                        const targetName = targetType === 'agent'
                            ? (agents.find(a => a.id === selectedTargetId)?.name || i18n.t('standups.label_target_agent'))
                            : (clusters.find(c => c.id === selectedTargetId)?.name || i18n.t('standups.label_target_cluster'));

                        EventBus.emit({
                            source: 'User',
                            text: `${text} (To: ${targetName})`,
                            severity: 'info'
                        });

                        // Dispatch to backend
                        if (targetType === 'agent') {
                            const agent = agents.find(a => a.id === selectedTargetId);
                            if (agent) {
                                const { modelId, provider } = resolveAgentModelConfig(agent);
                                TadpoleOSService.sendCommand(selectedTargetId, text, modelId, provider);
                            }
                        } else {
                            const cluster = clusters.find(c => c.id === selectedTargetId);
                            if (cluster?.alphaId) {
                                const alphaAgent = agents.find(a => a.id === cluster.alphaId);
                                if (alphaAgent) {
                                    const { modelId, provider } = resolveAgentModelConfig(alphaAgent);
                                    TadpoleOSService.sendCommand(
                                        cluster.alphaId,
                                        `[CLUSTER COMMAND: ${cluster.name}] ${text}`,
                                        modelId,
                                        provider,
                                        cluster.id,
                                        cluster.department, // Pass department as a string
                                        cluster.budgetUsd
                                    );
                                }
                            }
                        }
                    } else {
                        setTranscript(prev => [...prev, i18n.t('standups.msg_failed')]);
                    }
                }
            }
        };

        handleNeuralSync();

        return () => {
            unsubscribeBus();
            voiceClient.stopRecording();
        };
    }, [isLive, selectedTargetId, agents, clusters, targetType]);

    return (
        <div className="h-full grid grid-cols-1 lg:grid-cols-3 gap-6">
            {/* Active Meeting Area */}
            <div className="lg:col-span-2 bg-zinc-950 border border-zinc-800 rounded-xl p-8 flex flex-col items-center justify-center relative overflow-hidden shadow-sm">
                <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_center,_var(--tw-gradient-stops))] from-blue-900/10 via-transparent to-transparent"></div>

                <div className="z-10 text-center space-y-6">
                    <div className="w-32 h-32 rounded-full bg-zinc-900 border-4 border-zinc-800 flex items-center justify-center shadow-2xl relative">
                        {isLive ? <div className="absolute inset-0 rounded-full animate-ping bg-blue-500/10"></div> : null}
                        <Users size={48} className="text-zinc-600" />
                    </div>

                    <Tooltip content={i18n.t('standups.tooltip_interface')} position="bottom">
                        <div>
                            <h2 className="text-2xl font-bold text-zinc-100 cursor-help">{i18n.t('standups.title')}</h2>
                            <p className="text-zinc-500 mt-2 font-mono text-[10px] uppercase tracking-widest">{isLive ? i18n.t('standups.status_live', { time: '00:14:23' }) : i18n.t('standups.status_ready')}</p>
                        </div>
                    </Tooltip>

                    {/* Target Selector */}
                    <div className="w-full max-w-sm flex flex-col gap-3 p-4 bg-zinc-900/50 border border-zinc-800 rounded-2xl backdrop-blur-sm">
                        <div className="flex bg-zinc-950 p-1 rounded-lg border border-zinc-800">
                            <Tooltip content={i18n.t('standups.tooltip_agent')} position="top">
                                <button
                                    onClick={() => { setTargetType('agent'); setSelectedTargetId(agents[0]?.id || ''); }}
                                    className={`flex-1 flex items-center justify-center gap-2 py-1.5 text-[10px] font-bold uppercase tracking-widest transition-all rounded-md ${targetType === 'agent' ? 'bg-zinc-800 text-zinc-100' : 'text-zinc-500 hover:text-zinc-300'}`}
                                >
                                    <Users size={12} /> {i18n.t('standups.label_agent_node')}
                                </button>
                            </Tooltip>
                            <Tooltip content={i18n.t('standups.tooltip_cluster')} position="top">
                                <button
                                    onClick={() => { setTargetType('cluster'); setSelectedTargetId(clusters[0]?.id || ''); }}
                                    className={`flex-1 flex items-center justify-center gap-2 py-1.5 text-[10px] font-bold uppercase tracking-widest transition-all rounded-md ${targetType === 'cluster' ? 'bg-zinc-800 text-zinc-100' : 'text-zinc-500 hover:text-zinc-300'}`}
                                >
                                    <Target size={12} /> {i18n.t('standups.label_mission_cluster')}
                                </button>
                            </Tooltip>
                        </div>

                        <div className="relative">
                            <label htmlFor="target-select" className="sr-only">{i18n.t('standups.placeholder_select_target')}</label>
                            <select
                                id="target-select"
                                value={selectedTargetId}
                                onChange={(e) => setSelectedTargetId(e.target.value)}
                                className="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-4 py-2.5 text-xs text-zinc-200 focus:outline-none focus:border-zinc-700 transition-all font-mono appearance-none cursor-pointer"
                            >
                                {targetType === 'agent' ? (
                                    agents.map(a => <option key={a.id} value={a.id}>{a.name.toUpperCase()}</option>)
                                ) : (
                                    clusters.map(c => <option key={c.id} value={c.id}>{c.name.toUpperCase()}</option>)
                                )}
                            </select>
                            <ChevronDown size={14} className="absolute right-3 top-1/2 -translate-y-1/2 text-zinc-600 pointer-events-none" />
                        </div>
                    </div>

                    <AudioVisualizer isActive={isLive} />

                    <div className="flex gap-4">
                        <Tooltip content={isLive ? i18n.t('standups.tooltip_end') : i18n.t('standups.tooltip_start')} position="top">
                            <button
                                onClick={() => setIsLive(!isLive)}
                                className={`px-6 py-2 rounded-full font-bold flex items-center gap-2 transition-all ${isLive ? 'bg-red-500/10 text-red-500 hover:bg-red-500/20' : 'bg-emerald-500 text-black hover:bg-emerald-400'}`}>
                                {isLive ? <><Pause size={18} /> {i18n.t('standups.btn_end')}</> : <><Play size={18} /> {i18n.t('standups.btn_start')}</>}
                            </button>
                        </Tooltip>
                        <Tooltip content={i18n.t('standups.tooltip_mic')} position="top">
                            <button className={`p-2 rounded-full text-zinc-400 hover:bg-zinc-700 transition-colors ${isLive ? 'bg-red-900/40 text-red-400' : 'bg-zinc-800'}`}>
                                <Mic size={20} />
                            </button>
                        </Tooltip>
                    </div>
                </div>
            </div>

            {/* Live Transcript / Activity */}
            <div className="bg-zinc-950 border border-zinc-800 rounded-xl flex flex-col overflow-hidden">
                <Tooltip content={i18n.t('standups.tooltip_transcript')} position="left">
                    <div className="p-4 border-b border-zinc-800 bg-zinc-900 flex items-center justify-between cursor-help">
                        <h3 className="font-bold text-zinc-400 text-sm flex items-center gap-2">
                            <BarChart3 size={16} /> {i18n.t('standups.header_transcript')}
                        </h3>
                        <div className="flex items-center gap-2">
                            {isLive && <span className="w-2 h-2 bg-red-500 rounded-full animate-pulse"></span>}
                            <span className="text-xs text-zinc-500 font-mono">{isLive ? i18n.t('standups.status_rec') : i18n.t('standups.status_idle')}</span>
                        </div>
                    </div>
                </Tooltip>

                <div className="flex-1 overflow-y-auto p-4 space-y-4">
                    {transcript.length === 0 ? (
                        <div className="h-full flex flex-col items-center justify-center text-zinc-600 italic text-sm">
                            {i18n.t('standups.empty_transcript')}
                        </div>
                    ) : (
                        transcript.map((line, i) => {
                            const [speaker, text] = line.split(': ');
                            return (
                                <div key={i} className="flex gap-3">
                                    <div className={`w-8 h-8 rounded shrink-0 flex items-center justify-center text-xs font-bold ${speaker === 'User' ? 'bg-emerald-900/50 text-emerald-400' :
                                        speaker === 'Agent' ? 'bg-blue-900/50 text-blue-400' :
                                            speaker === 'Dev-1' ? 'bg-zinc-900/50 text-zinc-400' :
                                                speaker === 'User' ? 'bg-emerald-900/50 text-emerald-400' : 'bg-zinc-800 text-zinc-400'
                                        }`}>
                                        {speaker.substring(0, 1)}
                                    </div>
                                    <div>
                                        <div className="text-xs font-bold text-zinc-400 mb-0.5">{speaker}</div>
                                        <p className="text-sm text-zinc-200">{text}</p>
                                    </div>
                                </div>
                            )
                        })
                    )}
                    {activeSpeaker && (
                        <div className="flex gap-2 items-center text-zinc-500 text-xs pl-11 animate-pulse">
                            {i18n.t('standups.label_speaking', { name: activeSpeaker })}
                        </div>
                    )}
                </div>
            </div>
        </div>
    )
}


