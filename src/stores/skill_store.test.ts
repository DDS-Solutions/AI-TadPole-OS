/**
 * @docs ARCHITECTURE:State
 * 
 * ### AI Assist Note
 * **Test Suite**: Validates the Agent skill registry and capability mapping. 
 */

/**
 * @file skill_store.test.ts
 * @description Suite for the Swarm Scripting, Workflow, and MCP Tool registry.
 * @module Stores/SkillStore
 * @testedBehavior
 * - Skill Discovery: Fetching and sorting of local scripts, YAML workflows, and lifecycle hooks.
 * - Tool Orchestration: Pulse telemetry handling and Moving Average (MA) latency calculations.
 * - MCP Integration: Verification of external tool manifest synchronization.
 * @aiContext
 * - Refactored for 100% snake_case architectural parity.
 * - Mocks tadpole_os_service for unified skill retrieval.
 * - Uses fake timers to validate pulse UI timeout behavior.
 * - Verified 154 tests sweep continuation.
 * - AI awakening notes confirmed.
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
            mcp_tools: [],
            manifests: [],
            is_loading: false,
            error: null,
        });
        vi.clearAllMocks();
        vi.useFakeTimers(); // For handle_pulse
    });

    afterEach(() => {
        vi.useRealTimers();
    });

    describe('fetch_skills', () => {
        it('fetches and sorts skills and workflows successfully', async () => {
            const mock_data = {
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

            vi.mocked(tadpole_os_service.get_unified_skills).mockResolvedValue(mock_data as any);

            const store = use_skill_store.getState();
            
            const fetch_promise = store.fetch_skills();
            expect(use_skill_store.getState().is_loading).toBe(true);
            await fetch_promise;

            const state = use_skill_store.getState();
            expect(state.is_loading).toBe(false);
            expect(state.error).toBeNull();
            
            // Should be sorted alphabetically
            expect(state.scripts[0].name).toBe('AppBuilder');
            expect(state.workflows[0].name).toBe('A_Workflow');
            expect(tadpole_os_service.get_unified_skills).toHaveBeenCalled();
        });

        it('handles failure during fetch_skills', async () => {
            vi.mocked(tadpole_os_service.get_unified_skills).mockRejectedValue(new Error('Network error'));

            const store = use_skill_store.getState();
            await store.fetch_skills();

            const state = use_skill_store.getState();
            expect(state.is_loading).toBe(false);
            expect(state.error).toBe('Network error');
            expect(state.scripts.length).toBe(0);
        });
    });

    describe('fetch_mcp_tools', () => {
        it('fetches mcp tools successfully', async () => {
            const mock_tools = [{ name: 'test_tool', description: 'desc', input_schema: {}, source: 'local', stats: { invocations: 0, success_count: 0, failure_count: 0, avg_latency_ms: 0 }, category: 'user' as const }];
            vi.mocked(tadpole_os_service.get_mcp_tools).mockResolvedValue(mock_tools as any);

            const store = use_skill_store.getState();
            await store.fetch_mcp_tools();

            const state = use_skill_store.getState();
            expect(state.is_loading).toBe(false);
            expect(state.mcp_tools).toEqual(mock_tools);
            expect(tadpole_os_service.get_mcp_tools).toHaveBeenCalled();
        });

        it('handles failure during fetch_mcp_tools', async () => {
            vi.mocked(tadpole_os_service.get_mcp_tools).mockRejectedValue(new Error('MCP failure'));

            const store = use_skill_store.getState();
            await store.fetch_mcp_tools();

            const state = use_skill_store.getState();
            expect(state.is_loading).toBe(false);
            expect(state.error).toBe('MCP failure');
        });
    });

    describe('CUD Operations', () => {
        it('saves a skill script and refetches skills', async () => {
            vi.mocked(tadpole_os_service.save_skill_script).mockResolvedValue({} as any);
            vi.mocked(tadpole_os_service.get_unified_skills).mockResolvedValue({ scripts: [], workflows: [], hooks: [], manifests: [] } as any);

            const store = use_skill_store.getState();
            const mock_skill = { name: 'NewSkill', description: '', execution_command: '', schema: {}, category: 'user' as const };
            
            await store.save_skill_script(mock_skill);

            expect(tadpole_os_service.save_skill_script).toHaveBeenCalledWith('NewSkill', mock_skill);
            expect(tadpole_os_service.get_unified_skills).toHaveBeenCalled();
        });

        it('deletes a skill script and refetches', async () => {
            vi.mocked(tadpole_os_service.delete_skill_script).mockResolvedValue({} as any);
            vi.mocked(tadpole_os_service.get_unified_skills).mockResolvedValue({ scripts: [], workflows: [], hooks: [], manifests: [] } as any);

            const store = use_skill_store.getState();
            await store.delete_skill_script('OldSkill');

            expect(tadpole_os_service.delete_skill_script).toHaveBeenCalledWith('OldSkill');
            expect(tadpole_os_service.get_unified_skills).toHaveBeenCalled();
        });

        it('saves a workflow and refetches', async () => {
            vi.mocked(tadpole_os_service.save_workflow).mockResolvedValue({} as any);
            vi.mocked(tadpole_os_service.get_unified_skills).mockResolvedValue({ scripts: [], workflows: [], hooks: [], manifests: [] } as any);

            const store = use_skill_store.getState();
            const mock_wf = { name: 'NewWF', content: '...', category: 'user' as const };

            await store.save_workflow(mock_wf);

            expect(tadpole_os_service.save_workflow).toHaveBeenCalledWith('NewWF', mock_wf);
            expect(tadpole_os_service.get_unified_skills).toHaveBeenCalled();
        });

        it('deletes a workflow and refetches', async () => {
            vi.mocked(tadpole_os_service.delete_workflow).mockResolvedValue({} as any);
            vi.mocked(tadpole_os_service.get_unified_skills).mockResolvedValue({ scripts: [], workflows: [], hooks: [], manifests: [] } as any);

            const store = use_skill_store.getState();
            await store.delete_workflow('OldWF');

            expect(tadpole_os_service.delete_workflow).toHaveBeenCalledWith('OldWF');
            expect(tadpole_os_service.get_unified_skills).toHaveBeenCalled();
        });

        it('saves a hook and refetches', async () => {
            vi.mocked(tadpole_os_service.save_hook).mockResolvedValue({} as any);
            vi.mocked(tadpole_os_service.get_unified_skills).mockResolvedValue({ scripts: [], workflows: [], hooks: [], manifests: [] } as any);

            const store = use_skill_store.getState();
            const mock_hook = { 
                name: 'NewHook', 
                description: 'A test hook',
                hook_type: 'lifecycle',
                content: 'println("hello")',
                active: true,
                category: 'user' as const
            };

            await store.save_hook(mock_hook);

            expect(tadpole_os_service.save_hook).toHaveBeenCalledWith('NewHook', mock_hook);
            expect(tadpole_os_service.get_unified_skills).toHaveBeenCalled();
        });

        it('deletes a hook and refetches', async () => {
            vi.mocked(tadpole_os_service.delete_hook).mockResolvedValue({} as any);
            vi.mocked(tadpole_os_service.get_unified_skills).mockResolvedValue({ scripts: [], workflows: [], hooks: [], manifests: [] } as any);

            const store = use_skill_store.getState();
            await store.delete_hook('OldHook');

            expect(tadpole_os_service.delete_hook).toHaveBeenCalledWith('OldHook');
            expect(tadpole_os_service.get_unified_skills).toHaveBeenCalled();
        });
    });

    describe('handle_pulse', () => {
        it('updates stats for a specific tool and toggles is_pulsing', () => {
            const initial_tools = [
                { 
                    name: 'target_tool', 
                    description: '', 
                    input_schema: {}, 
                    source: '', 
                    stats: { invocations: 1, success_count: 1, failure_count: 0, avg_latency_ms: 100 },
                    is_pulsing: false,
                    category: 'user' as const
                },
                { 
                    name: 'other_tool', 
                    description: '', 
                    input_schema: {}, 
                    source: '', 
                    stats: { invocations: 0, success_count: 0, failure_count: 0, avg_latency_ms: 0 },
                    is_pulsing: false,
                    category: 'user' as const
                }
            ];

            use_skill_store.setState({ mcp_tools: initial_tools });
            const store = use_skill_store.getState();

            // Fire pulse
            store.handle_pulse('target_tool', 'success', 200);

            let state = use_skill_store.getState();
            let target = state.mcp_tools.find(t => t.name === 'target_tool')!;
            
            expect(target.stats.invocations).toBe(2);
            expect(target.stats.success_count).toBe(2);
            expect(target.stats.avg_latency_ms).toBe((100 + 200) / 2); // Moving average logic
            expect(target.is_pulsing).toBe(true);

            const other = state.mcp_tools.find(t => t.name === 'other_tool')!;
            expect(other.stats.invocations).toBe(0); // Untouched

            // Fast forward timer
            vi.advanceTimersByTime(1000);

            // Recheck state to ensure pulse turned off
            state = use_skill_store.getState();
            target = state.mcp_tools.find(t => t.name === 'target_tool')!;
            expect(target.is_pulsing).toBe(false);
        });

        it('handles failure pulses correctly', () => {
            const initial_tools = [
                { 
                    name: 'failing_tool', 
                    description: '', 
                    input_schema: {}, 
                    source: '', 
                    stats: { invocations: 0, success_count: 0, failure_count: 0, avg_latency_ms: 0 },
                    is_pulsing: false,
                    category: 'user' as const
                }
            ];

            use_skill_store.setState({ mcp_tools: initial_tools });
            const store = use_skill_store.getState();

            // Fire error pulse
            store.handle_pulse('failing_tool', 'error', 500);

            const state = use_skill_store.getState();
            const target = state.mcp_tools[0];
            
            expect(target.stats.invocations).toBe(1);
            expect(target.stats.success_count).toBe(0);
            expect(target.stats.failure_count).toBe(1);
            expect(target.stats.avg_latency_ms).toBe(500); // 0 start defaults to exactly latency
            expect(target.is_pulsing).toBe(true);
        });
    });
});

