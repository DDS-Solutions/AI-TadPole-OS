/**
 * @docs ARCHITECTURE:Infrastructure
 * 
 * ### AI Assist Note
 * **Infrastructure**: Manages the model utils.test. 
 * Part of the Tadpole-OS core infrastructure.
 */

import { describe, it, expect } from 'vitest';
import { 
    resolve_technical_model_id, 
    resolve_provider, 
    resolve_agent_model_config, 
    get_model_color 
} from './model_utils';

describe('model_utils', () => {

    describe('resolve_technical_model_id', () => {
        it('resolves known friendly names to technical IDs', () => {
            expect(resolve_technical_model_id('Gemini 1.5 Pro')).toBe('gemini-1.5-pro');
            expect(resolve_technical_model_id('Claude Opus 4.5')).toBe('claude-4.5-opus');
            expect(resolve_technical_model_id('Mixtral 8x7B (Groq)')).toBe('mixtral-8x7b-32768');
        });

        it('returns the input if no mapping exists', () => {
            expect(resolve_technical_model_id('Unknown Model Variant')).toBe('Unknown Model Variant');
            expect(resolve_technical_model_id('gpt-4')).toBe('gpt-4'); // Already technical or unknown
        });

        it('returns "unknown" if undefined or empty', () => {
            expect(resolve_technical_model_id(undefined)).toBe('unknown');
            expect(resolve_technical_model_id('')).toBe('unknown');
        });
    });

    describe('resolve_provider', () => {
        it('resolves standard prefixes to valid provider IDs', () => {
            expect(resolve_provider('gpt-4-turbo')).toBe('openai');
            expect(resolve_provider('o4-mini')).toBe('openai');
            expect(resolve_provider('claude-3-sonnet')).toBe('anthropic');
            expect(resolve_provider('gemini-pro')).toBe('google');
            expect(resolve_provider('llama-3')).toBe('meta');
            expect(resolve_provider('llama-3-groq-tool-use')).toBe('groq');
            expect(resolve_provider('grok-1')).toBe('xai');
            expect(resolve_provider('deepseek-coder')).toBe('deepseek');
            expect(resolve_provider('mistral-large')).toBe('mistral');
            expect(resolve_provider('qwen-max')).toBe('alibaba');
            expect(resolve_provider('inception-v1')).toBe('inception');
        });

        it('defaults to google for unknown models to fallback to default system tier', () => {
            expect(resolve_provider('random-model-name')).toBe('google');
            expect(resolve_provider('')).toBe('google');
        });
    });

    describe('resolve_agent_model_config', () => {
        it('parses primary slot correctly', () => {
            const agent = {
                model_config: { model_id: 'gpt-4o', provider: 'openai' },
                active_model_slot: 1
            };
            const result = resolve_agent_model_config(agent as any);
            expect(result.model_id).toBe('gpt-4o');
            expect(result.provider).toBe('openai');
        });

        it('falls back to string parsing if model_config is missing', () => {
            const agent = {
                model: 'claude-3-opus',
                active_model_slot: 1
            };
            const result = resolve_agent_model_config(agent as any);
            expect(result.model_id).toBe('claude-3-opus');
            expect(result.provider).toBe('anthropic');
        });

        it('handles slot 2 correctly', () => {
            const agent = {
                model: 'gpt-4o',
                model_config: { model_id: 'gpt-4o', provider: 'openai' },
                active_model_slot: 2,
                model_config2: { model_id: 'gemini-1.5-pro', provider: 'google' }
            };
            const result = resolve_agent_model_config(agent as any);
            expect(result.model_id).toBe('gemini-1.5-pro');
            expect(result.provider).toBe('google');
        });

        it('handles slot 3 correctly', () => {
            const agent = {
                model: 'gpt-4o',
                active_model_slot: 3,
                model3: 'grok-1.5',
                model_config3: { model_id: 'grok-1.5', provider: 'xai' }
            };
            const result = resolve_agent_model_config(agent as any);
            expect(result.model_id).toBe('grok-1.5');
            expect(result.provider).toBe('xai');
        });
    });

    describe('get_model_color', () => {
        it('returns proper tailwind classes for top providers', () => {
            expect(get_model_color('gpt-4')).toContain('emerald');
            expect(get_model_color('claude')).toContain('zinc');
            expect(get_model_color('gemini')).toContain('blue');
            expect(get_model_color('llama')).toContain('amber');
            expect(get_model_color('groq')).toContain('amber');
            expect(get_model_color('deepseek')).toContain('cyan');
            expect(get_model_color('grok')).toContain('zinc-100');
        });

        it('returns fallback styling for unknown or invalid inputs', () => {
            const fallback = 'text-zinc-400 border-zinc-800 bg-zinc-900';
            expect(get_model_color('Unknown Provider')).toBe(fallback);
            expect(get_model_color(undefined as any)).toBe(fallback);
        });
    });
});

