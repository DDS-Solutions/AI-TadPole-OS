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
import { resolve_technical_model_id, resolve_provider, resolve_agent_model_config, get_model_color, parse_active_model_slot } from './model_utils';
import type { Agent } from '../types';

describe('model_utils', () => {
    describe('parse_active_model_slot', () => {
        it('parses numeric slot values', () => {
            expect(parse_active_model_slot(1)).toBe(1);
            expect(parse_active_model_slot(2)).toBe(2);
            expect(parse_active_model_slot(3)).toBe(3);
        });

        it('parses numeric string slot values', () => {
            expect(parse_active_model_slot('1')).toBe(1);
            expect(parse_active_model_slot('2')).toBe(2);
            expect(parse_active_model_slot('3')).toBe(3);
        });

        it('parses Rust backend slot names (planning, execution, default)', () => {
            expect(parse_active_model_slot('planning')).toBe(1);
            expect(parse_active_model_slot('execution')).toBe(2);
            expect(parse_active_model_slot('default')).toBe(3);
        });

        it('parses UI slot names (primary, secondary, tertiary)', () => {
            expect(parse_active_model_slot('primary')).toBe(1);
            expect(parse_active_model_slot('secondary')).toBe(2);
            expect(parse_active_model_slot('tertiary')).toBe(3);
        });

        it('fallbacks to slot 1 for unknown or undefined inputs', () => {
            expect(parse_active_model_slot(undefined)).toBe(1);
            expect(parse_active_model_slot(null)).toBe(1);
            expect(parse_active_model_slot('unknown')).toBe(1);
        });
    });
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
            expect(resolve_provider('minimax-m3:cloud')).toBe('ollama-cloud');
            expect(resolve_provider('ollama-cloud:some-model')).toBe('ollama-cloud');
            expect(resolve_provider('openai:gpt-4o')).toBe('openai');
            expect(resolve_provider('anthropic:claude-3.5-sonnet')).toBe('anthropic');
            expect(resolve_provider('google:gemini-1.5')).toBe('google');
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

        it('resolves slot 2 model using fallback model_2 even if model_config2 is undefined', () => {
            const agent: Partial<Agent> = {
                name: 'Elon',
                model: 'gemini-1.5-pro',
                model_2: 'gpt-4o-2024-08-06',
                model_3: 'claude-3-5-sonnet',
                active_model_slot: 2,
                // model_config2 is intentionally UNDEFINED
            };
            const config = resolve_agent_model_config(agent as Agent);
            expect(config.model_id).toBe('gpt-4o-2024-08-06');
            expect(config.provider).toBe('openai');
        });

        it('dynamically switches resolved model when active_model_slot updates mid-turn', () => {
            const agent: Partial<Agent> = {
                ...base_agent,
                status: 'thinking',
                model: 'gemini-1.5-pro',
                model_2: 'gpt-4o',
                model_3: 'claude-3-5-sonnet',
                active_model_slot: 1,
                model_config2: { modelId: 'gpt-4o', provider: 'openai' },
                model_config3: { modelId: 'claude-3-5-sonnet', provider: 'anthropic' }
            };

            // Initial slot 1
            expect(resolve_agent_model_config(agent as Agent).model_id).toBe('gemini-1.5-pro');

            // Switch to Slot 2 mid-turn
            agent.active_model_slot = 2;
            expect(resolve_agent_model_config(agent as Agent).model_id).toBe('gpt-4o');
            expect(resolve_agent_model_config(agent as Agent).provider).toBe('openai');

            // Switch to Slot 3 mid-turn
            agent.active_model_slot = 3;
            expect(resolve_agent_model_config(agent as Agent).model_id).toBe('claude-3-5-sonnet');
            expect(resolve_agent_model_config(agent as Agent).provider).toBe('anthropic');
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
            expect(get_model_color('mystery')).toBe('text-zinc-400 border-[color:var(--color-border)] bg-[color:var(--color-surface)]');
        });
    });
});

// Metadata: [model_utils_test]
