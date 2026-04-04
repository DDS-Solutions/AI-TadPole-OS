import type { Mission_Cluster, Swarm_Proposal } from '../stores/workspace_store';

/**
 * Proposal_Service
 * Service for generating swarm intelligence proposals based on mission objectives.
 * Extracts heavy parsing logic from the Workspace_Store.
 * Refactored for strict snake_case compliance for backend parity.
 */
export class proposal_service {
    /**
     * generate_proposal
     * Analyzes a cluster's objective and generates a proposal for agent reconfigurations.
     */
    static generate_proposal(cluster: Mission_Cluster): Swarm_Proposal | null {
        if (!cluster || !cluster.objective) return null;

        const obj = cluster.objective.toLowerCase();
        let reasoning = `Alpha Node analysis of mission objective: "${cluster.objective}"`;
        const changes: Swarm_Proposal['changes'] = [];

        if (obj.includes('security') || obj.includes('patch') || obj.includes('vulnerability')) {
            reasoning += "\n- DEEP THREAT DETECTED: Elevating security protocols and switching to precision models.";
            cluster.collaborators.forEach(id => {
                changes.push({
                    agent_id: id,
                    proposed_role: 'Security Hardener',
                    proposed_model: 'DeepSeek V3.2',
                    added_skills: ['Scan Vulnerabilities', 'Code Audit']
                });
            });
        } else if (obj.includes('scale') || obj.includes('perf') || obj.includes('optimize')) {
            reasoning += "\n- PERFORMANCE BOTTLENECK: Shifting to high-efficiency models and DevOps automation.";
            cluster.collaborators.forEach(id => {
                changes.push({
                    agent_id: id,
                    proposed_role: 'Performance Architect',
                    proposed_model: 'Claude Sonnet 4.5',
                    added_skills: ['Check Server Health', 'View Logs']
                });
            });
        } else if (obj.includes('growth') || obj.includes('user') || obj.includes('feature')) {
            reasoning += "\n- USER-CENTRIC EXPANSION: Allocating creative models for feature synthesis.";
            cluster.collaborators.forEach(id => {
                changes.push({
                    agent_id: id,
                    proposed_role: 'Growth Catalyst',
                    proposed_model: 'GPT-5.2',
                    added_skills: ['Market Research', 'UX Audit']
                });
            });
        } else {
            reasoning += "\n- STANDARD OPS: Aligning team with baseline swarm efficiency.";
            cluster.collaborators.forEach(id => {
                changes.push({
                    agent_id: id,
                    added_skills: ['Deep Research']
                });
            });
        }

        return {
            cluster_id: cluster.id,
            reasoning,
            changes,
            timestamp: Date.now()
        };
    }
}
