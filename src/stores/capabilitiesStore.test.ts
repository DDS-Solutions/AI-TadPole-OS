/**
 * @file capabilitiesStore.test.ts
 * @description Suite for Skills, Workflows, and MCP Tool discovery/management.
 * @module Stores/CapabilitiesStore
 * @testedBehavior
 * - Discovery: Fetching and alphabetical sorting of platform capabilities.
 * - MCP Health: Moving average latency tracking via 'pulse' events.
 * - Lifecycle: Standard CUD operations for skills and complex workflows.
 * @aiContext
 * - Uses vitest fake timers to validate 'isPulsing' UI state transitions.
 * - Mocks TadpoleOSService for isolated capability discovery testing.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { useCapabilitiesStore } from './capabilitiesStore';
import { TadpoleOSService } from '../services/tadpoleosService';

// Mock TadpoleOSService
vi.mock('../services/tadpoleosService', () => ({
    TadpoleOSService: {
        getCapabilities: vi.fn(),
        getMcpTools: vi.fn(),
        saveSkill: vi.fn(),
        deleteSkill: vi.fn(),
        saveWorkflow: vi.fn(),
        deleteWorkflow: vi.fn(),
    }
}));

describe('useCapabilitiesStore', () => {
    beforeEach(() => {
        // Reset state
        useCapabilitiesStore.setState({
            skills: [],
            workflows: [],
            mcpTools: [],
            isLoading: false,
            error: null,
        });
        vi.clearAllMocks();
        vi.useFakeTimers(); // For handlePulse
    });

    afterEach(() => {
        vi.useRealTimers();
    });

    describe('fetchCapabilities', () => {
        it('fetches and sorts skills and workflows successfully', async () => {
            const mockData = {
                skills: [
                    { name: 'BrowserSearch', description: 'desc', execution_command: 'cmd', schema: {} },
                    { name: 'AppBuilder', description: 'desc', execution_command: 'cmd', schema: {} }
                ],
                workflows: [
                    { name: 'Z_Workflow', content: '...' },
                    { name: 'A_Workflow', content: '...' }
                ]
            };

            vi.mocked(TadpoleOSService.getCapabilities).mockResolvedValue(mockData as any);

            const store = useCapabilitiesStore.getState();
            
            const fetchPromise = store.fetchCapabilities();
            expect(useCapabilitiesStore.getState().isLoading).toBe(true);
            await fetchPromise;

            const state = useCapabilitiesStore.getState();
            expect(state.isLoading).toBe(false);
            expect(state.error).toBeNull();
            
            // Should be sorted alphabetically
            expect(state.skills[0].name).toBe('AppBuilder');
            expect(state.workflows[0].name).toBe('A_Workflow');
            expect(TadpoleOSService.getCapabilities).toHaveBeenCalled();
        });

        it('handles failure during fetchCapabilities', async () => {
            vi.mocked(TadpoleOSService.getCapabilities).mockRejectedValue(new Error('Network error'));

            const store = useCapabilitiesStore.getState();
            await store.fetchCapabilities();

            const state = useCapabilitiesStore.getState();
            expect(state.isLoading).toBe(false);
            expect(state.error).toBe('Network error');
            expect(state.skills.length).toBe(0);
        });
    });

    describe('fetchMcpTools', () => {
        it('fetches mcp tools successfully', async () => {
            const mockTools = [{ name: 'test_tool', description: 'desc', input_schema: {}, source: 'local', stats: { invocations: 0, success_count: 0, failure_count: 0, avg_latency_ms: 0 } }];
            vi.mocked(TadpoleOSService.getMcpTools).mockResolvedValue(mockTools as any);

            const store = useCapabilitiesStore.getState();
            await store.fetchMcpTools();

            const state = useCapabilitiesStore.getState();
            expect(state.isLoading).toBe(false);
            expect(state.mcpTools).toEqual(mockTools);
            expect(TadpoleOSService.getMcpTools).toHaveBeenCalled();
        });

        it('handles failure during fetchMcpTools', async () => {
            vi.mocked(TadpoleOSService.getMcpTools).mockRejectedValue(new Error('MCP failure'));

            const store = useCapabilitiesStore.getState();
            await store.fetchMcpTools();

            const state = useCapabilitiesStore.getState();
            expect(state.isLoading).toBe(false);
            expect(state.error).toBe('MCP failure');
        });
    });

    describe('CUD Operations', () => {
        it('saves a skill and refetches capabilities', async () => {
            vi.mocked(TadpoleOSService.saveSkill).mockResolvedValue({} as any);
            vi.mocked(TadpoleOSService.getCapabilities).mockResolvedValue({ skills: [], workflows: [] } as any);

            const store = useCapabilitiesStore.getState();
            const mockSkill = { name: 'NewSkill', description: '', execution_command: '', schema: {} };
            
            await store.saveSkill(mockSkill);

            expect(TadpoleOSService.saveSkill).toHaveBeenCalledWith('NewSkill', mockSkill);
            expect(TadpoleOSService.getCapabilities).toHaveBeenCalled();
        });

        it('deletes a skill and refetches', async () => {
            vi.mocked(TadpoleOSService.deleteSkill).mockResolvedValue({} as any);
            vi.mocked(TadpoleOSService.getCapabilities).mockResolvedValue({ skills: [], workflows: [] } as any);

            const store = useCapabilitiesStore.getState();
            await store.deleteSkill('OldSkill');

            expect(TadpoleOSService.deleteSkill).toHaveBeenCalledWith('OldSkill');
            expect(TadpoleOSService.getCapabilities).toHaveBeenCalled();
        });

        it('saves a workflow and refetches', async () => {
            vi.mocked(TadpoleOSService.saveWorkflow).mockResolvedValue({} as any);
            vi.mocked(TadpoleOSService.getCapabilities).mockResolvedValue({ skills: [], workflows: [] } as any);

            const store = useCapabilitiesStore.getState();
            const mockWf = { name: 'NewWF', content: '...' };

            await store.saveWorkflow(mockWf);

            expect(TadpoleOSService.saveWorkflow).toHaveBeenCalledWith('NewWF', mockWf);
            expect(TadpoleOSService.getCapabilities).toHaveBeenCalled();
        });

        it('deletes a workflow and refetches', async () => {
            vi.mocked(TadpoleOSService.deleteWorkflow).mockResolvedValue({} as any);
            vi.mocked(TadpoleOSService.getCapabilities).mockResolvedValue({ skills: [], workflows: [] } as any);

            const store = useCapabilitiesStore.getState();
            await store.deleteWorkflow('OldWF');

            expect(TadpoleOSService.deleteWorkflow).toHaveBeenCalledWith('OldWF');
            expect(TadpoleOSService.getCapabilities).toHaveBeenCalled();
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
                    isPulsing: false
                },
                { 
                    name: 'other_tool', 
                    description: '', 
                    input_schema: {}, 
                    source: '', 
                    stats: { invocations: 0, success_count: 0, failure_count: 0, avg_latency_ms: 0 },
                    isPulsing: false
                }
            ];

            useCapabilitiesStore.setState({ mcpTools: initialTools });
            const store = useCapabilitiesStore.getState();

            // Fire pulse
            store.handlePulse('target_tool', 'success', 200);

            let state = useCapabilitiesStore.getState();
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
            state = useCapabilitiesStore.getState();
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
                    isPulsing: false
                }
            ];

            useCapabilitiesStore.setState({ mcpTools: initialTools });
            const store = useCapabilitiesStore.getState();

            // Fire error pulse
            store.handlePulse('failing_tool', 'error', 500);

            const state = useCapabilitiesStore.getState();
            const target = state.mcpTools[0];
            
            expect(target.stats.invocations).toBe(1);
            expect(target.stats.success_count).toBe(0);
            expect(target.stats.failure_count).toBe(1);
            expect(target.stats.avg_latency_ms).toBe(500); // 0 start defaults to exactly latency
            expect(target.isPulsing).toBe(true);
        });
    });
});
