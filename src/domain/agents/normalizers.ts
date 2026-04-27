/**
 * @docs ARCHITECTURE:Domain
 * 
 * ### AI Assist Note
 * **Agent Normalizers**: Pure functions for transforming backend DTOs into Domain models.
 * Essential for absorbing breaking changes in the Rust API without impacting UI logic.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Failed parsing of JSON-stringified arrays (skills/workflows) or mapping drift between camelCase (wire) and snake_case (domain).
 * - **Telemetry Link**: Search `[Normalizer]` in UI traces.
 */

console.debug("[Normalizer] Domain logic loaded");

import type { 
    Agent, 
    AgentDto, 
    Department, 
    Agent_Status,
    Agent_Memory_Entry,
    Raw_Agent_Memory_Entry,
    Agent_Voice_Engine,
    Agent_Stt_Engine
} from '../../contracts/agent';

/**
 * normalize_agent_dto
 * Transforms the raw backend representation (camelCase Wire DTO) 
 * into the authoritative frontend domain model (snake_case).
 */
export const normalize_agent_dto = (dto: AgentDto, workspace_path?: string, existing_agent?: Agent): Agent => {
    // 1. Department Normalization (Handles legacy mapping)
    const raw_dept = dto.department || dto.metadata?.department || 'Operations';
    const dept = (raw_dept === 'QA' ? 'Quality Assurance' : raw_dept) as Department;

    // 2. Status Mapping
    const raw_status = dto.status || 'idle';
    const status = (raw_status === 'working' ? 'active' : raw_status) as Agent_Status;
    
    const has_current_task = Object.prototype.hasOwnProperty.call(dto, 'currentTask');
    const current_task = (status === 'active')
        ? (has_current_task ? (dto.currentTask ?? undefined) : existing_agent?.current_task)
        : (has_current_task ? (dto.currentTask ?? undefined) : undefined);

    const parse_json_array = (val: unknown): string[] => {
        if (Array.isArray(val)) return val;
        if (typeof val === 'string' && val.startsWith('[')) {
            try { return JSON.parse(val); } catch { return []; }
        }
        return [];
    };

    return {
        id: dto.id,
        name: dto.name || 'Unnamed Agent',
        role: dto.role || (dto.metadata?.role as string) || 'AI Agent',
        department: dept,
        description: dto.description || "",
        status: status,
        tokens_used: dto.tokensUsed || 0,
        model: dto.model || dto.modelId || dto.modelConfig?.modelId || 'Unknown',
        model_config: dto.modelConfig,
        workspace_path: workspace_path || dto.workspace,
        current_task: current_task || undefined,
        skills: parse_json_array(dto.skills),
        workflows: parse_json_array(dto.workflows),
        mcp_tools: parse_json_array(dto.mcpTools),
        theme_color: dto.themeColor,
        budget_usd: dto.budgetUsd || 0,
        cost_usd: dto.costUsd || 0,
        requires_oversight: dto.requiresOversight ?? false,
        model_2: dto.model2,
        model_3: dto.model3,
        model_config2: dto.modelConfig2,
        model_config3: dto.modelConfig3,
        active_model_slot: (dto.activeModelSlot as 1 | 2 | 3) || 1,
        failure_count: dto.failureCount ?? 0,
        last_failure_at: dto.lastFailureAt,
        created_at: dto.createdAt,
        last_pulse: dto.lastPulse,
        connector_configs: dto.connectorConfigs || [],
        metadata: dto.metadata || {},
        voice_id: dto.voiceId,
        voice_engine: dto.voiceEngine as Agent_Voice_Engine,
        stt_engine: (dto.sttEngine || dto.metadata?.stt_engine) as Agent_Stt_Engine,
        input_tokens: dto.tokenUsage?.inputTokens ?? 0,
        output_tokens: dto.tokenUsage?.outputTokens ?? 0,
        category: dto.category || 'user',
        current_reasoning_turn: dto.currentReasoningTurn,
        reasoning_depth: dto.reasoningDepth ?? dto.modelConfig?.reasoningDepth,
    };
};

/**
 * normalize_agent_memory_entry
 * Transforms a raw memory entry into a structured domain model.
 */
export const normalize_agent_memory_entry = (raw: Raw_Agent_Memory_Entry): Agent_Memory_Entry => {
    return {
        id: raw.id,
        text: raw.text || raw.content || '',
        mission_id: raw.mission_id || '',
        timestamp: typeof raw.timestamp === 'string' ? new Date(raw.timestamp).getTime() : (raw.timestamp || Date.now()),
        metadata: raw.metadata || {}
    };
};

// Metadata: [normalizers]

// Metadata: [normalizers]
