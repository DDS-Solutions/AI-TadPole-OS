/**
 * @docs ARCHITECTURE:Logic
 * 
 * ### AI Assist Note
 * **Logical Orchestrator**: Core hook for agent configuration state management. 
 * Consolidates identity, neural slots, voice identity, and governance into a unified `useReducer` flow for transactional updates.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Configuration save failure during network drop, `useReducer` state desync if external store updates trigger concurrently, or role promotion failure if `role_store` is read-only.
 * - **Telemetry Link**: Search for `[useAgentConfig]` or `AGENT_UPDATE_TRANSACTION` in tracing.
 */

import { useReducer, useMemo, useCallback, useEffect } from 'react';
import { config_reducer } from '../../hooks/useAgentForm';
import { tadpole_os_service } from '../../services/tadpoleos_service';
import { event_bus } from '../../services/event_bus';
import { use_role_store } from '../../stores/role_store';
import { use_model_store } from '../../stores/model_store';
import { resolve_agent_model_config } from '../../utils/model_utils';
import { ValidationUtils } from '../../utils/validation_utils';
import { i18n } from '../../i18n';
import type { Agent, AgentPatch, Role_Definition, Agent_Model_Slot_Key, Department } from '../../contracts/agent';
import { buildAgentFormState, serializeFormState } from '../../domain/agents/form_state';

const slugify_blueprint_id = (value: string) =>
    value
        .trim()
        .toLowerCase()
        .replace(/[^a-z0-9]+/g, '-')
        .replace(/^-+|-+$/g, '')
        || 'role-blueprint';

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
        if (!agent) return buildAgentFormState({} as Agent);
        return buildAgentFormState(agent);
    }, [agent]);

    const [state, dispatch] = useReducer(config_reducer, initial_state);
    const models = use_model_store((s) => s.models);

    // Phase 4: Sync empty model slots once model store hydrates
    useEffect(() => {
        if (models.length > 0) {
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
        }
    }, [models, state.slots]);

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

            on_update(agent?.id || 'new', updates);
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
        const success = await tadpole_os_service.pause_agent(agent.id);
        on_update(agent.id, { status: 'suspended' });
        event_bus.emit_log({
            source: 'System',
            text: success ? i18n.t('agent_config.agent_paused', { name: state.identity.name }) : i18n.t('agent_config.agent_paused_locally', { name: state.identity.name }),
            severity: 'info'
        });
    }, [agent, state.identity.name, on_update]);

    const handleResume = useCallback(async () => {
        if (!agent?.id) return;
        const success = await tadpole_os_service.resume_agent(agent.id);
        on_update(agent.id, { status: 'idle' });
        event_bus.emit_log({
            source: 'System',
            text: success ? i18n.t('agent_config.agent_resumed', { name: state.identity.name }) : i18n.t('agent_config.agent_resumed_locally', { name: state.identity.name }),
            severity: 'success'
        });
    }, [agent, state.identity.name, on_update]);

    const handleSendMessage = useCallback(async () => {
        if (!state.ui.direct_message.trim() || !agent?.id) return;
        const { get_settings } = await import('../../stores/settings_store');
        const { model_id, provider } = resolve_agent_model_config(agent, get_settings().default_model);
        await tadpole_os_service.send_command(agent.id, state.ui.direct_message, model_id, provider);
        event_bus.emit_log({ source: 'User', text: `→ ${state.identity.name}: ${state.ui.direct_message}`, severity: 'info' });
        dispatch({ type: 'SET_UI', field: 'direct_message', value: '' });
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

            const blueprint: Role_Definition = {
                id: slugify_blueprint_id(blueprint_name),
                name: blueprint_name,
                department: state.identity.department as Department,
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
                    text: `${i18n.t('agent_config.save_failed')} Blueprint was kept local only.`,
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
    }, [state.ui.new_role_name, state.slots, state.active_tab, state.identity.department, state.identity.name, state.identity.role, state.mcp_tools, state.governance.requires_oversight, add_role]);

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
