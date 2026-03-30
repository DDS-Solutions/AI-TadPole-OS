import { EventBus } from './eventBus';
import { getSettings, useSettingsStore } from '../stores/settingsStore';
import type { MessagePart } from '../stores/sovereignStore';

/** Payload for agent update/status events from the WebSocket. */
export interface AgentUpdateEvent {
    type: 'agent:update' | 'agent:status' | 'engine:ui_invalidate';
    agentId?: string;
    status?: string;
    data?: Record<string, unknown> | Partial<import('../types').Agent>;
    resource?: 'agents' | 'missions' | 'system';
    id?: string;
}

/** Payload for engine health broadcast events. */
export interface EngineHealthEvent {
    type: 'engine:health';
    uptime?: number;
    agentCount?: number;
    activeMissions?: number;
    [key: string]: unknown;
}

/** Payload for inter-cluster handoff events. */
export interface HandoffEvent {
    type: 'agent:handoff';
    fromCluster: string;
    toCluster: string;
    agentId: string;
    payload?: Record<string, unknown>;
}

/** Payload for MCP tool pulse events. */
export interface McpPulseEvent {
    type: 'engine:mcp_pulse';
    tool: string;
    status: 'success' | 'error';
    latency: number;
}

/** Maximum number of reconnect attempts before giving up. */
const MAX_RETRIES = 10;
/** Initial backoff delay in ms. */
const INITIAL_BACKOFF = 2000;
/** Maximum backoff delay in ms. */
const MAX_BACKOFF = 30000;

/** Connection states for the socket. */
export type ConnectionState = 'connected' | 'connecting' | 'disconnected' | 'reconnecting';

type StateListener = (state: ConnectionState) => void;

type StoreBindings = {
    useAgentStore: typeof import('../stores/agentStore').useAgentStore;
    useSovereignStore: typeof import('../stores/sovereignStore').useSovereignStore;
    useTraceStore: typeof import('../stores/traceStore').useTraceStore;
};

/**
 * WebSocket client for streaming real-time logs from TadpoleOS.
 * Reads the connection URL from the centralized settings store.
 * Falls back to `ws://localhost:8000` if unconfigured.
 */
class TadpoleOSSocketClient {
    private socket: WebSocket | null = null;
    private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
    private isExplicitlyClosed = false;
    private retryCount = 0;
    private lastUrl = '';
    private lastKey = '';
    private storeBindingsPromise: Promise<StoreBindings> | null = null;

    // State Management
    private state: ConnectionState = 'disconnected';
    private stateListeners: StateListener[] = [];

    /** Subscribe to connection state changes. */
    subscribeStatus(listener: StateListener): () => void {
        this.stateListeners.push(listener);
        listener(this.state); // Immediate update
        return () => {
            this.stateListeners = this.stateListeners.filter(l => l !== listener);
        };
    }

    constructor() {
        // REACTIVE RECONNECTION: Watch for infrastructure changes
        // Initialize trackers
        const initial = getSettings();
        this.lastUrl = initial.TadpoleOSUrl;
        this.lastKey = initial.TadpoleOSApiKey;

        useSettingsStore.subscribe((state) => {
            const { TadpoleOSUrl, TadpoleOSApiKey } = state.settings;

            if (TadpoleOSUrl !== this.lastUrl || TadpoleOSApiKey !== this.lastKey) {
                this.lastUrl = TadpoleOSUrl;
                this.lastKey = TadpoleOSApiKey;

                if (this.isExplicitlyClosed) return;

                console.debug(`[TadpoleOS] Infrastructure settings changed. Reconnecting...`);
                this.disconnect();
                // Reset closure flag so connect() can proceed
                this.isExplicitlyClosed = false;
                this.connect();
            }
        });
    }

    private setState(newState: ConnectionState) {
        if (this.state !== newState) {
            this.state = newState;
            this.stateListeners.forEach(l => l(newState));
        }
    }

    // Agent Update Management
    private agentUpdateListeners: ((update: AgentUpdateEvent) => void)[] = [];

    subscribeAgentUpdates(listener: (update: AgentUpdateEvent) => void): () => void {
        this.agentUpdateListeners.push(listener);
        return () => {
            this.agentUpdateListeners = this.agentUpdateListeners.filter(l => l !== listener);
        };
    }

    // Health Management
    private healthListeners: ((health: EngineHealthEvent) => void)[] = [];

    subscribeHealth(listener: (health: EngineHealthEvent) => void): () => void {
        this.healthListeners.push(listener);
        return () => {
            this.healthListeners = this.healthListeners.filter(l => l !== listener);
        };
    }

    // Handoff Management
    private handoffListeners: ((handoff: HandoffEvent) => void)[] = [];

    subscribeHandoff(listener: (handoff: HandoffEvent) => void): () => void {
        this.handoffListeners.push(listener);
        return () => {
            this.handoffListeners = this.handoffListeners.filter(l => l !== listener);
        };
    }

    // MCP Pulse Management
    private mcpPulseListeners: ((pulse: McpPulseEvent) => void)[] = [];

    subscribePulse(listener: (pulse: McpPulseEvent) => void): () => void {
        this.mcpPulseListeners.push(listener);
        return () => {
            this.mcpPulseListeners = this.mcpPulseListeners.filter(l => l !== listener);
        };
    }

    // Audio Stream Management
    private audioStreamListeners: ((chunk: ArrayBuffer) => void)[] = [];

    subscribeAudioStream(listener: (chunk: ArrayBuffer) => void): () => void {
        this.audioStreamListeners.push(listener);
        return () => {
            this.audioStreamListeners = this.audioStreamListeners.filter(l => l !== listener);
        };
    }

    getConnectionState(): ConnectionState {
        return this.state;
    }

    private getStoreBindings(): Promise<StoreBindings> {
        if (!this.storeBindingsPromise) {
            this.storeBindingsPromise = Promise.all([
                import('../stores/agentStore'),
                import('../stores/sovereignStore'),
                import('../stores/traceStore')
            ]).then(([agentStore, sovereignStore, traceStore]) => ({
                useAgentStore: agentStore.useAgentStore,
                useSovereignStore: sovereignStore.useSovereignStore,
                useTraceStore: traceStore.useTraceStore
            }));
        }
        return this.storeBindingsPromise;
    }

    /** Opens the WebSocket connection and begins listening for events. */
    connect() {
        // Guard: Don't connect if already connecting or connected
        if (this.socket || this.reconnectTimer || this.state === 'connected') {
            return;
        }

        this.isExplicitlyClosed = false;
        this.setState('connecting');

        // Get URL from centralized settings, converting http/https to ws/wss
        const { TadpoleOSUrl, TadpoleOSApiKey } = getSettings();
        // Remove trailing slash if present, then replace http with ws
        const baseUrl = (TadpoleOSUrl || 'http://localhost:8000').trim().replace(/\/$/, '').replace(/^http/, 'ws');
        // Centralized token retrieval from SettingsStore
        const token = TadpoleOSApiKey || 'tadpole-dev-token-2026';
        // SEC-01: Token sent via Sec-WebSocket-Protocol header, not URL query params.
        // This prevents token leakage in browser history, proxy logs, and dev tools.
        const wsUrl = `${baseUrl}/v1/engine/ws`;

        try {
            const ws = new WebSocket(wsUrl, [`bearer.${token}`]);
            ws.binaryType = 'arraybuffer';
            this.socket = ws;

            ws.onopen = () => {
                if (this.socket !== ws) return; // Guard against stale connections
                this.retryCount = 0; // Reset on successful connection
                this.setState('connected');

                EventBus.emit({
                    source: 'System',
                    text: 'Connected to TadpoleOS Log Stream.',
                    severity: 'success'
                });
            };

            this.socket.onmessage = (event) => {
                if (event.data instanceof ArrayBuffer) {
                    this.audioStreamListeners.forEach(l => l(event.data));
                    return;
                }

                try {
                    const data = JSON.parse(event.data);
                    this.handleSocketMessage(data);
                } catch {
                    // Ignore malformed packets silently
                }
            };

            ws.onclose = () => {
                if (this.socket === ws) {
                    this.socket = null;
                    this.setState('disconnected');
                    if (!this.isExplicitlyClosed) {
                        this.scheduleReconnect();
                    }
                }
            };

            ws.onerror = () => {
                if (this.socket === ws) {
                    this.setState('disconnected');
                    ws.close();
                }
            };

        } catch (error) {
            console.error('[TadpoleOS] Connection failed:', error);
            this.setState('disconnected');
            this.scheduleReconnect();
        }
    }

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    private async handleSocketMessage(data: any) {
        const { useAgentStore, useSovereignStore, useTraceStore } = await this.getStoreBindings();
        const agentStore = useAgentStore.getState();
        const sovereignStore = useSovereignStore.getState();

        if (data.type === 'log' || data.type === 'thought') {
            const agentName = agentStore.agents.find(a => String(a.id) === String(data.agentId))?.name || data.agentName;

            EventBus.emit({
                id: data.id || data.requestId, // Deduplication key from backend
                source: data.agentId ? 'Agent' : 'System',
                agentId: data.agentId,
                agentName,
                text: data.text || data.message || JSON.stringify(data),
                severity: (data.level === 'error' ? 'error' : 'info'),
                metadata: data
            });

            // PRO-INTEGRATION: Bridge agent logs/thoughts to SovereignChat
            if (data.agentId && (data.type === 'thought' || (data.type === 'log' && data.level !== 'debug'))) {
                const agent = agentStore.agents.find(a => String(a.id) === String(data.agentId));
                const resolvedName = agent?.name || data.agentName || data.agentId;

                const messageId = data.messageId || data.requestId || ((typeof crypto !== 'undefined' && crypto.randomUUID) ? crypto.randomUUID() : Date.now().toString());

                const newPart: MessagePart = data.type === 'thought'
                    ? { type: 'thought', content: data.message || data.text || '', status: 'done' }
                    : { type: 'text', content: data.message || data.text || '' };

                const didUpdate = sovereignStore.appendMessagePart(messageId, newPart);
                if (!didUpdate) {
                    sovereignStore.addMessage({
                        id: messageId,
                        senderId: String(data.agentId),
                        senderName: resolvedName as string,
                        agentId: String(data.agentId),
                        text: data.message || data.text || '',
                        parts: [newPart],
                        scope: sovereignStore.activeScope
                    });
                }
            }
        } else if (data.type === 'agent:message') {
            const agentName = agentStore.agents.find(a => String(a.id) === String(data.agentId))?.name || data.agentName;

            EventBus.emit({
                id: data.id || data.messageId, // Pass through backend ID for deduplication
                source: 'Agent',
                agentId: data.agentId,
                agentName,
                text: data.text || 'Mission action complete.',
                severity: 'info',
                metadata: data
            });
            // Bridge to SovereignChat
            const agent = agentStore.agents.find(a => a.id === data.agentId);

            const resolvedName = agent?.name || data.agentName || data.agentId || 'Agent';
            const messageId = data.messageId || data.id || ((typeof crypto !== 'undefined' && crypto.randomUUID) ? crypto.randomUUID() : Date.now().toString());

            const newPart: MessagePart = { type: 'text' as const, content: data.text || data.message || 'Mission action complete.' };

            const didUpdate = sovereignStore.appendMessagePart(messageId as string, newPart);
            if (!didUpdate) {
                sovereignStore.addMessage({
                    id: messageId as string,
                    senderId: data.agentId || 'system',
                    senderName: resolvedName as string,
                    agentId: data.agentId, // Explicitly pass agentId for filtering
                    targetNode: resolvedName as string,
                    text: data.text || data.message || 'Mission action complete.',
                    parts: [newPart],
                    scope: sovereignStore.activeScope
                });
            }
        } else if (data.type === 'agent:update' || data.type === 'agent:status' || data.type === 'engine:ui_invalidate') {
            const normalizedData = data.type === 'agent:status'
                ? { ...data, type: 'agent:update', data: { status: data.status } }
                : data;

            if (data.type === 'engine:ui_invalidate') {
                EventBus.emit({
                    source: 'System',
                    text: `UI Invalidated: ${data.resource}${data.id ? ` (#${data.id})` : ''}`,
                    severity: 'info'
                });
            }

            this.agentUpdateListeners.forEach(l => l(normalizedData));
        } else if (data.type === 'engine:health') {
            this.healthListeners.forEach(l => l(data));
        } else if (data.type === 'agent:handoff') {
            this.handoffListeners.forEach(l => l(data));
        } else if (data.type === 'engine:mcp_pulse') {
            this.mcpPulseListeners.forEach(l => l(data));
        } else if (data.type === 'trace:span') {
            useTraceStore.getState().addSpan(data.span);
        } else if (data.type === 'trace:span_update') {
            useTraceStore.getState().updateSpan(data.spanId, data.update);
        } else if (data.type === 'engine:scheduled_job_complete') {
            EventBus.emit({
                source: 'System',
                text: `Scheduled Job '${data.job_name}' completed. Cost: $${data.cost_usd?.toFixed(4)}`,
                severity: data.status === 'failed' ? 'error' : 'success'
            });
        }
    }

    private scheduleReconnect() {
        if (this.retryCount >= MAX_RETRIES) {
            EventBus.emit({
                source: 'System',
                text: `TadpoleOS: Connection failed after ${MAX_RETRIES} attempts. Verify URL in Settings.`,
                severity: 'error'
            });
            return;
        }

        const delay = Math.min(INITIAL_BACKOFF * Math.pow(2, this.retryCount), MAX_BACKOFF);
        this.retryCount++;

        this.setState('reconnecting');
        if (this.reconnectTimer) clearTimeout(this.reconnectTimer);
        this.reconnectTimer = setTimeout(() => {
            this.reconnectTimer = null;
            this.connect();
        }, delay);
    }

    disconnect() {
        this.isExplicitlyClosed = true;
        this.retryCount = 0;
        this.setState('disconnected');
        if (this.reconnectTimer) clearTimeout(this.reconnectTimer);
        this.socket?.close();
    }
}

/** 
 * Lazy constructor to avoid side-effects at import time.
 * This prevents circular dependency issues and store initialization order bugs.
 */
let instance: TadpoleOSSocketClient | null = null;
export const getTadpoleOSSocket = () => {
    if (!instance) {
        instance = new TadpoleOSSocketClient();
    }
    return instance;
};

/** 
 * Type-safe Proxy for the TadpoleOSSocketClient.
 * Ensures the singleton is lazily initialized and correctly typed.
 */
export const TadpoleOSSocket = new Proxy({} as TadpoleOSSocketClient, {
    get(_, prop) {
        const inst = getTadpoleOSSocket();
        const value = inst[prop as keyof TadpoleOSSocketClient];
        if (typeof value === 'function') {
            return (value as (...args: unknown[]) => unknown).bind(inst);
        }
        return value;
    },
    set(_, prop, value) {
        const inst = getTadpoleOSSocket();
        const key = prop as keyof TadpoleOSSocketClient;
        // Type-safe property setting
        (inst[key] as typeof value) = value;
        return true;
    }
});

