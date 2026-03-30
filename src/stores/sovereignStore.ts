import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export type SovereignScope = 'agent' | 'cluster' | 'swarm';

export type MessagePart = 
    | { type: 'text', content: string }
    | { type: 'thought', content: string, status: 'thinking' | 'done' }
    | { type: 'tool', name: string, input: unknown, output?: unknown };

export interface ChatMessage {
    id: string;
    senderId: string;
    senderName: string;
    text: string;
    parts?: MessagePart[];
    timestamp: string;
    scope: SovereignScope;
    agentId?: string;
    isSubAgent?: boolean;
    lineage?: string[];
    targetNode?: string;
}

interface SovereignState {
    messages: ChatMessage[];
    messageIndexById: Record<string, number>;
    activeScope: SovereignScope;
    selectedAgentId: string | null;
    targetAgent: string;
    targetCluster: string;
    isDetached: boolean;

    // Actions
    addMessage: (msg: Omit<ChatMessage, 'id' | 'timestamp'> & { id?: string, timestamp?: string }) => void;
    updateMessage: (id: string, updates: Partial<ChatMessage>) => void;
    appendMessagePart: (id: string, part: MessagePart) => boolean;
    getMessageById: (id: string) => ChatMessage | undefined;
    setScope: (scope: SovereignScope) => void;
    setSelectedAgentId: (agentId: string | null) => void;
    setTargetAgent: (name: string) => void;
    setTargetCluster: (name: string) => void;
    setDetached: (detached: boolean) => void;
    clearHistory: () => void;
}

type PersistedSovereignState = Pick<
    SovereignState,
    'activeScope' | 'selectedAgentId' | 'targetAgent' | 'targetCluster' | 'isDetached'
>;

// Cross-window synchronization
const chatChannel = typeof window !== 'undefined' ? new BroadcastChannel('tadpole-chat-sync') : null;

export const useSovereignStore = create<SovereignState>()(
    persist(
        (set, get) => ({
            messages: [],
            messageIndexById: {},
            activeScope: 'agent',
            selectedAgentId: null,
            targetAgent: 'Agent of Nine',
            targetCluster: '',
            isDetached: false,

            addMessage: (msg) => {
                const newMsg = {
                    ...msg,
                    id: msg.id || ((typeof crypto !== 'undefined' && crypto.randomUUID) ? crypto.randomUUID() : Date.now().toString(36)),
                    timestamp: msg.timestamp || new Date().toISOString(),
                    parts: msg.parts || (msg.text ? [{ type: 'text', content: msg.text }] : [])
                };
                let wasAdded = false;
                set((state) => {
                    if (state.messageIndexById[newMsg.id] !== undefined) {
                        return state; // Deduplicate
                    }
                    wasAdded = true;
                    return {
                        messages: [...state.messages, newMsg],
                        messageIndexById: {
                            ...state.messageIndexById,
                            [newMsg.id]: state.messages.length
                        }
                    };
                });
                // Sync to other windows
                if (wasAdded) {
                    chatChannel?.postMessage({ type: 'ADD_MESSAGE', payload: newMsg });
                }
            },

            updateMessage: (id, updates) => {
                let updatedMsg: ChatMessage | undefined;
                set((state) => {
                    const idx = state.messageIndexById[id];
                    if (idx === undefined) return state;

                    const nextMessages = [...state.messages];
                    updatedMsg = { ...nextMessages[idx], ...updates };
                    nextMessages[idx] = updatedMsg;
                    return { messages: nextMessages };
                });
                if (updatedMsg) {
                    chatChannel?.postMessage({ type: 'UPDATE_MESSAGE', payload: updatedMsg });
                }
            },

            appendMessagePart: (id, part) => {
                let updatedMsg: ChatMessage | undefined;
                set((state) => {
                    const idx = state.messageIndexById[id];
                    if (idx === undefined) return state;

                    const currentMessage = state.messages[idx];
                    updatedMsg = {
                        ...currentMessage,
                        parts: [...(currentMessage.parts || []), part]
                    };

                    const nextMessages = [...state.messages];
                    nextMessages[idx] = updatedMsg;
                    return { messages: nextMessages };
                });

                if (updatedMsg) {
                    chatChannel?.postMessage({ type: 'UPDATE_MESSAGE', payload: updatedMsg });
                    return true;
                }
                return false;
            },

            getMessageById: (id) => {
                const state = get();
                const idx = state.messageIndexById[id];
                return idx === undefined ? undefined : state.messages[idx];
            },

            setScope: (activeScope) => {
                set({ activeScope });
                chatChannel?.postMessage({ type: 'SET_SCOPE', payload: activeScope });
            },

            setSelectedAgentId: (selectedAgentId) => {
                set({ selectedAgentId });
                chatChannel?.postMessage({ type: 'SET_AGENT', payload: selectedAgentId });
            },

            setTargetAgent: (targetAgent) => {
                set({ targetAgent });
                chatChannel?.postMessage({ type: 'SET_TARGET_AGENT', payload: targetAgent });
            },

            setTargetCluster: (targetCluster) => {
                set({ targetCluster });
                chatChannel?.postMessage({ type: 'SET_TARGET_CLUSTER', payload: targetCluster });
            },

            setDetached: (isDetached) => set({ isDetached }),

            clearHistory: () => {
                set({ messages: [], messageIndexById: {} });
                chatChannel?.postMessage({ type: 'CLEAR_HISTORY' });
            },
        }),
        {
            name: 'tadpole-sovereign-chat',
            version: 2,
            partialize: (state): PersistedSovereignState => ({
                activeScope: state.activeScope,
                selectedAgentId: state.selectedAgentId,
                targetAgent: state.targetAgent,
                targetCluster: state.targetCluster,
                isDetached: state.isDetached
            }),
            migrate: (persistedState: unknown, version: number) => {
                const state = (persistedState || {}) as Partial<PersistedSovereignState>;
                if (version <= 1) {
                    return {
                        activeScope: state.activeScope ?? 'agent',
                        selectedAgentId: state.selectedAgentId ?? null,
                        targetAgent: state.targetAgent || 'Agent of Nine',
                        targetCluster: state.targetCluster ?? '',
                        isDetached: state.isDetached ?? false
                    };
                }
                return {
                    activeScope: state.activeScope ?? 'agent',
                    selectedAgentId: state.selectedAgentId ?? null,
                    targetAgent: state.targetAgent || 'Agent of Nine',
                    targetCluster: state.targetCluster ?? '',
                    isDetached: state.isDetached ?? false
                };
            }
        }
    )
);

// Listen for sync events
if (chatChannel) {
    chatChannel.onmessage = (event: MessageEvent) => {
        const { type, payload } = event.data as { type: string, payload: unknown };
        const state = useSovereignStore.getState();

        switch (type) {
            case 'ADD_MESSAGE':
                {
                    const msg = payload as ChatMessage;
                    if (state.messageIndexById[msg.id] === undefined) {
                        useSovereignStore.setState({
                            messages: [...state.messages, msg],
                            messageIndexById: {
                                ...state.messageIndexById,
                                [msg.id]: state.messages.length
                            }
                        });
                    }
                }
                break;
            case 'UPDATE_MESSAGE':
                {
                    const msg = payload as ChatMessage;
                    const idx = state.messageIndexById[msg.id];
                    if (idx !== undefined) {
                        const nextMessages = [...state.messages];
                        nextMessages[idx] = msg;
                        useSovereignStore.setState({ messages: nextMessages });
                    }
                }
                break;
            case 'SET_SCOPE':
                useSovereignStore.setState({ activeScope: payload as SovereignScope });
                break;
            case 'SET_AGENT':
                useSovereignStore.setState({ selectedAgentId: payload as string | null });
                break;
            case 'SET_TARGET_AGENT':
                useSovereignStore.setState({ targetAgent: payload as string });
                break;
            case 'SET_TARGET_CLUSTER':
                useSovereignStore.setState({ targetCluster: payload as string });
                break;
            case 'CLEAR_HISTORY':
                useSovereignStore.setState({ messages: [], messageIndexById: {} });
                break;
        }
    };
}
