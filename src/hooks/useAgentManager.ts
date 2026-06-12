/**
 * @docs ARCHITECTURE:UI-Hooks
 * 
 * ### AI Assist Note
 * **Core technical resource for the Tadpole OS Sovereign infrastructure.**
 * Handles reactive state and high-fidelity user interactions.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: UI regression, hook desync, or API timeout.
 * - **Telemetry Link**: Search `[useAgentManager]` in observability traces.
 */

import { useState, useEffect, useMemo, useCallback } from 'react';
import { use_agent_store } from '../stores/agent_store';
import { agent_service } from '../services/agent_service';
import { use_role_store } from '../stores/role_store';
import { get_settings } from '../stores/settings_store';
import { resolve_provider } from '../utils/model_utils';
import { i18n } from '../i18n';
import type { Agent, Department } from '../types';

export function useAgentManager() {
    const agents = use_agent_store(s => s.agents);
    const roles = use_role_store(s => s.roles);

    const [selected_agent, set_selected_agent] = useState<Agent | null>(null);
    const [is_creating, set_is_creating] = useState(false);
    const [search_query, set_search_query] = useState('');
    const [filter_role, set_filter_role] = useState<string>('all');
    const [active_tab, set_active_tab] = useState<'user' | 'ai'>('user');

    useEffect(() => {
        if (agents.length === 0) {
            void agent_service.load_agents_into_store();
        }
    }, [agents.length]);

    const handle_update_agent = useCallback((id: string, updates: Partial<Agent>) => {
        if (is_creating) {
            // Optimistic add
            const new_agent = { ...selected_agent!, category: active_tab, ...updates } as Agent;
            use_agent_store.setState(s => ({ agents: [...s.agents, new_agent] }));
            agent_service.broadcast_update(new_agent.id, new_agent);
            
            set_is_creating(false);
            set_selected_agent(null);
        } else {
            void agent_service.update_agent(id, updates);
            if (selected_agent && selected_agent.id === id) {
                set_selected_agent(prev => prev ? { ...prev, ...updates } : null);
            }
        }
    }, [is_creating, selected_agent, active_tab]);

    const handle_add_new_click = useCallback(() => {
        if (agents.length >= 25) {
            alert(i18n.t('agent_manager.error_limit_reached'));
            return;
        }

        const settings = get_settings();
        const new_agent: Agent = {
            id: `node_${Math.random().toString(36).substring(2, 11)}`,
            name: i18n.t('agent_manager.placeholder_name'),
            role: active_tab === 'ai' ? `AI-${i18n.t('agent_manager.placeholder_role')}` : i18n.t('agent_manager.placeholder_role'),
            status: "idle",
            category: active_tab,
            model: settings.default_model,
            skills: [],
            workflows: [],
            tokens_used: 0,
            input_tokens: 0,
            output_tokens: 0,
            cost_usd: 0,
            budget_usd: 1.0,
            description: "",
            active_model_slot: 1,
            mcp_tools: [],
            requires_oversight: false,
            failure_count: 0,
            connector_configs: [],
            metadata: {},
            department: i18n.t('agent_manager.default_dept') as Department,
            theme_color: active_tab === 'user' ? "#22c55e" : "#10b981",
            model_config: {
                modelId: settings.default_model,
                provider: resolve_provider(settings.default_model),
                temperature: settings.default_temperature,
                systemPrompt: "",
                skills: [],
                workflows: []
            }
        };

        set_is_creating(true);
        set_selected_agent(new_agent);
    }, [agents.length, active_tab]);

    const filtered_agents = useMemo(() => {
        return agents.filter(agent => {
            let matches_tab = agent.category === active_tab;
            if (active_tab === 'ai' && agent.category === 'user') {
                matches_tab = agent.metadata?.has_participated_in_swarm === true || !!agent.active_mission;
            }

            const matches_search = agent.name.toLowerCase().includes(search_query.toLowerCase()) ||
                agent.role.toLowerCase().includes(search_query.toLowerCase());
            const matches_role = filter_role === 'all' || agent.role === filter_role;
            return matches_tab && matches_search && matches_role;
        });
    }, [agents, active_tab, search_query, filter_role]);

    const filter_roles = useMemo(() => {
        const all_role_names = Object.keys(roles).sort();
        return Array.from(new Set([
            ...all_role_names,
            ...agents.filter(a => a.category === active_tab).map(a => a.role)
        ])).sort();
    }, [roles, agents, active_tab]);

    return {
        agents: filtered_agents,
        total_agents: agents.length,
        user_agents_count: agents.filter(a => a.category === 'user').length,
        ai_agents_count: agents.filter(a => a.category === 'ai' || a.metadata?.has_participated_in_swarm === true).length,
        selected_agent,
        is_creating,
        search_query,
        filter_role,
        active_tab,
        filter_roles,
        
        set_selected_agent,
        set_is_creating,
        set_search_query,
        set_filter_role,
        set_active_tab,
        
        handle_update_agent,
        handle_add_new_click
    };
}

// Metadata: [useAgentManager]
