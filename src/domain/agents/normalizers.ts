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

interface RobustInventoryItem {
    assetId?: string;
    asset_id?: string;
    assetName?: string;
    asset_name?: string;
    assetData?: string;
    asset_data?: string;
}

/**
 * RobustAgentDto
 * Internal type for the normalizer to handle both backend camelCase (DTO)
 * and legacy/mock snake_case properties without 'any' casting.
 */
interface RobustAgentDto extends Omit<Partial<AgentDto>, 'inventory'> {
    department?: string;
    tokens_used?: number;
    model_config?: Record<string, unknown>;
    workspace_path?: string;
    current_task?: string | null;
    mcp_tools?: string | string[];
    theme_color?: string;
    budget_usd?: number;
    cost_usd?: number;
    requires_oversight?: boolean;
    model_2?: string;
    model_3?: string;
    model_config2?: Record<string, unknown>;
    model_config3?: Record<string, unknown>;
    planningSlot?: Record<string, unknown> | null;
    planning_slot?: Record<string, unknown> | null;
    executionSlot?: Record<string, unknown> | null;
    execution_slot?: Record<string, unknown> | null;
    active_model_slot?: number;
    failure_count?: number;
    last_failure_at?: string;
    created_at?: string;
    last_pulse?: string | null;
    connector_configs?: Record<string, unknown>[];
    voice_id?: string;
    voice_engine?: Agent_Voice_Engine;
    stt_engine?: Agent_Stt_Engine;
    input_tokens?: number;
    output_tokens?: number;
    current_reasoning_turn?: number;
    reasoning_depth?: number;
    economic_zone?: string;
    daily_spend_limit?: number;
    daily_spent_accumulated?: number;
    balance?: number;
    inventory?: RobustInventoryItem[];
}

/**
 * normalize_agent_dto
 * Transforms the raw backend representation (camelCase Wire DTO) 
 * into the authoritative frontend domain model (snake_case).
 * 
 * ### Robustness Pattern
 * 1. **Partial Merges**: Prioritizes `dto` fields, but falls back to `existing_agent` to prevent identity loss.
 * 2. **Dual-Case Support**: Checks both camelCase (API) and snake_case (Internal/Mock) properties.
 */
export const normalize_agent_dto = (dto: AgentDto, workspace_path?: string, existing_agent?: Agent): Agent => {
    const d = dto as RobustAgentDto;
    const tokenUsage = dto.tokenUsage as Record<string, unknown> | undefined;
    const rawDto = dto as unknown as Record<string, unknown>;
    const nestedTokenUsage = rawDto.token_usage as Record<string, unknown> | undefined;

    const get_val = <T>(key_wire: keyof RobustAgentDto, key_domain: keyof RobustAgentDto, fallback: T): T => {
        const wire_val = d[key_wire] as unknown as T;
        if (wire_val !== undefined && wire_val !== null) return wire_val;

        const domain_val = d[key_domain] as unknown as T;
        if (domain_val !== undefined && domain_val !== null) return domain_val;

        if (existing_agent && (existing_agent as unknown as Record<string, unknown>)[key_domain] !== undefined) {
            return (existing_agent as unknown as Record<string, unknown>)[key_domain] as T;
        }
        return fallback;
    };

    const metadata = get_val<Record<string, unknown>>('metadata', 'metadata', {});
    const get_metadata_string = (key: string): string | undefined => {
        const value = metadata[key];
        return typeof value === 'string' && value.trim() ? value : undefined;
    };
    const get_non_empty_string = (
        key_wire: keyof RobustAgentDto,
        key_domain: keyof RobustAgentDto,
        fallback: string,
        metadata_key?: string,
    ): string => {
        const value = get_val<string | undefined>(key_wire, key_domain, undefined);
        if (typeof value === 'string' && value.trim()) return value;
        return (metadata_key ? get_metadata_string(metadata_key) : undefined) || fallback;
    };

    // 1. Department Normalization (Handles legacy mapping)
    const raw_dept = get_non_empty_string('department', 'department', 'Operations', 'department');
    const dept = (raw_dept === 'QA' ? 'Quality Assurance' : raw_dept) as Department;
    // 2. Status Mapping
    const raw_status = get_val<string>('status', 'status', 'idle');
    const status = (raw_status === 'working' ? 'active' : raw_status) as Agent_Status;

    const raw_task = d.currentTask ?? d.current_task ?? existing_agent?.current_task;

    // PERFORMANCE: Truncate extremely large current_task strings to prevent UI thread blocking
    const current_task = (typeof raw_task === 'string' && raw_task.length > 5000)
        ? raw_task.substring(0, 5000) + '... [TRUNCATED]'
        : raw_task;

    const parse_json_array = (wire_key: keyof RobustAgentDto, domain_key: keyof RobustAgentDto): string[] => {
        const val = get_val<string | string[]>(wire_key, domain_key, [] as string[]);
        if (Array.isArray(val)) return val;
        if (typeof val === 'string' && val.startsWith('[')) {
            try { return JSON.parse(val); } catch (e) {
                console.warn(`[Normalizer] Failed to parse JSON array for key '${String(wire_key)}':`, e);
                return [];
            }
        }
        return [];
    };

    const planningSlotVal = d.planningSlot ?? d.planning_slot;
    const executionSlotVal = d.executionSlot ?? d.execution_slot;

    const resolve_slot_value = <T>(
        incoming_slot: any,
        incoming_field: any,
        existing_val: T | undefined,
        extractor: (slot: any) => T | undefined
    ): T | undefined => {
        if (incoming_slot === null) return undefined;
        if (incoming_slot !== undefined) {
            return extractor(incoming_slot);
        }
        if (incoming_field === null) return undefined;
        if (incoming_field !== undefined) {
            return typeof incoming_field === 'object' && incoming_field !== null ? extractor(incoming_field) : incoming_field;
        }
        return existing_val;
    };

    const model_2 = resolve_slot_value(
        planningSlotVal,
        d.model2 ?? d.model_2 ?? (d.modelConfig2 ?? d.model_config2)?.modelId ?? (d.modelConfig2 ?? d.model_config2)?.model_id,
        existing_agent?.model_2,
        (slot) => slot.modelId ?? slot.model_id
    );

    const model_3 = resolve_slot_value(
        executionSlotVal,
        d.model3 ?? d.model_3 ?? (d.modelConfig3 ?? d.model_config3)?.modelId ?? (d.modelConfig3 ?? d.model_config3)?.model_id,
        existing_agent?.model_3,
        (slot) => slot.modelId ?? slot.model_id
    );

    const model_config2 = resolve_slot_value(
        planningSlotVal,
        d.modelConfig2 ?? d.model_config2,
        existing_agent?.model_config2,
        (slot) => slot
    );

    const model_config3 = resolve_slot_value(
        executionSlotVal,
        d.modelConfig3 ?? d.model_config3,
        existing_agent?.model_config3,
        (slot) => slot
    );

    return {
        id: dto.id || existing_agent?.id || 'unknown',
        name: get_non_empty_string('name', 'name', 'Unnamed Agent'),
        role: get_non_empty_string('role', 'role', 'AI Agent', 'role'),
        department: dept,
        description: get_val('description', 'description', ''),
        status: status,
        // TOKEN NORMALIZATION: Read from all possible backend formats.
        // Priority: tokenUsage.totalTokens (modern Rust) > tokenUsage.total_tokens (snake_case) > token_usage.totalTokens / total_tokens > tokensUsed (legacy) > tokens_used (snake_case)
        tokens_used: (
            (dto.tokenUsage?.totalTokens) ??
            (tokenUsage?.total_tokens as number | undefined) ??
            (nestedTokenUsage?.totalTokens as number | undefined) ??
            (nestedTokenUsage?.total_tokens as number | undefined) ??
            (d as unknown as { tokensUsed?: number }).tokensUsed ??
            d.tokens_used ??
            0
        ),
        model: get_val('modelId', 'model', get_val('model', 'model', 'Unknown')),
        model_config: get_val('modelConfig', 'model_config', undefined),
        workspace_path: workspace_path || get_val('workspace', 'workspace_path', undefined),
        current_task: current_task || undefined,
        skills: parse_json_array('skills', 'skills'),
        workflows: parse_json_array('workflows', 'workflows'),
        mcp_tools: parse_json_array('mcpTools', 'mcp_tools'),
        theme_color: get_val('themeColor', 'theme_color', undefined),
        budget_usd: get_val('budgetUsd', 'budget_usd', 0),
        cost_usd: get_val('costUsd', 'cost_usd', 0),
        requires_oversight: get_val('requiresOversight', 'requires_oversight', false),
        model_2: model_2,
        model_3: model_3,
        model_config2: model_config2,
        model_config3: model_config3,
        active_model_slot: (get_val('activeModelSlot', 'active_model_slot', 1) as 1 | 2 | 3),
        failure_count: get_val('failureCount', 'failure_count', 0),
        last_failure_at: get_val('lastFailureAt', 'last_failure_at', undefined),
        created_at: get_val('createdAt', 'created_at', undefined),
        last_pulse: get_val('lastPulse', 'last_pulse', null),
        connector_configs: get_val('connectorConfigs', 'connector_configs', []),
        metadata,
        voice_id: get_val('voiceId', 'voice_id', undefined),
        voice_engine: get_val<Agent_Voice_Engine | undefined>('voiceEngine', 'voice_engine', undefined),
        stt_engine: get_val<Agent_Stt_Engine | undefined>('sttEngine', 'stt_engine', get_metadata_string('stt_engine') as Agent_Stt_Engine | undefined),
        input_tokens: (
            dto.tokenUsage?.inputTokens ??
            (tokenUsage?.input_tokens as number | undefined) ??
            (nestedTokenUsage?.inputTokens as number | undefined) ??
            (nestedTokenUsage?.input_tokens as number | undefined) ??
            d.input_tokens ??
            existing_agent?.input_tokens ??
            0
        ),
        output_tokens: (
            dto.tokenUsage?.outputTokens ??
            (tokenUsage?.output_tokens as number | undefined) ??
            (nestedTokenUsage?.outputTokens as number | undefined) ??
            (nestedTokenUsage?.output_tokens as number | undefined) ??
            d.output_tokens ??
            existing_agent?.output_tokens ??
            0
        ),
        category: get_val('category', 'category', 'user'),
        current_reasoning_turn: get_val('currentReasoningTurn', 'current_reasoning_turn', undefined),
        reasoning_depth: get_val('reasoningDepth', 'reasoning_depth', undefined),
        economic_zone: get_val('economicZone', 'economic_zone', 'DEV'),
        daily_spend_limit: get_val('dailySpendLimit', 'daily_spend_limit', 0),
        daily_spent_accumulated: get_val('dailySpentAccumulated', 'daily_spent_accumulated', 0),
        balance: get_val('balance', 'balance', 0),
        inventory: ((d.inventory || existing_agent?.inventory || []) as RobustInventoryItem[]).map(item => ({
            asset_id: item.assetId || item.asset_id || '',
            asset_name: item.assetName || item.asset_name || '',
            asset_data: item.assetData || item.asset_data || undefined
        })),
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

// Metadata: [Normalizer]
