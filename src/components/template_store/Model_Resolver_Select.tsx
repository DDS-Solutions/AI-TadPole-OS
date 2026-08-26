/**
 * @docs ARCHITECTURE:UI-Components
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Template_Store / Model_Resolver_Select
 * - **Primary Entrypoints**: `Model_Resolver_Select`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: `Model_Resolver_Select.test.tsx`
 */

import { useState, useId } from 'react';
import { Sparkles, Cpu, HardDrive, Sliders, CheckCircle2 } from 'lucide-react';
import { use_settings_store } from '../../stores/settings_store';
import type { ModelMappingSelection, ModelMappingStrategy } from './types';

interface ModelResolverSelectProps {
    value: ModelMappingSelection;
    onChange: (selection: ModelMappingSelection) => void;
    templateOriginalModel?: string;
}

export function Model_Resolver_Select({
    value,
    onChange,
    templateOriginalModel = 'gemini-pro-latest, gpt-4o-mini'
}: ModelResolverSelectProps) {
    const { settings } = use_settings_store();
    const [customProvider, setCustomProvider] = useState(value.provider || 'openrouter');
    const [customModelId, setCustomModelId] = useState(value.modelId || 'stealth/ox-alpha');
    const providerSelectId = useId();
    const modelInputId = useId();

    const systemProvider = settings.default_provider || 'openrouter';
    const systemModel = settings.default_model || 'stealth/ox-alpha';

    const handleSelectStrategy = (strategy: ModelMappingStrategy) => {
        if (strategy === 'system') {
            onChange({
                strategy: 'system',
                provider: systemProvider,
                modelId: systemModel
            });
        } else if (strategy === 'template') {
            onChange({
                strategy: 'template'
            });
        } else if (strategy === 'ollama') {
            onChange({
                strategy: 'ollama',
                provider: 'ollama',
                modelId: 'gemma4:e4b',
                baseUrl: 'http://127.0.0.1:11434'
            });
        } else if (strategy === 'custom') {
            onChange({
                strategy: 'custom',
                provider: customProvider,
                modelId: customModelId
            });
        }
    };

    const handleCustomProviderChange = (newProvider: string) => {
        setCustomProvider(newProvider);
        if (value.strategy === 'custom') {
            onChange({
                strategy: 'custom',
                provider: newProvider,
                modelId: customModelId
            });
        }
    };

    const handleCustomModelChange = (newModelId: string) => {
        setCustomModelId(newModelId);
        if (value.strategy === 'custom') {
            onChange({
                strategy: 'custom',
                provider: customProvider,
                modelId: newModelId
            });
        }
    };

    return (
        <div className="p-5 bg-zinc-950/60 border border-[color:var(--color-border)] rounded-xl mt-6">
            <div className="flex items-center justify-between mb-4">
                <div className="flex items-center gap-2">
                    <Sliders size={16} className="text-emerald-400" />
                    <h3 className="text-xs font-bold uppercase tracking-wider text-zinc-300">
                        Model Provider Resolver & LLM Mapping
                    </h3>
                </div>
                <span className="text-[11px] font-mono text-zinc-500">
                    Target: <span className="text-emerald-400 font-semibold">
                        {value.strategy === 'system' && `${systemProvider} (${systemModel})`}
                        {value.strategy === 'template' && `Template Authored (${templateOriginalModel})`}
                        {value.strategy === 'ollama' && 'Ollama Local (gemma4:e4b)'}
                        {value.strategy === 'custom' && `${customProvider} (${customModelId})`}
                    </span>
                </span>
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
                {/* 1. System Default */}
                <button
                    type="button"
                    onClick={() => handleSelectStrategy('system')}
                    className={`p-3.5 rounded-lg border text-left transition-all relative flex flex-col justify-between ${
                        value.strategy === 'system'
                            ? 'bg-emerald-950/30 border-emerald-500/60 shadow-lg shadow-emerald-950/40 text-white'
                            : 'bg-[color:var(--color-surface)]/40 border-[color:var(--color-border)]/50 text-zinc-400 hover:text-zinc-200 hover:border-zinc-700'
                    }`}
                >
                    <div>
                        <div className="flex items-center justify-between mb-1.5">
                            <div className="flex items-center gap-2">
                                <Sparkles size={15} className={value.strategy === 'system' ? 'text-emerald-400' : 'text-zinc-400'} />
                                <span className="text-xs font-bold">Use System Default</span>
                            </div>
                            {value.strategy === 'system' && <CheckCircle2 size={14} className="text-emerald-400" />}
                        </div>
                        <p className="text-[11px] text-zinc-400 leading-snug">
                            {systemProvider} / <span className="font-mono text-zinc-300">{systemModel}</span>
                        </p>
                    </div>
                    <span className="mt-2 text-[9px] uppercase tracking-wider font-bold text-emerald-400/90 bg-emerald-500/10 px-1.5 py-0.5 rounded w-fit">
                        Recommended
                    </span>
                </button>

                {/* 2. Route to Local Ollama */}
                <button
                    type="button"
                    onClick={() => handleSelectStrategy('ollama')}
                    className={`p-3.5 rounded-lg border text-left transition-all relative flex flex-col justify-between ${
                        value.strategy === 'ollama'
                            ? 'bg-emerald-950/30 border-emerald-500/60 shadow-lg shadow-emerald-950/40 text-white'
                            : 'bg-[color:var(--color-surface)]/40 border-[color:var(--color-border)]/50 text-zinc-400 hover:text-zinc-200 hover:border-zinc-700'
                    }`}
                >
                    <div>
                        <div className="flex items-center justify-between mb-1.5">
                            <div className="flex items-center gap-2">
                                <HardDrive size={15} className={value.strategy === 'ollama' ? 'text-emerald-400' : 'text-zinc-400'} />
                                <span className="text-xs font-bold">Route to Local Ollama</span>
                            </div>
                            {value.strategy === 'ollama' && <CheckCircle2 size={14} className="text-emerald-400" />}
                        </div>
                        <p className="text-[11px] text-zinc-400 leading-snug">
                            ollama / <span className="font-mono text-zinc-300">gemma4:e4b</span>
                        </p>
                    </div>
                    <span className="mt-2 text-[9px] uppercase tracking-wider font-bold text-cyan-400/90 bg-cyan-500/10 px-1.5 py-0.5 rounded w-fit">
                        Zero-Cloud Privacy
                    </span>
                </button>

                {/* 3. Keep Template Originals */}
                <button
                    type="button"
                    onClick={() => handleSelectStrategy('template')}
                    className={`p-3.5 rounded-lg border text-left transition-all relative flex flex-col justify-between ${
                        value.strategy === 'template'
                            ? 'bg-emerald-950/30 border-emerald-500/60 shadow-lg shadow-emerald-950/40 text-white'
                            : 'bg-[color:var(--color-surface)]/40 border-[color:var(--color-border)]/50 text-zinc-400 hover:text-zinc-200 hover:border-zinc-700'
                    }`}
                >
                    <div>
                        <div className="flex items-center justify-between mb-1.5">
                            <div className="flex items-center gap-2">
                                <Cpu size={15} className={value.strategy === 'template' ? 'text-emerald-400' : 'text-zinc-400'} />
                                <span className="text-xs font-bold">Keep Template Models</span>
                            </div>
                            {value.strategy === 'template' && <CheckCircle2 size={14} className="text-emerald-400" />}
                        </div>
                        <p className="text-[11px] text-zinc-400 leading-snug line-clamp-1">
                            {templateOriginalModel}
                        </p>
                    </div>
                    <span className="mt-2 text-[9px] uppercase tracking-wider font-bold text-zinc-400 bg-zinc-800 px-1.5 py-0.5 rounded w-fit">
                        As Authored
                    </span>
                </button>
            </div>

            {/* Custom Provider Configuration Collapsible / Toggle */}
            <div className="mt-3 pt-3 border-t border-[color:var(--color-border)]/40 flex items-center justify-between">
                <button
                    type="button"
                    onClick={() => handleSelectStrategy('custom')}
                    className={`text-xs flex items-center gap-1.5 transition-colors ${
                        value.strategy === 'custom' ? 'text-emerald-400 font-semibold' : 'text-zinc-500 hover:text-zinc-300'
                    }`}
                >
                    <span>⚙️ Advanced: Specify custom model provider</span>
                </button>

                {value.strategy === 'custom' && (
                    <div className="flex items-center gap-2">
                        <label htmlFor={providerSelectId} className="sr-only">Custom Provider</label>
                        <select
                            id={providerSelectId}
                            value={customProvider}
                            onChange={(e) => handleCustomProviderChange(e.target.value)}
                            aria-label="Custom Provider"
                            className="text-xs bg-zinc-900 border border-zinc-700 rounded px-2 py-1 text-zinc-200 focus:outline-none focus:border-emerald-500"
                        >
                            <option value="openrouter">OpenRouter</option>
                            <option value="openai">OpenAI</option>
                            <option value="google">Google Gemini</option>
                            <option value="groq">Groq</option>
                            <option value="anthropic">Anthropic</option>
                            <option value="ollama">Ollama</option>
                            <option value="mistral">Mistral</option>
                            <option value="deepseek">DeepSeek</option>
                        </select>
                        <label htmlFor={modelInputId} className="sr-only">Model Identifier</label>
                        <input
                            id={modelInputId}
                            type="text"
                            value={customModelId}
                            onChange={(e) => handleCustomModelChange(e.target.value)}
                            placeholder="model-id"
                            aria-label="Model Identifier"
                            className="text-xs bg-zinc-900 border border-zinc-700 rounded px-2 py-1 text-zinc-200 w-36 font-mono focus:outline-none focus:border-emerald-500"
                        />
                    </div>
                )}
            </div>
        </div>
    );
}

// Metadata: [Template_Store]
