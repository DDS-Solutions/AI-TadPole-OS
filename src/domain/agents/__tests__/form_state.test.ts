/**
 * @docs ARCHITECTURE:Quality:Verification
 *
 * ### AI Context Alignment
 * - **Subsystem**: Test Verification Suite / form_state.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { describe, it, expect } from 'vitest';
import { buildAgentFormState, serializeFormState } from '../form_state';
import type { Agent, AgentFormState } from '../../../contracts/agent';

describe('Agent Form State Domain Logic', () => {
    describe('buildAgentFormState', () => {
        it('correctly maps a full domain agent into hierarchical form state', () => {
            const mock_agent: Agent = {
                id: 'agent-alpha',
                name: 'Alpha Sentinel',
                role: 'Security Gatekeeper',
                department: 'Engineering',
                status: 'active',
                model: 'claude-3-5-sonnet',
                model_2: 'gpt-4o',
                model_3: 'gemma4:31b',
                active_model_slot: 2,
                category: 'sentinel',
                skills: ['audit', 'verify-changes'],
                workflows: ['/audit', '/test'],
                mcp_tools: ['read_file', 'grep_search'],
                budget_usd: 150,
                requires_oversight: true,
                theme_color: '#3b82f6',
                voice_id: 'echo',
                voice_engine: 'openai',
                stt_engine: 'whisper',
                connector_configs: [{ type: 'github', uri: 'https://github.com' }],
                model_config: {
                    modelId: 'claude-3-5-sonnet',
                    provider: 'anthropic',
                    temperature: 0.2,
                    systemPrompt: 'Primary system instructions',
                    reasoningDepth: 2,
                    actThreshold: 0.85,
                    skills: ['audit'],
                    workflows: ['/audit']
                }
            };

            const form = buildAgentFormState(mock_agent);

            expect(form.main_tab).toBe('cognition');
            expect(form.active_tab).toBe('secondary');
            expect(form.identity.name).toBe('Alpha Sentinel');
            expect(form.identity.role).toBe('Security Gatekeeper');
            expect(form.identity.department).toBe('Engineering');
            expect(form.slots.primary.model).toBe('claude-3-5-sonnet');
            expect(form.slots.primary.provider).toBe('anthropic');
            expect(form.slots.primary.temperature).toBe(0.2);
            expect(form.slots.primary.system_prompt).toBe('Primary system instructions');
            expect(form.slots.secondary.model).toBe('gpt-4o');
            expect(form.slots.tertiary.model).toBe('gemma4:31b');
            expect(form.mcp_tools).toEqual(['read_file', 'grep_search']);
            expect(form.governance.budget_usd).toBe(150);
            expect(form.governance.requires_oversight).toBe(true);
            expect(form.voice.voice_id).toBe('echo');
            expect(form.voice.voice_engine).toBe('openai');
            expect(form.voice.stt_engine).toBe('whisper');
            expect(form.ui.theme_color).toBe('#3b82f6');
        });

        it('safely handles empty or partial agent without undefined string fields', () => {
            const empty_agent = {} as Agent;
            const form = buildAgentFormState(empty_agent);

            expect(form.identity.name).toBe('');
            expect(form.identity.role).toBe('');
            expect(form.identity.department).toBe('Operations');
            expect(form.slots.primary.model).toBe('');
            expect(form.active_tab).toBe('primary');
            expect(form.voice.voice_id).toBe('alloy');
            expect(form.voice.voice_engine).toBe('browser');
            expect(form.governance.budget_usd).toBe(0);
            expect(form.governance.requires_oversight).toBe(false);
            expect(form.mcp_tools).toEqual([]);
        });

        it('maps slot 3 to tertiary active tab', () => {
            const agent = { active_model_slot: 3 } as Agent;
            const form = buildAgentFormState(agent);
            expect(form.active_tab).toBe('tertiary');
        });
    });

    describe('serializeFormState', () => {
        it('serializes hierarchical form state back into a flat domain patch', () => {
            const form_state: AgentFormState = {
                main_tab: 'cognition',
                active_tab: 'tertiary',
                identity: {
                    name: 'Beta Worker',
                    role: 'Code Optimizer',
                    department: 'Engineering'
                },
                voice: {
                    voice_id: 'nova',
                    voice_engine: 'groq',
                    stt_engine: 'groq'
                },
                slots: {
                    primary: {
                        provider: 'anthropic',
                        model: 'claude-3-5-sonnet',
                        temperature: 0.7,
                        system_prompt: 'Instructions 1',
                        reasoning_depth: 1,
                        act_threshold: 0.9,
                        skills: ['clean-code'],
                        workflows: ['/refactor']
                    },
                    secondary: {
                        provider: 'openai',
                        model: 'gpt-4o',
                        temperature: 0.5,
                        system_prompt: 'Instructions 2',
                        reasoning_depth: 2,
                        act_threshold: 0.8,
                        skills: ['architecture'],
                        workflows: ['/architecture-review']
                    },
                    tertiary: {
                        provider: 'ollama-cloud',
                        model: 'gemma4:31b',
                        temperature: 0.3,
                        system_prompt: 'Instructions 3',
                        reasoning_depth: 3,
                        act_threshold: 0.7,
                        skills: ['clean-code', 'rust-pro'],
                        workflows: ['/refactor', '/test']
                    }
                },
                mcp_tools: ['fetch_url'],
                governance: {
                    budget_usd: 250,
                    requires_oversight: true,
                    shadows_human_id: 'human-001',
                    economic_zone: 'PROD',
                    daily_spend_limit: 50
                },
                ui: {
                    direct_message: '',
                    saving: false,
                    theme_color: '#ef4444',
                    new_role_name: '',
                    show_promote: false
                },
                connector_configs: [{ type: 'postgres', uri: 'postgres://localhost' }]
            };

            const patch = serializeFormState(form_state);

            expect(patch.name).toBe('Beta Worker');
            expect(patch.role).toBe('Code Optimizer');
            expect(patch.department).toBe('Engineering');
            expect(patch.active_model_slot).toBe(3);
            expect(patch.model).toBe('claude-3-5-sonnet');
            expect(patch.model_2).toBe('gpt-4o');
            expect(patch.model_3).toBe('gemma4:31b');
            expect(patch.model_config?.provider).toBe('anthropic');
            expect(patch.model_config2?.provider).toBe('openai');
            expect(patch.model_config3?.provider).toBe('ollama-cloud');
            expect(patch.budget_usd).toBe(250);
            expect(patch.requires_oversight).toBe(true);
            expect(patch.shadows_human_id).toBe('human-001');
            expect(patch.economic_zone).toBe('PROD');
            expect(patch.daily_spend_limit).toBe(50);
            expect(patch.theme_color).toBe('#ef4444');
            expect(patch.mcp_tools).toEqual(['fetch_url']);
            expect(patch.connector_configs).toEqual([{ type: 'postgres', uri: 'postgres://localhost' }]);

            // Capabilities across slots should be aggregated and deduplicated
            expect(patch.skills).toEqual(['clean-code', 'architecture', 'rust-pro']);
            expect(patch.workflows).toEqual(['/refactor', '/architecture-review', '/test']);
        });
    });
});
