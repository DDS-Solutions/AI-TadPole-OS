/**
 * @docs ARCHITECTURE:Logic
 * 
 * ### AI Assist Note
 * **Logical Orchestrator**: Core hook for agent configuration state management. 
 * Consolidates identity, neural slots, voice identity, and governance into a unified `useReducer` flow for transactional updates.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Configuration save failure during network drop, `useReducer` state desync if external store updates trigger concurrently, or role promotion failure if `role_store` is read-only.
 * - **Telemetry Link**: Search for `[useAgentConfig]` or AGENT_UPDATE_TRANSACTION in tracing.
 */

import { useReducer, useMemo, useCallback, useEffect, useRef } from 'react';
import { config_reducer } from '../../hooks/useAgentForm';
import { tadpole_os_service } from '../../services/tadpoleos_service';
import { event_bus } from '../../services/event_bus';
import { use_role_store } from '../../stores/role_store';
import { use_model_store } from '../../stores/model_store';
import { resolve_agent_model_config, resolve_provider } from '../../utils/model_utils';
import { ValidationUtils } from '../../utils/validation_utils';
import { i18n } from '../../i18n';
import type { Agent, AgentPatch, Role_Definition, Agent_Model_Slot_Key, Department } from '../../contracts/agent';
import { buildAgentFormState, serializeFormState } from '../../domain/agents/form_state';
import { get_settings } from '../../stores/settings_store';

const slugify_blueprint_id = (value: string) =>
    value
        .trim()
        .toLowerCase()
        .replace(/[^a-z0-9]+/g, '-')
        .replace(/^-+|-+$/g, '')
    || 'role-blueprint';

const EMPTY_AGENT: Agent = {
    id: '',
    name: '',
    role: '',
    department: 'Operations',
    status: 'idle',
    model: 'gemini-1.5-flash',
    skills: [],
    workflows: [],
    cost_usd: 0,
    budget_usd: 0,
    theme_color: '#10b981',
    category: 'user',
    tokens_used: 0
};

/**
 * useAgentConfig
 * Primary hook for managing the state and actions of the agent configuration interface.
 * Orchestrates identity, model slots, capabilities, and governance.
 */
export function useAgentConfig(
    agent: Agent | undefined,
    on_update: (id: string, updates: AgentPatch) => void,
    on_close: () => void
) {
    // Phase 3: Use authoritative form state builder
    const initial_state = useMemo(() => {
        if (!agent) return buildAgentFormState(EMPTY_AGENT);
        return buildAgentFormState(agent);
    }, [agent]);

    const [state, dispatch] = useReducer(config_reducer, initial_state);
    const models = use_model_store((s) => s.models);
    const hydratedRef = useRef(false);

    // Phase 4: Sync empty model slots once model store hydrates
    useEffect(() => {
        if (models.length > 0 && !hydratedRef.current) {
            const slots: Agent_Model_Slot_Key[] = ['primary', 'secondary', 'tertiary'];
            slots.forEach(slotKey => {
                const slot = state.slots[slotKey];
                if (!slot.model) {
                    const provider_models = models.filter(m => m.provider === slot.provider);
                    if (provider_models.length > 0) {
                        dispatch({ type: 'UPDATE_SLOT', slot: slotKey, field: 'model', value: provider_models[0].name });
                    }
                }
            });
            hydratedRef.current = true;
        }
    }, [models, state.slots]);

    const lastAgentRef = useRef<Agent | undefined>(agent);

    // Phase 5: Reactive synchronization from parent agent prop changes back to form state reducer
    useEffect(() => {
        if (!agent) return;
        const lastAgent = lastAgentRef.current;
        lastAgentRef.current = agent;

        if (!lastAgent) return; // Skip on initial mount since state is already built

        // 1. Synchronize active tab slot selection using a slot index map
        const SLOT_INDEX_MAP: Record<number, Agent_Model_Slot_Key> = {
            1: 'primary',
            2: 'secondary',
            3: 'tertiary'
        };
        const expected_tab = (agent.active_model_slot ? SLOT_INDEX_MAP[agent.active_model_slot] : undefined) ?? 'primary';
        if (agent.active_model_slot !== lastAgent.active_model_slot) {
            dispatch({ type: 'SET_TAB', payload: expected_tab });
        }

        // 2. Synchronize model slots and providers
        const slotKeys: Agent_Model_Slot_Key[] = ['primary', 'secondary', 'tertiary'];
        slotKeys.forEach(slotKey => {
            const agentModel = slotKey === 'primary' ? agent.model : slotKey === 'secondary' ? agent.model_2 : agent.model_3;
            const lastAgentModel = slotKey === 'primary' ? lastAgent.model : slotKey === 'secondary' ? lastAgent.model_2 : lastAgent.model_3;

            const config = slotKey === 'primary' ? agent.model_config : slotKey === 'secondary' ? agent.model_config2 : agent.model_config3;
            const lastConfig = slotKey === 'primary' ? lastAgent.model_config : slotKey === 'secondary' ? lastAgent.model_config2 : lastAgent.model_config3;

            if (agentModel !== lastAgentModel) {
                dispatch({ type: 'UPDATE_SLOT', slot: slotKey, field: 'model', value: agentModel || '' });
            }

            const expectedProvider = config?.provider || resolve_provider(agentModel || '');
            const lastExpectedProvider = lastConfig?.provider || resolve_provider(lastAgentModel || '');
            if (expectedProvider !== lastExpectedProvider) {
                dispatch({ type: 'UPDATE_SLOT', slot: slotKey, field: 'provider', value: expectedProvider });
            }

            // Sync other configuration parameters (prompt, temperature, depth, threshold)
            const expectedPrompt = config?.systemPrompt || '';
            const lastPrompt = lastConfig?.systemPrompt || '';
            if (expectedPrompt !== lastPrompt) {
                dispatch({ type: 'UPDATE_SLOT', slot: slotKey, field: 'system_prompt', value: expectedPrompt });
            }

            const expectedTemp = config?.temperature ?? (slotKey === 'primary' ? 0.7 : slotKey === 'secondary' ? 0.5 : 0.9);
            const lastTemp = lastConfig?.temperature ?? (slotKey === 'primary' ? 0.7 : slotKey === 'secondary' ? 0.5 : 0.9);
            if (expectedTemp !== lastTemp) {
                dispatch({ type: 'UPDATE_SLOT', slot: slotKey, field: 'temperature', value: expectedTemp });
            }

            const expectedDepth = config?.reasoningDepth ?? 1;
            const lastDepth = lastConfig?.reasoningDepth ?? 1;
            if (expectedDepth !== lastDepth) {
                dispatch({ type: 'UPDATE_SLOT', slot: slotKey, field: 'reasoning_depth', value: expectedDepth });
            }

            const expectedAct = config?.actThreshold ?? 0.9;
            const lastAct = lastConfig?.actThreshold ?? 0.9;
            if (expectedAct !== lastAct) {
                dispatch({ type: 'UPDATE_SLOT', slot: slotKey, field: 'act_threshold', value: expectedAct });
            }
        });
    }, [agent]);

    const add_role = use_role_store((s) => s.add_role);

    const handleRoleChange = useCallback((new_role: string) => {
        dispatch({ type: 'RESET_ROLE', role: new_role });
    }, []);

    const handleProviderChange = useCallback((slot: Agent_Model_Slot_Key, val: string) => {
        dispatch({ type: 'UPDATE_SLOT', slot, field: 'provider', value: val });
        const provider_models = use_model_store.getState().models.filter(m => m.provider === val);
        if (provider_models.length > 0) {
            dispatch({ type: 'UPDATE_SLOT', slot, field: 'model', value: provider_models[0].name });
        } else {
            dispatch({ type: 'UPDATE_SLOT', slot, field: 'model', value: '' });
        }
    }, []);

    const handleSave = useCallback(async () => {
        const { identity, governance } = state;

        if (!ValidationUtils.is_valid_name(identity.name)) {
            event_bus.emit_log({ source: 'System', text: 'Invalid Neural Name: 2-64 characters required.', severity: 'warning' });
            return;
        }

        if (governance.budget_usd < 0) {
            event_bus.emit_log({ source: 'System', text: 'Fiscal Burn limit must be non-negative.', severity: 'warning' });
            return;
        }

        dispatch({ type: 'SET_UI', field: 'saving', value: true });
        try {
            // Phase 3: Use authoritative form state serializer
            const updates = serializeFormState(state);

            // Preserve specific metadata and category from original if missing
            if (agent?.metadata) {
                updates.metadata = { ...agent.metadata, ...updates.metadata };
            }
            if (agent?.category) {
                updates.category = agent.category;
            }

            await on_update(agent?.id || 'new', updates);
            on_close();

            event_bus.emit_log({
                source: 'System',
                text: i18n.t('agent_config.agent_updated', { name: identity.name }),
                severity: 'success'
            });
        } catch (error) {
            console.error('[ConfigPanel] Save Failed:', error);
            event_bus.emit_log({
                source: 'System',
                text: i18n.t('agent_config.save_failed'),
                severity: 'error'
            });
        } finally {
            dispatch({ type: 'SET_UI', field: 'saving', value: false });
        }
    }, [state, agent, on_update, on_close]);

    const handlePause = useCallback(async () => {
        if (!agent?.id) return;
        try {
            const success = await tadpole_os_service.pause_agent(agent.id);
            if (success) {
                await on_update(agent.id, { status: 'suspended' });
                event_bus.emit_log({
                    source: 'System',
                    text: i18n.t('agent_config.agent_paused', { name: state.identity.name }),
                    severity: 'info'
                });
            } else {
                event_bus.emit_log({
                    source: 'System',
                    text: i18n.t('agent_config.pause_failed') || 'Failed to pause agent.',
                    severity: 'error'
                });
            }
        } catch (error) {
            console.error('[ConfigPanel] Pause Failed:', error);
            event_bus.emit_log({
                source: 'System',
                text: 'Error pausing agent.',
                severity: 'error'
            });
        }
    }, [agent, state.identity.name, on_update]);

    const handleResume = useCallback(async () => {
        if (!agent?.id) return;
        try {
            const success = await tadpole_os_service.resume_agent(agent.id);
            if (success) {
                await on_update(agent.id, { status: 'idle' });
                event_bus.emit_log({
                    source: 'System',
                    text: i18n.t('agent_config.agent_resumed', { name: state.identity.name }),
                    severity: 'success'
                });
            } else {
                event_bus.emit_log({
                    source: 'System',
                    text: i18n.t('agent_config.resume_failed') || 'Failed to resume agent.',
                    severity: 'error'
                });
            }
        } catch (error) {
            console.error('[ConfigPanel] Resume Failed:', error);
            event_bus.emit_log({
                source: 'System',
                text: 'Error resuming agent.',
                severity: 'error'
            });
        }
    }, [agent, state.identity.name, on_update]);

    const handleSendMessage = useCallback(async () => {
        if (!state.ui.direct_message.trim() || !agent?.id) return;
        try {
            const { model_id, provider } = resolve_agent_model_config(agent, get_settings().default_model);
            await tadpole_os_service.send_command(agent.id, state.ui.direct_message, model_id, provider);
            event_bus.emit_log({ source: 'User', text: `→ ${state.identity.name}: ${state.ui.direct_message}`, severity: 'info' });
            dispatch({ type: 'SET_UI', field: 'direct_message', value: '' });
        } catch (error) {
            console.error('[ConfigPanel] Send command failed:', error);
            event_bus.emit_log({
                source: 'System',
                text: 'Failed to send message to agent.',
                severity: 'error'
            });
        }
    }, [state.ui.direct_message, state.identity.name, agent]);

    const handlePromote = useCallback(() => {
        if (!state.ui.new_role_name.trim()) {
            event_bus.emit_log({ text: i18n.t('agent_config.enter_role_name'), severity: 'warning', source: 'System' });
            return;
        }

        void (async () => {
            const blueprint_name = state.ui.new_role_name.trim();
            const active_slot = state.slots[state.active_tab];
            let synced_to_backend = true;

            const validDepartments = ['Executive', 'Operations', 'Engineering', 'Marketing', 'Sales', 'Product', 'Quality Assurance'] as const;
            const dept = state.identity.department;
            const department = (validDepartments as readonly string[]).includes(dept) ? (dept as Department) : 'Operations';

            const blueprint: Role_Definition = {
                id: slugify_blueprint_id(blueprint_name),
                name: blueprint_name,
                department,
                description: active_slot.system_prompt || `${state.identity.role} blueprint for ${state.identity.name}`,
                skills: active_slot.skills,
                workflows: active_slot.workflows,
                mcp_tools: state.mcp_tools,
                requires_oversight: state.governance.requires_oversight,
                model_id: active_slot.model,
                created_at: new Date().toISOString()
            };

            try {
                await tadpole_os_service.save_role_blueprint(blueprint);
            } catch (error) {
                synced_to_backend = false;
                console.error('[ConfigPanel] Role blueprint sync failed:', error);
                event_bus.emit_log({
                    text: `${i18n.t('agent_config.save_failed') || 'Save failed.'} Blueprint was kept local only.`,
                    severity: 'warning',
                    source: 'System'
                });
            }

            add_role(blueprint);

            event_bus.emit_log({
                text: synced_to_backend
                    ? i18n.t('agent_config.role_saved', { name: blueprint_name })
                    : `Blueprint "${blueprint_name}" saved locally only.`,
                severity: synced_to_backend ? 'success' : 'info',
                source: 'System'
            });

            dispatch({ type: 'RESET_ROLE', role: blueprint_name });
            dispatch({ type: 'SET_UI', field: 'show_promote', value: false });
            dispatch({ type: 'SET_UI', field: 'new_role_name', value: '' });
        })();
    }, [state, add_role]);

    return {
        state,
        dispatch,
        handleRoleChange,
        handleProviderChange,
        handleSave,
        handlePause,
        handleResume,
        handleSendMessage,
        handlePromote
    };
}

// Metadata: [useAgentConfig]
