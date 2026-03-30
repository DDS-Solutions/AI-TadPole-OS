import type { MissionCluster, SwarmProposal } from '../stores/workspaceStore';

/**
 * Service for generating swarm intelligence proposals based on mission objectives.
 * Extracts heavy parsing logic from the workspaceStore.
 */
export class ProposalService {
    /**
     * Analyzes a cluster's objective and generates a proposal for agent reconfigurations.
     */
    static generateProposal(cluster: MissionCluster): SwarmProposal | null {
        if (!cluster || !cluster.objective) return null;

        const obj = cluster.objective.toLowerCase();
        let reasoning = `Alpha Node analysis of mission objective: "${cluster.objective}"`;
        const changes: SwarmProposal['changes'] = [];

        if (obj.includes('security') || obj.includes('patch') || obj.includes('vulnerability')) {
            reasoning += "\n- DEEP THREAT DETECTED: Elevating security protocols and switching to precision models.";
            cluster.collaborators.forEach(id => {
                changes.push({
                    agentId: id,
                    proposedRole: 'Security Hardener',
                    proposedModel: 'DeepSeek V3.2',
                    addedSkills: ['Scan Vulnerabilities', 'Code Audit']
                });
            });
        } else if (obj.includes('scale') || obj.includes('perf') || obj.includes('optimize')) {
            reasoning += "\n- PERFORMANCE BOTTLENECK: Shifting to high-efficiency models and DevOps automation.";
            cluster.collaborators.forEach(id => {
                changes.push({
                    agentId: id,
                    proposedRole: 'Performance Architect',
                    proposedModel: 'Claude Sonnet 4.5',
                    addedSkills: ['Check Server Health', 'View Logs']
                });
            });
        } else if (obj.includes('growth') || obj.includes('user') || obj.includes('feature')) {
            reasoning += "\n- USER-CENTRIC EXPANSION: Allocating creative models for feature synthesis.";
            cluster.collaborators.forEach(id => {
                changes.push({
                    agentId: id,
                    proposedRole: 'Growth Catalyst',
                    proposedModel: 'GPT-5.2',
                    addedSkills: ['Market Research', 'UX Audit']
                });
            });
        } else {
            reasoning += "\n- STANDARD OPS: Aligning team with baseline swarm efficiency.";
            cluster.collaborators.forEach(id => {
                changes.push({
                    agentId: id,
                    addedSkills: ['Deep Research']
                });
            });
        }

        return {
            clusterId: cluster.id,
            reasoning,
            changes,
            timestamp: Date.now()
        };
    }
}
