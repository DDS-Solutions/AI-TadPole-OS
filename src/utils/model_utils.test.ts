/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Verification of the Model Resolver and Provider Registry.** 
 * Validates the normalization of friendly model names (e.g., 'Gemini 3.1 Pro') into backend technical IDs and the heuristic resolution of AI providers based on naming patterns. 
 * Ensures that agent model configuration correctly respects multi-slot overrides and global intelligence synchronization.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Incorrect provider mapping leading to invalid API key usage or failure to apply global default model to legacy agents.
 * - **Telemetry Link**: Run `npm run test` or search `[model_utils.test]` in Vitest logs.
 */

import { describe, it, expect } from 'vitest';
import { resolve_technical_model_id, resolve_provider, resolve_agent_model_config, get_model_color } from './model_utils';
import type { Agent } from '../types';

describe('model_utils', () => {
    describe('resolve_technical_model_id', () => {
        it('resolves mapped names correctly', () => {
            expect(resolve_technical_model_id('Gemini 1.5 Pro')).toBe('gemini-1.5-pro');
            expect(resolve_technical_model_id('GPT-5.2')).toBe('gpt-5.2-preview');
        });

        it('returns original name if no mapping found', () => {
            expect(resolve_technical_model_id('Custom Model')).toBe('Custom Model');
        });

        it('returns unknown for empty/null input', () => {
            expect(resolve_technical_model_id(undefined)).toBe('unknown');
            expect(resolve_technical_model_id('')).toBe('unknown');
        });
    });

    describe('resolve_provider', () => {
        it('identifies core providers', () => {
            expect(resolve_provider('gpt-4')).toBe('openai');
            expect(resolve_provider('claude-3')).toBe('anthropic');
            expect(resolve_provider('gemini-pro')).toBe('google');
            expect(resolve_provider('ollama:llama3')).toBe('ollama');
        });

        it('identifies secondary vendors', () => {
            expect(resolve_provider('mistral-large')).toBe('mistral');
            expect(resolve_provider('deepseek-v3')).toBe('deepseek');
            expect(resolve_provider('grok-2')).toBe('xai');
        });

        it('handles groq/llama ambiguity', () => {
            expect(resolve_provider('llama-3-groq')).toBe('groq');
            expect(resolve_provider('llama-3-vanilla')).toBe('meta');
        });

        it('fallbacks to google by default', () => {
            expect(resolve_provider('mystery-ai')).toBe('google');
        });
    });

    describe('resolve_agent_model_config', () => {
        const base_agent: Partial<Agent> = {
            name: 'Test Agent',
            model: 'gemini-1.5-flash',
            active_model_slot: 1
        };

        it('resolves basic agent config', () => {
            const config = resolve_agent_model_config(base_agent as Agent);
            expect(config.model_id).toBe('gemini-1.5-flash');
            expect(config.provider).toBe('google');
        });

        it('respects global overrides for default agents', () => {
            const config = resolve_agent_model_config(base_agent as Agent, 'ollama:phi3');
            expect(config.model_id).toBe('ollama:phi3');
            expect(config.provider).toBe('ollama');
        });

        it('handles multi-slot overrides (Slot 2)', () => {
            const agent: Partial<Agent> = {
                ...base_agent,
                active_model_slot: 2,
                model_2: 'claude-3-sonnet',
                model_config2: { modelId: 'claude-3-sonnet', provider: 'anthropic', apiKey: 'key' }
            };
            const config = resolve_agent_model_config(agent as Agent);
            expect(config.model_id).toBe('claude-3-sonnet');
            expect(config.provider).toBe('anthropic');
        });

        it('handles multi-slot overrides (Slot 3)', () => {
            const agent: Partial<Agent> = {
                ...base_agent,
                active_model_slot: 3,
                model_3: 'gpt-4',
                model_config3: { modelId: 'gpt-4', provider: 'openai', apiKey: 'key' }
            };
            const config = resolve_agent_model_config(agent as Agent);
            expect(config.model_id).toBe('gpt-4');
            expect(config.provider).toBe('openai');
        });
    });

    describe('get_model_color', () => {
        it('returns correct Tailwind classes for known providers', () => {
            expect(get_model_color('gpt-4')).toContain('emerald');
            expect(get_model_color('claude')).toContain('zinc');
            expect(get_model_color('gemini')).toContain('green');
            expect(get_model_color('llama')).toContain('amber');
            expect(get_model_color('deepseek')).toContain('cyan');
        });

        it('returns fallback for unknown models', () => {
            expect(get_model_color('mystery')).toBe('text-zinc-400 border-zinc-800 bg-zinc-900');
        });
    });
});

// Metadata: [model_utils_test]

// Metadata: [model_utils_test]
