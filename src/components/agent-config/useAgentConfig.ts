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

import { useReducer, useCallback } from 'react';
import { config_reducer } from '../../hooks/use_agent_form';
import { tadpole_os_service } from '../../services/tadpoleos_service';
import { event_bus } from '../../services/event_bus';
import { use_role_store } from '../../stores/role_store';
import type { Role_State, Role_Actions } from '../../stores/role_store';
import { use_model_store } from '../../stores/model_store';
import { resolve_agent_model_config, resolve_provider } from '../../utils/model_utils';
import { ValidationUtils } from '../../utils/validation_utils';
import { i18n } from '../../i18n';
import type { Agent, Department } from '../../types';

/**
 * useAgentConfig
 * Primary hook for managing the state and actions of the agent configuration interface.
 * Orchestrates identity, model slots, capabilities, and governance.
 */
export function useAgentConfig(
    agent: Agent | undefined, 
    on_update: (id: string, updates: Partial<Agent>) => void, 
    on_close: () => void
) {
    const [state, dispatch] = useReducer(config_reducer, {
        main_tab: 'cognition',
        active_tab: agent?.active_model_slot === 2 ? 'secondary' : agent?.active_model_slot === 3 ? 'tertiary' : 'primary',
        identity: {
            name: agent?.name || 'New Neural Node',
            role: agent?.role || 'assistant',
            department: agent?.department || 'Engineering'
        },
        slots: {
            primary: {
                provider: agent?.model_config?.provider || (agent?.model ? resolve_provider(agent.model) : 'google'),
                model: agent?.model || 'gemini-1.5-flash',
                temperature: agent?.model_config?.temperature ?? 0.7,
                system_prompt: agent?.model_config?.system_prompt ?? '',
                skills: agent?.model_config?.skills ?? agent?.skills ?? [],
                workflows: agent?.model_config?.workflows ?? agent?.workflows ?? []
            },
            secondary: {
                provider: agent?.model_config2?.provider || resolve_provider(agent?.model_2 ?? 'Claude Opus 4.5'),
                model: agent?.model_2 ?? 'Claude Opus 4.5',
                temperature: agent?.model_config2?.temperature ?? 0.5,
                system_prompt: agent?.model_config2?.system_prompt ?? '',
                skills: agent?.model_config2?.skills ?? [],
                workflows: agent?.model_config2?.workflows ?? []
            },
            tertiary: {
                provider: agent?.model_config3?.provider || resolve_provider(agent?.model_3 ?? 'LLaMA 4 Maverick'),
                model: agent?.model_3 ?? 'LLaMA 4 Maverick',
                temperature: agent?.model_config3?.temperature ?? 0.9,
                system_prompt: agent?.model_config3?.system_prompt ?? '',
                skills: agent?.model_config3?.skills ?? [],
                workflows: agent?.model_config3?.workflows ?? []
            }
        },
        voice: {
            voice_id: agent?.voice_id || 'alloy',
            voice_engine: agent?.voice_engine || 'browser',
            stt_engine: agent?.stt_engine || 'groq'
        },
        ui: {
            direct_message: '',
            saving: false,
            theme_color: agent?.theme_color || '#10b981',
            new_role_name: '',
            show_promote: false
        },
        governance: {
            budget_usd: agent?.budget_usd || 0,
            requires_oversight: agent?.requires_oversight || false
        },
        mcp_tools: agent?.mcp_tools || [],
        connector_configs: agent?.connector_configs || []
    });

    const add_role = use_role_store((s: Role_State & Role_Actions) => s.add_role);

    const handle_role_change = useCallback((new_role: string) => {
        dispatch({ type: 'RESET_ROLE', role: new_role });
    }, []);

    const handle_provider_change = useCallback((slot: 'primary' | 'secondary' | 'tertiary', val: string) => {
        dispatch({ type: 'UPDATE_SLOT', slot, field: 'provider', value: val });
        const provider_models = use_model_store.getState().models.filter(m => m.provider === val);
        if (provider_models.length > 0) {
            dispatch({ type: 'UPDATE_SLOT', slot, field: 'model', value: provider_models[0].name });
        } else {
            dispatch({ type: 'UPDATE_SLOT', slot, field: 'model', value: '' });
        }
    }, []);

    const handle_save = useCallback(async () => {
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
            const { identity, slots, voice, ui, governance, mcp_tools } = state;
            const updates: Partial<Agent> = {
                role: identity.role,
                name: identity.name,
                department: identity.department as Department,
                budget_usd: governance.budget_usd,
                requires_oversight: governance.requires_oversight,
                voice_id: voice.voice_id,
                voice_engine: voice.voice_engine,
                stt_engine: voice.stt_engine,
                theme_color: ui.theme_color,
                mcp_tools,
                connector_configs: state.connector_configs,
                category: agent?.category || 'user'
            };

            const build_config = (slot_key: keyof typeof slots) => {
                const slot = slots[slot_key];
                return {
                    model_id: slot.model,
                    provider: slot.provider,
                    temperature: slot.temperature,
                    system_prompt: slot.system_prompt,
                    skills: slot.skills,
                    workflows: slot.workflows
                };
            };

            updates.model = slots.primary.model;
            updates.model_config = build_config('primary');
            updates.model_2 = slots.secondary.model;
            updates.model_config2 = build_config('secondary');
            updates.model_3 = slots.tertiary.model;
            updates.model_config3 = build_config('tertiary');
            updates.active_model_slot = state.active_tab === 'primary' ? 1 : state.active_tab === 'secondary' ? 2 : 3;
            updates.skills = [...new Set([...slots.primary.skills, ...slots.secondary.skills, ...slots.tertiary.skills])];
            updates.workflows = [...new Set([...slots.primary.workflows, ...slots.secondary.workflows, ...slots.tertiary.workflows])];

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

    const handle_pause = useCallback(async () => {
        if (!agent?.id) return;
        const success = await tadpole_os_service.pause_agent(agent.id);
        on_update(agent.id, { status: 'suspended' });
        event_bus.emit_log({
            source: 'System',
            text: success ? i18n.t('agent_config.agent_paused', { name: state.identity.name }) : i18n.t('agent_config.agent_paused_locally', { name: state.identity.name }),
            severity: 'info'
        });
    }, [agent?.id, state.identity.name, on_update]);

    const handle_resume = useCallback(async () => {
        if (!agent?.id) return;
        const success = await tadpole_os_service.resume_agent(agent.id);
        on_update(agent.id, { status: 'idle' });
        event_bus.emit_log({
            source: 'System',
            text: success ? i18n.t('agent_config.agent_resumed', { name: state.identity.name }) : i18n.t('agent_config.agent_resumed_locally', { name: state.identity.name }),
            severity: 'success'
        });
    }, [agent?.id, state.identity.name, on_update]);

    const handle_send_message = useCallback(async () => {
        if (!state.ui.direct_message.trim() || !agent?.id) return;
        const { model_id, provider } = resolve_agent_model_config(agent);
        await tadpole_os_service.send_command(agent.id, state.ui.direct_message, model_id, provider);
        event_bus.emit_log({ source: 'User', text: `→ ${state.identity.name}: ${state.ui.direct_message}`, severity: 'info' });
        dispatch({ type: 'SET_UI', field: 'direct_message', value: '' });
    }, [state.ui.direct_message, state.identity.name, agent]);

    const handle_promote = useCallback(() => {
        if (!state.ui.new_role_name.trim()) {
            event_bus.emit_log({ text: i18n.t('agent_config.enter_role_name'), severity: 'warning', source: 'System' });
            return;
        }

        add_role(state.ui.new_role_name, {
            skills: state.slots[state.active_tab].skills,
            workflows: state.slots[state.active_tab].workflows
        });

        event_bus.emit_log({
            text: i18n.t('agent_config.role_saved', { name: state.ui.new_role_name }),
            severity: 'success',
            source: 'System'
        });

        dispatch({ type: 'RESET_ROLE', role: state.ui.new_role_name });
        dispatch({ type: 'SET_UI', field: 'show_promote', value: false });
        dispatch({ type: 'SET_UI', field: 'new_role_name', value: '' });
    }, [state.ui.new_role_name, state.slots, state.active_tab, add_role]);

    return {
        state,
        dispatch,
        handle_role_change,
        handle_provider_change,
        handle_save,
        handle_pause,
        handle_resume,
        handle_send_message,
        handle_promote
    };
}

