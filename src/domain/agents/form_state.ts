/**
 * @docs ARCHITECTURE:Domain
 * 
 * ### AI Assist Note
 * **Agent Form State Logic**: Dedicated helpers for converting between 
 * flat Domain models and grouped UI Form state.
 * Prevents logic duplication in the configuration component.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Status mismatch between domain and form, or incorrect slot mapping (primary/secondary/tertiary).
 * - **Telemetry Link**: Search `[FormState]` in UI traces.
 */

console.debug("[FormState] Domain logic loaded");

import type { Agent, AgentFormState, AgentPatch, Department } from '../../contracts/agent';
import { resolve_provider } from '../../utils/model_utils';

/**
 * buildAgentFormState
 * Converts a Domain Agent into a ready-to-use Form State for the configuration UI.
 */
export const buildAgentFormState = (agent: Agent): AgentFormState => {
    // Phase 4: Ensure provider/model consistency on hydration
    const primary_provider = agent.model_config?.provider || resolve_provider(agent.model);
    const secondary_provider = agent.model_config2?.provider || resolve_provider(agent.model_2 || 'claude');
    const tertiary_provider = agent.model_config3?.provider || resolve_provider(agent.model_3 || 'llama');

    return {
        main_tab: 'cognition',
        active_tab: agent.active_model_slot === 2 ? 'secondary' : agent.active_model_slot === 3 ? 'tertiary' : 'primary',
        identity: {
            name: agent.name,
            role: agent.role,
            department: agent.department
        },
        voice: {
            voice_id: agent.voice_id || 'alloy',
            voice_engine: agent.voice_engine || 'browser',
            stt_engine: agent.stt_engine || 'groq'
        },
        slots: {
            primary: {
                provider: primary_provider,
                model: agent.model,
                temperature: agent.model_config?.temperature ?? 0.7,
                system_prompt: agent.model_config?.systemPrompt ?? '',
                reasoning_depth: agent.model_config?.reasoningDepth ?? 1,
                act_threshold: agent.model_config?.actThreshold ?? 0.9,
                skills: agent.model_config?.skills ?? agent.skills ?? [],
                workflows: agent.model_config?.workflows ?? agent.workflows ?? []
            },
            secondary: {
                provider: secondary_provider,
                model: agent.model_2 ?? 'Claude Opus 4.5',
                temperature: agent.model_config2?.temperature ?? 0.5,
                system_prompt: agent.model_config2?.systemPrompt ?? '',
                reasoning_depth: agent.model_config2?.reasoningDepth ?? 1,
                act_threshold: agent.model_config2?.actThreshold ?? 0.9,
                skills: agent.model_config2?.skills ?? [],
                workflows: agent.model_config2?.workflows ?? []
            },
            tertiary: {
                provider: tertiary_provider,
                model: agent.model_3 ?? 'LLaMA 4 Maverick',
                temperature: agent.model_config3?.temperature ?? 0.9,
                system_prompt: agent.model_config3?.systemPrompt ?? '',
                reasoning_depth: agent.model_config3?.reasoningDepth ?? 1,
                act_threshold: agent.model_config3?.actThreshold ?? 0.9,
                skills: agent.model_config3?.skills ?? [],
                workflows: agent.model_config3?.workflows ?? []
            }
        },
        mcp_tools: agent.mcp_tools || [],
        governance: {
            budget_usd: agent.budget_usd || 0,
            requires_oversight: agent.requires_oversight || false
        },
        ui: {
            direct_message: '',
            saving: false,
            theme_color: agent.theme_color || '#10b981',
            new_role_name: '',
            show_promote: false
        },
        connector_configs: agent.connector_configs || []
    };
};

/**
 * serializeFormState
 * Aggregates hierarchical form state back into a flat Partial<Agent> (Domain Patch) for persistence.
 */
export const serializeFormState = (state: AgentFormState): AgentPatch => {
    const { identity, slots, voice, ui, governance, mcp_tools, connector_configs } = state;
    
    return {
        name: identity.name,
        role: identity.role,
        department: identity.department as Department,
        budget_usd: governance.budget_usd,
        requires_oversight: governance.requires_oversight,
        voice_id: voice.voice_id,
        voice_engine: voice.voice_engine,
        stt_engine: voice.stt_engine,
        theme_color: ui.theme_color,
        mcp_tools,
        connector_configs,
        model: slots.primary.model,
        model_2: slots.secondary.model,
        model_3: slots.tertiary.model,
        model_config: {
            modelId: slots.primary.model,
            provider: slots.primary.provider,
            temperature: slots.primary.temperature,
            systemPrompt: slots.primary.system_prompt,
            reasoningDepth: slots.primary.reasoning_depth,
            actThreshold: slots.primary.act_threshold,
            skills: slots.primary.skills,
            workflows: slots.primary.workflows
        },
        model_config2: {
            modelId: slots.secondary.model,
            provider: slots.secondary.provider,
            temperature: slots.secondary.temperature,
            systemPrompt: slots.secondary.system_prompt,
            reasoningDepth: slots.secondary.reasoning_depth,
            actThreshold: slots.secondary.act_threshold,
            skills: slots.secondary.skills,
            workflows: slots.secondary.workflows
        },
        model_config3: {
            modelId: slots.tertiary.model,
            provider: slots.tertiary.provider,
            temperature: slots.tertiary.temperature,
            systemPrompt: slots.tertiary.system_prompt,
            reasoningDepth: slots.tertiary.reasoning_depth,
            actThreshold: slots.tertiary.act_threshold,
            skills: slots.tertiary.skills,
            workflows: slots.tertiary.workflows
        },
        active_model_slot: state.active_tab === 'primary' ? 1 : state.active_tab === 'secondary' ? 2 : 3,
        skills: Array.from(new Set([
            ...slots.primary.skills,
            ...slots.secondary.skills,
            ...slots.tertiary.skills
        ])),
        workflows: Array.from(new Set([
            ...slots.primary.workflows,
            ...slots.secondary.workflows,
            ...slots.tertiary.workflows
        ]))
    };
};

// Metadata: [form_state]

// Metadata: [form_state]
