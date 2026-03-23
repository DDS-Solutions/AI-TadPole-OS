/**
 * @file skillStore.test.ts
 * @description Suite for Skills, Workflows, and MCP Tool discovery/management.
 * @module Stores/SkillStore
 * @testedBehavior
 * - Discovery: Fetching and alphabetical sorting of platform skills.
 * - MCP Health: Moving average latency tracking via 'pulse' events.
 * - Lifecycle: Standard CUD operations for skills and complex workflows.
 * @aiContext
 * - Uses vitest fake timers to validate 'isPulsing' UI state transitions.
 * - Mocks TadpoleOSService for isolated skill discovery testing.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { useSkillStore } from './skillStore';
import { TadpoleOSService } from '../services/tadpoleosService';

// Mock TadpoleOSService
vi.mock('../services/tadpoleosService', () => ({
    TadpoleOSService: {
        getUnifiedSkills: vi.fn(),
        getMcpTools: vi.fn(),
        saveSkillScript: vi.fn(),
        deleteSkillScript: vi.fn(),
        saveWorkflow: vi.fn(),
        deleteWorkflow: vi.fn(),
        saveHook: vi.fn(),
        deleteHook: vi.fn(),
    }
}));

describe('useSkillStore', () => {
    beforeEach(() => {
        // Reset state
        useSkillStore.setState({
            scripts: [],
            workflows: [],
            hooks: [],
            mcpTools: [],
            manifests: [],
            isLoading: false,
            error: null,
        });
        vi.clearAllMocks();
        vi.useFakeTimers(); // For handlePulse
    });

    afterEach(() => {
        vi.useRealTimers();
    });

    describe('fetchSkills', () => {
        it('fetches and sorts skills and workflows successfully', async () => {
            const mockData = {
                scripts: [
                    { name: 'BrowserSearch', description: 'desc', execution_command: 'cmd', schema: {}, category: 'user' },
                    { name: 'AppBuilder', description: 'desc', execution_command: 'cmd', schema: {}, category: 'user' }
                ],
                workflows: [
                    { name: 'Z_Workflow', content: '...', category: 'user' },
                    { name: 'A_Workflow', content: '...', category: 'user' }
                ],
                hooks: [
                    { 
                        name: 'on_init', 
                        description: 'desc', 
                        hook_type: 'lifecycle', 
                        content: '', 
                        active: true, 
                        category: 'user' 
                    }
                ],
                manifests: []
            };

            vi.mocked(TadpoleOSService.getUnifiedSkills).mockResolvedValue(mockData as any);

            const store = useSkillStore.getState();
            
            const fetchPromise = store.fetchSkills();
            expect(useSkillStore.getState().isLoading).toBe(true);
            await fetchPromise;

            const state = useSkillStore.getState();
            expect(state.isLoading).toBe(false);
            expect(state.error).toBeNull();
            
            // Should be sorted alphabetically
            expect(state.scripts[0].name).toBe('AppBuilder');
            expect(state.workflows[0].name).toBe('A_Workflow');
            expect(TadpoleOSService.getUnifiedSkills).toHaveBeenCalled();
        });

        it('handles failure during fetchSkills', async () => {
            vi.mocked(TadpoleOSService.getUnifiedSkills).mockRejectedValue(new Error('Network error'));

            const store = useSkillStore.getState();
            await store.fetchSkills();

            const state = useSkillStore.getState();
            expect(state.isLoading).toBe(false);
            expect(state.error).toBe('Network error');
            expect(state.scripts.length).toBe(0);
        });
    });

    describe('fetchMcpTools', () => {
        it('fetches mcp tools successfully', async () => {
            const mockTools = [{ name: 'test_tool', description: 'desc', input_schema: {}, source: 'local', stats: { invocations: 0, success_count: 0, failure_count: 0, avg_latency_ms: 0 }, category: 'user' as const }];
            vi.mocked(TadpoleOSService.getMcpTools).mockResolvedValue(mockTools as any);

            const store = useSkillStore.getState();
            await store.fetchMcpTools();

            const state = useSkillStore.getState();
            expect(state.isLoading).toBe(false);
            expect(state.mcpTools).toEqual(mockTools);
            expect(TadpoleOSService.getMcpTools).toHaveBeenCalled();
        });

        it('handles failure during fetchMcpTools', async () => {
            vi.mocked(TadpoleOSService.getMcpTools).mockRejectedValue(new Error('MCP failure'));

            const store = useSkillStore.getState();
            await store.fetchMcpTools();

            const state = useSkillStore.getState();
            expect(state.isLoading).toBe(false);
            expect(state.error).toBe('MCP failure');
        });
    });

    describe('CUD Operations', () => {
        it('saves a skill script and refetches skills', async () => {
            vi.mocked(TadpoleOSService.saveSkillScript).mockResolvedValue({} as any);
            vi.mocked(TadpoleOSService.getUnifiedSkills).mockResolvedValue({ skills: [], workflows: [], hooks: [], manifests: [] } as any);

            const store = useSkillStore.getState();
            const mockSkill = { name: 'NewSkill', description: '', execution_command: '', schema: {}, category: 'user' as const };
            
            await store.saveSkillScript(mockSkill);

            expect(TadpoleOSService.saveSkillScript).toHaveBeenCalledWith('NewSkill', mockSkill);
            expect(TadpoleOSService.getUnifiedSkills).toHaveBeenCalled();
        });

        it('deletes a skill script and refetches', async () => {
            vi.mocked(TadpoleOSService.deleteSkillScript).mockResolvedValue({} as any);
            vi.mocked(TadpoleOSService.getUnifiedSkills).mockResolvedValue({ skills: [], workflows: [], hooks: [], manifests: [] } as any);

            const store = useSkillStore.getState();
            await store.deleteSkillScript('OldSkill');

            expect(TadpoleOSService.deleteSkillScript).toHaveBeenCalledWith('OldSkill');
            expect(TadpoleOSService.getUnifiedSkills).toHaveBeenCalled();
        });

        it('saves a workflow and refetches', async () => {
            vi.mocked(TadpoleOSService.saveWorkflow).mockResolvedValue({} as any);
            vi.mocked(TadpoleOSService.getUnifiedSkills).mockResolvedValue({ skills: [], workflows: [], hooks: [], manifests: [] } as any);

            const store = useSkillStore.getState();
            const mockWf = { name: 'NewWF', content: '...', category: 'user' as const };

            await store.saveWorkflow(mockWf);

            expect(TadpoleOSService.saveWorkflow).toHaveBeenCalledWith('NewWF', mockWf);
            expect(TadpoleOSService.getUnifiedSkills).toHaveBeenCalled();
        });

        it('deletes a workflow and refetches', async () => {
            vi.mocked(TadpoleOSService.deleteWorkflow).mockResolvedValue({} as any);
            vi.mocked(TadpoleOSService.getUnifiedSkills).mockResolvedValue({ skills: [], workflows: [], hooks: [], manifests: [] } as any);

            const store = useSkillStore.getState();
            await store.deleteWorkflow('OldWF');

            expect(TadpoleOSService.deleteWorkflow).toHaveBeenCalledWith('OldWF');
            expect(TadpoleOSService.getUnifiedSkills).toHaveBeenCalled();
        });

        it('saves a hook and refetches', async () => {
            vi.mocked(TadpoleOSService.saveHook).mockResolvedValue({} as any);
            vi.mocked(TadpoleOSService.getUnifiedSkills).mockResolvedValue({ skills: [], workflows: [], hooks: [], manifests: [] } as any);

            const store = useSkillStore.getState();
            const mockHook = { 
                name: 'NewHook', 
                description: 'A test hook',
                hook_type: 'lifecycle',
                content: 'println("hello")',
                active: true,
                category: 'user' as const
            };

            await store.saveHook(mockHook);

            expect(TadpoleOSService.saveHook).toHaveBeenCalledWith('NewHook', mockHook);
            expect(TadpoleOSService.getUnifiedSkills).toHaveBeenCalled();
        });

        it('deletes a hook and refetches', async () => {
            vi.mocked(TadpoleOSService.deleteHook).mockResolvedValue({} as any);
            vi.mocked(TadpoleOSService.getUnifiedSkills).mockResolvedValue({ skills: [], workflows: [], hooks: [], manifests: [] } as any);

            const store = useSkillStore.getState();
            await store.deleteHook('OldHook');

            expect(TadpoleOSService.deleteHook).toHaveBeenCalledWith('OldHook');
            expect(TadpoleOSService.getUnifiedSkills).toHaveBeenCalled();
        });
    });

    describe('handlePulse', () => {
        it('updates stats for a specific tool and toggles isPulsing', () => {
            const initialTools = [
                { 
                    name: 'target_tool', 
                    description: '', 
                    input_schema: {}, 
                    source: '', 
                    stats: { invocations: 1, success_count: 1, failure_count: 0, avg_latency_ms: 100 },
                    isPulsing: false,
                    category: 'user' as const
                },
                { 
                    name: 'other_tool', 
                    description: '', 
                    input_schema: {}, 
                    source: '', 
                    stats: { invocations: 0, success_count: 0, failure_count: 0, avg_latency_ms: 0 },
                    isPulsing: false,
                    category: 'user' as const
                }
            ];

            useSkillStore.setState({ mcpTools: initialTools });
            const store = useSkillStore.getState();

            // Fire pulse
            store.handlePulse('target_tool', 'success', 200);

            let state = useSkillStore.getState();
            let target = state.mcpTools.find(t => t.name === 'target_tool')!;
            
            expect(target.stats.invocations).toBe(2);
            expect(target.stats.success_count).toBe(2);
            expect(target.stats.avg_latency_ms).toBe((100 + 200) / 2); // Moving average logic
            expect(target.isPulsing).toBe(true);

            const other = state.mcpTools.find(t => t.name === 'other_tool')!;
            expect(other.stats.invocations).toBe(0); // Untouched

            // Fast forward timer
            vi.advanceTimersByTime(1000);

            // Recheck state to ensure pulse turned off
            state = useSkillStore.getState();
            target = state.mcpTools.find(t => t.name === 'target_tool')!;
            expect(target.isPulsing).toBe(false);
        });

        it('handles failure pulses correctly', () => {
            const initialTools = [
                { 
                    name: 'failing_tool', 
                    description: '', 
                    input_schema: {}, 
                    source: '', 
                    stats: { invocations: 0, success_count: 0, failure_count: 0, avg_latency_ms: 0 },
                    isPulsing: false,
                    category: 'user' as const
                }
            ];

            useSkillStore.setState({ mcpTools: initialTools });
            const store = useSkillStore.getState();

            // Fire error pulse
            store.handlePulse('failing_tool', 'error', 500);

            const state = useSkillStore.getState();
            const target = state.mcpTools[0];
            
            expect(target.stats.invocations).toBe(1);
            expect(target.stats.success_count).toBe(0);
            expect(target.stats.failure_count).toBe(1);
            expect(target.stats.avg_latency_ms).toBe(500); // 0 start defaults to exactly latency
            expect(target.isPulsing).toBe(true);
        });
    });
});
