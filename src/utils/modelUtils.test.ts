import { describe, it, expect } from 'vitest';
import { resolveTechnicalModelId, resolveProvider, resolveAgentModelConfig, getModelColor } from './modelUtils';

describe('modelUtils', () => {

    describe('resolveTechnicalModelId', () => {
        it('resolves known friendly names to technical IDs', () => {
            expect(resolveTechnicalModelId('Gemini 1.5 Pro')).toBe('gemini-1.5-pro');
            expect(resolveTechnicalModelId('Claude Opus 4.5')).toBe('claude-4.5-opus');
            expect(resolveTechnicalModelId('Mixtral 8x7B (Groq)')).toBe('mixtral-8x7b-32768');
        });

        it('returns the input if no mapping exists', () => {
            expect(resolveTechnicalModelId('Unknown Model Variant')).toBe('Unknown Model Variant');
            expect(resolveTechnicalModelId('gpt-4')).toBe('gpt-4'); // Already technical or unknown
        });

        it('returns "unknown" if undefined or empty', () => {
            expect(resolveTechnicalModelId(undefined)).toBe('unknown');
            expect(resolveTechnicalModelId('')).toBe('unknown');
        });
    });

    describe('resolveProvider', () => {
        it('resolves standard prefixes to valid provider IDs', () => {
            expect(resolveProvider('gpt-4-turbo')).toBe('openai');
            expect(resolveProvider('o4-mini')).toBe('openai');
            expect(resolveProvider('claude-3-sonnet')).toBe('anthropic');
            expect(resolveProvider('gemini-pro')).toBe('google');
            expect(resolveProvider('llama-3')).toBe('meta');
            expect(resolveProvider('llama-3-groq-tool-use')).toBe('groq');
            expect(resolveProvider('grok-1')).toBe('xai');
            expect(resolveProvider('deepseek-coder')).toBe('deepseek');
            expect(resolveProvider('mistral-large')).toBe('mistral');
            expect(resolveProvider('qwen-max')).toBe('alibaba');
            expect(resolveProvider('inception-v1')).toBe('inception');
        });

        it('defaults to google for unknown models to fallback to default system tier', () => {
            expect(resolveProvider('random-model-name')).toBe('google');
            expect(resolveProvider('')).toBe('google');
        });
    });

    describe('resolveAgentModelConfig', () => {
        it('parses primary slot correctly', () => {
            const agent = {
                modelConfig: { modelId: 'gpt-4o', provider: 'openai' },
                activeModelSlot: 1
            };
            const result = resolveAgentModelConfig(agent as any);
            expect(result.modelId).toBe('gpt-4o');
            expect(result.provider).toBe('openai');
        });

        it('falls back to string parsing if modelConfig is missing', () => {
            const agent = {
                model: 'claude-3-opus',
                activeModelSlot: 1
            };
            const result = resolveAgentModelConfig(agent as any);
            expect(result.modelId).toBe('claude-3-opus');
            expect(result.provider).toBe('anthropic');
        });

        it('handles slot 2 correctly', () => {
            const agent = {
                model: 'gpt-4o',
                modelConfig: { modelId: 'gpt-4o', provider: 'openai' },
                activeModelSlot: 2,
                modelConfig2: { modelId: 'gemini-1.5-pro', provider: 'google' }
            };
            const result = resolveAgentModelConfig(agent as any);
            expect(result.modelId).toBe('gemini-1.5-pro');
            expect(result.provider).toBe('google');
        });

        it('handles slot 3 correctly', () => {
            const agent = {
                model: 'gpt-4o',
                activeModelSlot: 3,
                model3: 'grok-1.5',
                modelConfig3: { modelId: 'grok-1.5', provider: 'xai' }
            };
            const result = resolveAgentModelConfig(agent as any);
            expect(result.modelId).toBe('grok-1.5');
            expect(result.provider).toBe('xai');
        });
    });

    describe('getModelColor', () => {
        it('returns proper tailwind classes for top providers', () => {
            expect(getModelColor('gpt-4')).toContain('emerald');
            expect(getModelColor('claude')).toContain('zinc');
            expect(getModelColor('gemini')).toContain('blue');
            expect(getModelColor('llama')).toContain('amber');
            expect(getModelColor('groq')).toContain('amber');
            expect(getModelColor('deepseek')).toContain('cyan');
            expect(getModelColor('grok')).toContain('zinc-100');
        });

        it('returns fallback styling for unknown or invalid inputs', () => {
            const fallback = 'text-zinc-400 border-zinc-800 bg-zinc-900';
            expect(getModelColor('Unknown Provider')).toBe(fallback);
            expect(getModelColor(undefined as any)).toBe(fallback);
        });
    });

});
