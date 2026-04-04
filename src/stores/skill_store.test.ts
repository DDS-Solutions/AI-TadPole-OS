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
 * - Mocks tadpole_os_service for isolated skill discovery testing.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { use_skill_store } from './skill_store';
import { tadpole_os_service } from '../services/tadpoleos_service';

// Mock tadpole_os_service
vi.mock('../services/tadpoleos_service', () => ({
    tadpole_os_service: {
        get_unified_skills: vi.fn(),
        get_mcp_tools: vi.fn(),
        save_skill_script: vi.fn(),
        delete_skill_script: vi.fn(),
        save_workflow: vi.fn(),
        delete_workflow: vi.fn(),
        save_hook: vi.fn(),
        delete_hook: vi.fn(),
    }
}));

describe('use_skill_store', () => {
    beforeEach(() => {
        // Reset state
        use_skill_store.setState({
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

            vi.mocked(tadpole_os_service.get_unified_skills).mockResolvedValue(mockData as any);

            const store = use_skill_store.getState();
            
            const fetchPromise = store.fetchSkills();
            expect(use_skill_store.getState().isLoading).toBe(true);
            await fetchPromise;

            const state = use_skill_store.getState();
            expect(state.isLoading).toBe(false);
            expect(state.error).toBeNull();
            
            // Should be sorted alphabetically
            expect(state.scripts[0].name).toBe('AppBuilder');
            expect(state.workflows[0].name).toBe('A_Workflow');
            expect(tadpole_os_service.get_unified_skills).toHaveBeenCalled();
        });

        it('handles failure during fetchSkills', async () => {
            vi.mocked(tadpole_os_service.get_unified_skills).mockRejectedValue(new Error('Network error'));

            const store = use_skill_store.getState();
            await store.fetchSkills();

            const state = use_skill_store.getState();
            expect(state.isLoading).toBe(false);
            expect(state.error).toBe('Network error');
            expect(state.scripts.length).toBe(0);
        });
    });

    describe('fetchMcpTools', () => {
        it('fetches mcp tools successfully', async () => {
            const mockTools = [{ name: 'test_tool', description: 'desc', input_schema: {}, source: 'local', stats: { invocations: 0, success_count: 0, failure_count: 0, avg_latency_ms: 0 }, category: 'user' as const }];
            vi.mocked(tadpole_os_service.get_mcp_tools).mockResolvedValue(mockTools as any);

            const store = use_skill_store.getState();
            await store.fetchMcpTools();

            const state = use_skill_store.getState();
            expect(state.isLoading).toBe(false);
            expect(state.mcpTools).toEqual(mockTools);
            expect(tadpole_os_service.get_mcp_tools).toHaveBeenCalled();
        });

        it('handles failure during fetchMcpTools', async () => {
            vi.mocked(tadpole_os_service.get_mcp_tools).mockRejectedValue(new Error('MCP failure'));

            const store = use_skill_store.getState();
            await store.fetchMcpTools();

            const state = use_skill_store.getState();
            expect(state.isLoading).toBe(false);
            expect(state.error).toBe('MCP failure');
        });
    });

    describe('CUD Operations', () => {
        it('saves a skill script and refetches skills', async () => {
            vi.mocked(tadpole_os_service.save_skill_script).mockResolvedValue({} as any);
            vi.mocked(tadpole_os_service.get_unified_skills).mockResolvedValue({ skills: [], workflows: [], hooks: [], manifests: [] } as any);

            const store = use_skill_store.getState();
            const mockSkill = { name: 'NewSkill', description: '', execution_command: '', schema: {}, category: 'user' as const };
            
            await store.save_skill_script(mockSkill);

            expect(tadpole_os_service.save_skill_script).toHaveBeenCalledWith('NewSkill', mockSkill);
            expect(tadpole_os_service.get_unified_skills).toHaveBeenCalled();
        });

        it('deletes a skill script and refetches', async () => {
            vi.mocked(tadpole_os_service.delete_skill_script).mockResolvedValue({} as any);
            vi.mocked(tadpole_os_service.get_unified_skills).mockResolvedValue({ skills: [], workflows: [], hooks: [], manifests: [] } as any);

            const store = use_skill_store.getState();
            await store.delete_skill_script('OldSkill');

            expect(tadpole_os_service.delete_skill_script).toHaveBeenCalledWith('OldSkill');
            expect(tadpole_os_service.get_unified_skills).toHaveBeenCalled();
        });

        it('saves a workflow and refetches', async () => {
            vi.mocked(tadpole_os_service.save_workflow).mockResolvedValue({} as any);
            vi.mocked(tadpole_os_service.get_unified_skills).mockResolvedValue({ skills: [], workflows: [], hooks: [], manifests: [] } as any);

            const store = use_skill_store.getState();
            const mockWf = { name: 'NewWF', content: '...', category: 'user' as const };

            await store.save_workflow(mockWf);

            expect(tadpole_os_service.save_workflow).toHaveBeenCalledWith('NewWF', mockWf);
            expect(tadpole_os_service.get_unified_skills).toHaveBeenCalled();
        });

        it('deletes a workflow and refetches', async () => {
            vi.mocked(tadpole_os_service.delete_workflow).mockResolvedValue({} as any);
            vi.mocked(tadpole_os_service.get_unified_skills).mockResolvedValue({ skills: [], workflows: [], hooks: [], manifests: [] } as any);

            const store = use_skill_store.getState();
            await store.delete_workflow('OldWF');

            expect(tadpole_os_service.delete_workflow).toHaveBeenCalledWith('OldWF');
            expect(tadpole_os_service.get_unified_skills).toHaveBeenCalled();
        });

        it('saves a hook and refetches', async () => {
            vi.mocked(tadpole_os_service.save_hook).mockResolvedValue({} as any);
            vi.mocked(tadpole_os_service.get_unified_skills).mockResolvedValue({ skills: [], workflows: [], hooks: [], manifests: [] } as any);

            const store = use_skill_store.getState();
            const mockHook = { 
                name: 'NewHook', 
                description: 'A test hook',
                hook_type: 'lifecycle',
                content: 'println("hello")',
                active: true,
                category: 'user' as const
            };

            await store.save_hook(mockHook);

            expect(tadpole_os_service.save_hook).toHaveBeenCalledWith('NewHook', mockHook);
            expect(tadpole_os_service.get_unified_skills).toHaveBeenCalled();
        });

        it('deletes a hook and refetches', async () => {
            vi.mocked(tadpole_os_service.delete_hook).mockResolvedValue({} as any);
            vi.mocked(tadpole_os_service.get_unified_skills).mockResolvedValue({ skills: [], workflows: [], hooks: [], manifests: [] } as any);

            const store = use_skill_store.getState();
            await store.delete_hook('OldHook');

            expect(tadpole_os_service.delete_hook).toHaveBeenCalledWith('OldHook');
            expect(tadpole_os_service.get_unified_skills).toHaveBeenCalled();
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

            use_skill_store.setState({ mcpTools: initialTools });
            const store = use_skill_store.getState();

            // Fire pulse
            store.handlePulse('target_tool', 'success', 200);

            let state = use_skill_store.getState();
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
            state = use_skill_store.getState();
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

            use_skill_store.setState({ mcpTools: initialTools });
            const store = use_skill_store.getState();

            // Fire error pulse
            store.handlePulse('failing_tool', 'error', 500);

            const state = use_skill_store.getState();
            const target = state.mcpTools[0];
            
            expect(target.stats.invocations).toBe(1);
            expect(target.stats.success_count).toBe(0);
            expect(target.stats.failure_count).toBe(1);
            expect(target.stats.avg_latency_ms).toBe(500); // 0 start defaults to exactly latency
            expect(target.isPulsing).toBe(true);
        });
    });
});

