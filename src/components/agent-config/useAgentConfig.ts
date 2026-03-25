import { useReducer, useCallback } from 'react';
import { configReducer } from '../../hooks/useAgentForm';
import { TadpoleOSService } from '../../services/tadpoleosService';
import { EventBus } from '../../services/eventBus';
import { useRoleStore } from '../../stores/roleStore';
import { useModelStore } from '../../stores/modelStore';
import { resolveAgentModelConfig, resolveProvider } from '../../utils/modelUtils';
import { i18n } from '../../i18n';
import type { Agent } from '../../types';

export function useAgentConfig(agent: Agent | undefined, onUpdate: (id: string, updates: Partial<Agent>) => void, onClose: () => void) {
    const [state, dispatch] = useReducer(configReducer, {
        mainTab: 'cognition',
        activeTab: 'primary',
        identity: {
            name: agent?.name || 'New Neural Node',
            role: agent?.role || 'assistant'
        },
        slots: {
            primary: {
                provider: agent?.modelConfig?.provider || (agent?.model ? resolveProvider(agent.model) : 'google'),
                model: agent?.model || 'gemini-1.5-flash',
                temperature: agent?.modelConfig?.temperature ?? 0.7,
                systemPrompt: agent?.modelConfig?.systemPrompt ?? '',
                skills: agent?.modelConfig?.skills ?? agent?.skills ?? [],
                workflows: agent?.modelConfig?.workflows ?? agent?.workflows ?? []
            },
            secondary: {
                provider: agent?.modelConfig2?.provider || resolveProvider(agent?.model2 ?? 'Claude Opus 4.5'),
                model: agent?.model2 ?? 'Claude Opus 4.5',
                temperature: agent?.modelConfig2?.temperature ?? 0.5,
                systemPrompt: agent?.modelConfig2?.systemPrompt ?? '',
                skills: agent?.modelConfig2?.skills ?? [],
                workflows: agent?.modelConfig2?.workflows ?? []
            },
            tertiary: {
                provider: agent?.modelConfig3?.provider || resolveProvider(agent?.model3 ?? 'LLaMA 4 Maverick'),
                model: agent?.model3 ?? 'LLaMA 4 Maverick',
                temperature: agent?.modelConfig3?.temperature ?? 0.9,
                systemPrompt: agent?.modelConfig3?.systemPrompt ?? '',
                skills: agent?.modelConfig3?.skills ?? [],
                workflows: agent?.modelConfig3?.workflows ?? []
            }
        },
        voice: {
            voiceId: agent?.voiceId || 'alloy',
            voiceEngine: agent?.voiceEngine || 'browser',
            sttEngine: agent?.sttEngine || 'groq'
        },
        ui: {
            directMessage: '',
            saving: false,
            themeColor: agent?.themeColor || '#10b981',
            newRoleName: '',
            showPromote: false
        },
        governance: {
            budgetUsd: agent?.budgetUsd || 0,
            requiresOversight: agent?.requiresOversight || false
        },
        mcpTools: agent?.mcpTools || []
    });

    const addRole = useRoleStore(s => s.addRole);

    const handleRoleChange = useCallback((newRole: string) => {
        dispatch({ type: 'RESET_ROLE', role: newRole });
    }, []);

    const handleProviderChange = useCallback((slot: 'primary' | 'secondary' | 'tertiary', val: string) => {
        dispatch({ type: 'UPDATE_SLOT', slot, field: 'provider', value: val });
        const providerModels = useModelStore.getState().models.filter(m => m.provider === val);
        if (providerModels.length > 0) {
            dispatch({ type: 'UPDATE_SLOT', slot, field: 'model', value: providerModels[0].name });
        } else {
            dispatch({ type: 'UPDATE_SLOT', slot, field: 'model', value: '' });
        }
    }, []);

    const handleSave = useCallback(async () => {
        dispatch({ type: 'SET_UI', field: 'saving', value: true });
        try {
            const { identity, slots, voice, ui, governance, mcpTools } = state;
            const updates: Partial<Agent> = {
                role: identity.role,
                name: identity.name,
                budgetUsd: governance.budgetUsd,
                requiresOversight: governance.requiresOversight,
                voiceId: voice.voiceId,
                voiceEngine: voice.voiceEngine,
                sttEngine: voice.sttEngine,
                themeColor: ui.themeColor,
                mcpTools,
                category: agent?.category || 'user'
            };

            const buildConfig = (slotKey: keyof typeof slots) => {
                const slot = slots[slotKey];
                return {
                    modelId: slot.model,
                    provider: slot.provider,
                    temperature: slot.temperature,
                    systemPrompt: slot.systemPrompt,
                    skills: slot.skills,
                    workflows: slot.workflows
                };
            };

            updates.model = slots.primary.model;
            updates.modelConfig = buildConfig('primary');
            updates.model2 = slots.secondary.model;
            updates.modelConfig2 = buildConfig('secondary');
            updates.model3 = slots.tertiary.model;
            updates.modelConfig3 = buildConfig('tertiary');
            updates.skills = [...new Set([...slots.primary.skills, ...slots.secondary.skills, ...slots.tertiary.skills])];
            updates.workflows = [...new Set([...slots.primary.workflows, ...slots.secondary.workflows, ...slots.tertiary.workflows])];

            onUpdate(agent?.id || 'new', updates);
            onClose();

            EventBus.emit({
                source: 'System',
                text: i18n.t('agent_config.agent_updated', { name: identity.name }),
                severity: 'success'
            });
        } catch (error) {
            console.error('[ConfigPanel] Save Failed:', error);
            EventBus.emit({
                source: 'System',
                text: i18n.t('agent_config.save_failed'),
                severity: 'error'
            });
        } finally {
            dispatch({ type: 'SET_UI', field: 'saving', value: false });
        }
    }, [state, agent, onUpdate, onClose]);

    const handlePause = useCallback(async () => {
        if (!agent?.id) return;
        const success = await TadpoleOSService.pauseAgent(agent.id);
        onUpdate(agent.id, { status: 'idle' });
        EventBus.emit({
            source: 'System',
            text: success ? i18n.t('agent_config.agent_paused', { name: state.identity.name }) : i18n.t('agent_config.agent_paused_locally', { name: state.identity.name }),
            severity: 'info'
        });
    }, [agent?.id, state.identity.name, onUpdate]);

    const handleResume = useCallback(async () => {
        if (!agent?.id) return;
        const success = await TadpoleOSService.resumeAgent(agent.id);
        onUpdate(agent.id, { status: 'active' });
        EventBus.emit({
            source: 'System',
            text: success ? i18n.t('agent_config.agent_resumed', { name: state.identity.name }) : i18n.t('agent_config.agent_resumed_locally', { name: state.identity.name }),
            severity: 'success'
        });
    }, [agent?.id, state.identity.name, onUpdate]);

    const handleSendMessage = useCallback(async () => {
        if (!state.ui.directMessage.trim() || !agent?.id) return;
        const { modelId, provider } = resolveAgentModelConfig(agent);
        await TadpoleOSService.sendCommand(agent.id, state.ui.directMessage, modelId, provider);
        EventBus.emit({ source: 'User', text: `→ ${state.identity.name}: ${state.ui.directMessage}`, severity: 'info' });
        dispatch({ type: 'SET_UI', field: 'directMessage', value: '' });
    }, [state.ui.directMessage, state.identity.name, agent]);

    const handlePromote = useCallback(() => {
        if (!state.ui.newRoleName.trim()) {
            EventBus.emit({ text: i18n.t('agent_config.enter_role_name'), severity: 'warning', source: 'System' });
            return;
        }

        addRole(state.ui.newRoleName, {
            skills: state.slots[state.activeTab].skills,
            workflows: state.slots[state.activeTab].workflows
        });

        EventBus.emit({
            text: i18n.t('agent_config.role_saved', { name: state.ui.newRoleName }),
            severity: 'success',
            source: 'System'
        });

        dispatch({ type: 'RESET_ROLE', role: state.ui.newRoleName });
        dispatch({ type: 'SET_UI', field: 'showPromote', value: false });
        dispatch({ type: 'SET_UI', field: 'newRoleName', value: '' });
    }, [state.ui.newRoleName, state.slots, state.activeTab, addRole]);

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
