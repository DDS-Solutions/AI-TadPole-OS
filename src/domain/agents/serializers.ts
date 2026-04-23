/**
 * @docs ARCHITECTURE:Domain
 * 
 * ### AI Assist Note
 * **Agent Serializers**: Pure functions for transforming Domain patches back to backend DTOs.
 * Centralizes the bidirectional mapping between snake_case (frontend) and camelCase (backend).
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Improper mapping of optional fields causing 400 Bad Request from Axum backend, or field-name drift during contract updates.
 * - **Telemetry Link**: Search `[Serializer]` in UI traces.
 */

console.debug("[Serializer] Domain logic loaded");

import type { AgentPatch, AgentUpdateDto } from '../../contracts/agent';

/**
 * serialize_agent_update
 * Maps frontend Domain updates back to the backend AgentUpdateDto shape.
 * Implements the "One Merge Policy" for metadata vs first-class fields.
 */
export const serialize_agent_update = (patch: AgentPatch): AgentUpdateDto => {
    const dto: AgentUpdateDto = {};

    if (patch.role !== undefined) dto.role = patch.role;
    if (patch.name !== undefined) dto.name = patch.name;
    if (patch.department !== undefined) dto.department = patch.department;
    
    // Model resolution logic (Migrated from legacy mappers)
    if (patch.model !== undefined) {
        dto.modelId = patch.model;
        if (patch.model_config?.provider) {
            dto.provider = patch.model_config.provider;
        } else {
            const m = patch.model.toLowerCase();
            if (m.includes('gpt')) dto.provider = 'openai';
            else if (m.includes('claude')) dto.provider = 'anthropic';
            else if (m.includes('gemini')) dto.provider = 'google';
            else if (m.includes('llama') || m.includes('mixtral')) dto.provider = 'groq';
        }
    }

    if (patch.model_config !== undefined) {
        dto.modelId = patch.model_config.modelId;
        dto.provider = patch.model_config.provider;
        dto.modelConfig = patch.model_config;
    }

    if (patch.theme_color !== undefined) dto.themeColor = patch.theme_color;
    if (patch.active_model_slot !== undefined) dto.activeModelSlot = patch.active_model_slot;
    if (patch.model_config2 !== undefined) dto.modelConfig2 = patch.model_config2;
    if (patch.model_config3 !== undefined) dto.modelConfig3 = patch.model_config3;
    if (patch.budget_usd !== undefined) dto.budgetUsd = patch.budget_usd;
    
    if (patch.skills !== undefined) dto.skills = patch.skills;
    if (patch.workflows !== undefined) dto.workflows = patch.workflows;
    if (patch.mcp_tools !== undefined) dto.mcpTools = patch.mcp_tools;
    
    if (patch.voice_id !== undefined) dto.voiceId = patch.voice_id;
    if (patch.voice_engine !== undefined) dto.voiceEngine = patch.voice_engine;
    if (patch.category !== undefined) dto.category = patch.category;
    if (patch.created_at !== undefined) dto.createdAt = patch.created_at;
    
    if (patch.input_tokens !== undefined) dto.tokenUsage = { ...dto.tokenUsage, inputTokens: Math.floor(patch.input_tokens) };
    if (patch.output_tokens !== undefined) dto.tokenUsage = { ...dto.tokenUsage, outputTokens: Math.floor(patch.output_tokens) };
    if (patch.tokens_used !== undefined) dto.tokensUsed = Math.floor(patch.tokens_used);
    
    if (patch.current_task !== undefined) dto.currentTask = patch.current_task;
    if (patch.failure_count !== undefined) dto.failureCount = Math.floor(patch.failure_count);
    if (patch.connector_configs !== undefined) dto.connectorConfigs = patch.connector_configs;
    if (patch.metadata !== undefined) dto.metadata = patch.metadata;
    if (patch.requires_oversight !== undefined) dto.requiresOversight = patch.requires_oversight;

    return dto;
};


// Metadata: [serializers]

// Metadata: [serializers]
