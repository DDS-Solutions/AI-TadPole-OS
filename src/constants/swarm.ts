/**
 * SWARM_NODE_STATUS
 * Standardized status codes for Swarm nodes.
 * Parity with Pulse_Node status in types/index.ts
 */
export const SWARM_NODE_STATUS = {
    IDLE: 0,
    BUSY: 1,
    ERROR: 2,
    DEGRADED: 3,
    MISSION_HUB: 4, // UI-specific extension
} as const;

export type SwarmNodeStatus = typeof SWARM_NODE_STATUS[keyof typeof SWARM_NODE_STATUS];
