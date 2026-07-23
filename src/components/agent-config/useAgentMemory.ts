/**
 * @docs ARCHITECTURE:Interface
 *
 * ### AI Assist Note
 * **useAgentMemory**: Encapsulates vector-store CRUD for a single agent's memory.
 * Extracted from `AgentConfigPanel` so the panel is pure orchestration/layout,
 * with no embedded async data logic.
 *
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Memory load failure (API 500) or agentId undefined on first render.
 * - **Telemetry Link**: Search `[AgentConfigPanel]` in UI traces.
 */

import { useState, useCallback, useEffect, useRef } from 'react';
import { tadpole_os_service } from '../../services/tadpoleos_service';
import { event_bus } from '../../services/event_bus';
import { i18n } from '../../i18n';

export interface Memory_Entry {
    id: string;
    content?: string;
    text?: string;
}

export interface UseAgentMemoryReturn {
    memories: Memory_Entry[];
    isLoadingMemories: boolean;
    memoryInput: string;
    setMemoryInput: (val: string) => void;
    loadMemories: () => Promise<void>;
    handleSaveMemory: () => Promise<void>;
    handleDeleteMemory: (id: string) => Promise<void>;
}

export function useAgentMemory(agentId: string | undefined, mainTab: string): UseAgentMemoryReturn {
    const [memories, setMemories] = useState<Memory_Entry[]>([]);
    const [isLoadingMemories, setIsLoadingMemories] = useState(false);
    const [memoryInput, setMemoryInput] = useState('');
    const fetchedRef = useRef<string | null>(null);

    const loadMemories = useCallback(async () => {
        if (!agentId) return;
        setIsLoadingMemories(true);
        try {
            const response = await tadpole_os_service.get_agent_memory(agentId) as { entries: Memory_Entry[] };
            setMemories(response.entries || []);
        } catch (error) {
            console.error('[AgentConfigPanel] loadMemories failed:', error);
            event_bus.emit_log({
                text: i18n.t('agent_config.memory_load_failed') || 'Failed to load agent memories.',
                severity: 'error',
                source: 'System'
            });
            setMemories([]);
        } finally {
            setIsLoadingMemories(false);
        }
    }, [agentId]);

    useEffect(() => {
        if (agentId && mainTab === 'memory' && fetchedRef.current !== agentId) {
            fetchedRef.current = agentId;
            void loadMemories().catch(() => {
                fetchedRef.current = null;
            });
        }
    }, [agentId, mainTab, loadMemories]);

    const handleSaveMemory = async () => {
        if (!memoryInput.trim() || !agentId) return;
        try {
            await tadpole_os_service.save_agent_memory(agentId, memoryInput);
            setMemoryInput('');
            event_bus.emit_log({
                text: i18n.t('agent_config.memory_saved') || 'Memory entry recorded successfully.',
                severity: 'success',
                source: 'System'
            });
            try {
                await loadMemories();
            } catch (loadError) {
                console.error('[AgentConfigPanel] loadMemories after save failed:', loadError);
            }
        } catch (error) {
            console.error('[AgentConfigPanel] handleSaveMemory failed:', error);
            event_bus.emit_log({
                text: i18n.t('agent_config.memory_save_failed') || 'Failed to record memory entry.',
                severity: 'error',
                source: 'System'
            });
        }
    };

    const handleDeleteMemory = async (id: string) => {
        if (!agentId) return;
        try {
            await tadpole_os_service.delete_agent_memory(agentId, id);
            event_bus.emit_log({
                text: i18n.t('agent_config.memory_deleted') || 'Memory entry purged successfully.',
                severity: 'info',
                source: 'System'
            });
            try {
                await loadMemories();
            } catch (loadError) {
                console.error('[AgentConfigPanel] loadMemories after delete failed:', loadError);
            }
        } catch (error) {
            console.error('[AgentConfigPanel] handleDeleteMemory failed:', error);
            event_bus.emit_log({
                text: i18n.t('agent_config.memory_delete_failed') || 'Failed to purge memory entry.',
                severity: 'error',
                source: 'System'
            });
        }
    };

    return {
        memories,
        isLoadingMemories,
        memoryInput,
        setMemoryInput,
        loadMemories,
        handleSaveMemory,
        handleDeleteMemory,
    };
}

// Metadata: [useAgentMemory]
