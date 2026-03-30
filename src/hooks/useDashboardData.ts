import { useState, useEffect, useMemo, useRef } from 'react';
import { useAgentStore } from '../stores/agentStore';
import { useNodeStore } from '../stores/nodeStore';
import { useEngineStatus } from '../hooks/useEngineStatus';
import { EventBus, type LogEntry } from '../services/eventBus';
import { useWorkspaceStore } from '../stores/workspaceStore';
import { useRoleStore } from '../stores/roleStore';

export function useDashboardData() {
    const { isOnline } = useEngineStatus();
    const { agents: agentsList, fetchAgents, updateAgent, addAgent, initTelemetry } = useAgentStore();
    const { nodes, fetchNodes, discoverNodes, isLoading: nodesLoading } = useNodeStore();
    const [logs, setLogs] = useState<LogEntry[]>(() => EventBus.getHistory());
    const logsEndRef = useRef<HTMLDivElement>(null);

    const agentsCount = agentsList.length;

    useEffect(() => {
        fetchAgents();
        fetchNodes();
        const unsubscribeTelemetry = initTelemetry();

        const unsubscribeLogs = EventBus.subscribe((entry) => {
            setLogs(prev => [...prev, entry].slice(-100));
        });

        return () => {
            unsubscribeLogs();
            unsubscribeTelemetry();
        };
    }, [fetchAgents, fetchNodes, initTelemetry]);

    // Auto-scroll to bottom of logs
    useEffect(() => {
        if (logsEndRef.current) {
            logsEndRef.current.scrollIntoView({ behavior: 'smooth' });
        }
    }, [logs]);

    const { clusters, toggleClusterActive } = useWorkspaceStore();
    const assignedAgentIds = useMemo(() => new Set(clusters.flatMap(c => c.collaborators).map(String)), [clusters]);

    const roles = useRoleStore(s => s.roles);
    const availableRoles = useMemo(() => Object.keys(roles).sort(), [roles]);

    const activeAgents = useMemo(() =>
        agentsList.filter(a => (a.status === 'active' || a.status === 'speaking') && assignedAgentIds.has(a.id)).length,
        [agentsList, assignedAgentIds]);

    const totalCost = useMemo(() => agentsList.reduce((acc, curr) => acc + (curr.costUsd || 0), 0), [agentsList]);
    const totalBudget = useMemo(() => agentsList.reduce((acc, curr) => acc + (curr.budgetUsd || 0), 0), [agentsList]);
    const budgetUtil = totalBudget > 0 ? (totalCost / totalBudget) * 100 : 0;

    const totalTokens = useMemo(() => agentsList.reduce((acc, curr) => acc + (curr.tokensUsed || 0), 0), [agentsList]);
    const totalInputTokens = useMemo(() => agentsList.reduce((acc, curr) => acc + (curr.inputTokens || 0), 0), [agentsList]);
    const totalOutputTokens = useMemo(() => agentsList.reduce((acc, curr) => acc + (curr.outputTokens || 0), 0), [agentsList]);

    // Calculate Recruitment Velocity (Agents created in the last 24 hours)
    // We use a stable reference for "now" to satisfy React purity rules for useMemo.
    const [now] = useState(() => Date.now());
    const recruitVelocity = useMemo(() => {
        const twentyFourHoursAgo = now - (24 * 60 * 60 * 1000);
        return agentsList.filter(a => {
            const createdTime = a.createdAt ? new Date(a.createdAt).getTime() : 0;
            return createdTime > twentyFourHoursAgo;
        }).length;
    }, [agentsList, now]);


    const nodesRefined = useMemo(() => nodes.map(n => ({
        ...n,
        runningAgents: n.runningAgents || []
    })), [nodes]);

    return {
        isOnline,
        agentsList,
        agentsCount,
        activeAgents,
        totalCost,
        totalTokens,
        totalInputTokens,
        totalOutputTokens,
        budgetUtil,
        recruitVelocity,
        nodes: nodesRefined,
        nodesLoading,
        logs,
        logsEndRef,
        assignedAgentIds,
        availableRoles,
        clusters,
        toggleClusterActive,
        fetchAgents,
        updateAgent,
        addAgent,
        fetchNodes,
        discoverNodes
    };
}
