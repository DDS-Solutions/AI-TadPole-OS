/**
 * @docs ARCHITECTURE:Testing
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend React Hooks / useSkillsManager.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` React hook lifecycle adheres to Rules of Hooks without conditional execution branches.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useSkillsManager } from './useSkillsManager';
import { use_skill_store } from '../stores/skill_store';

describe('useSkillsManager', () => {
    beforeEach(() => {
        vi.restoreAllMocks();
        use_skill_store.setState({
            scripts: [
                { name: 'AuditSkill', description: 'System audit', category: 'user', content: 'test' } as any
            ],
            workflows: [
                { name: 'DeployWorkflow', category: 'user', content: 'steps' } as any
            ],
            hooks: [
                { name: 'PreValidation', description: 'Guard hook', hook_type: 'pre_validation', content: 'check', active: true, category: 'user' }
            ],
            mcp_tools: [
                { name: 'WebScout', description: 'Browser scout', tools: [] } as any
            ]
        });
    });

    it('initializes with default tab and search query', () => {
        const { result } = renderHook(() => useSkillsManager());
        expect(result.current.active_tab).toBe('all');
        expect(result.current.search_query).toBe('');
        expect(result.current.scripts.length).toBe(1);
    });

    it('updates active tab and filters memoized lists on search', () => {
        const { result } = renderHook(() => useSkillsManager());

        act(() => {
            result.current.set_active_tab('scripts');
            result.current.set_search_query('audit');
        });

        expect(result.current.active_tab).toBe('scripts');
        expect(result.current.scripts.length).toBe(1);

        act(() => {
            result.current.set_search_query('nonexistent');
        });

        expect(result.current.scripts.length).toBe(0);
    });

    it('manages hook creation and edit transitions', () => {
        const { result } = renderHook(() => useSkillsManager());

        act(() => {
            result.current.handle_create_hook();
        });
        expect(result.current.hook_form.name).toBe('');

        act(() => {
            result.current.handle_edit_hook({
                name: 'CustomHook',
                description: 'Custom desc',
                hook_type: 'pre_validation',
                content: 'code',
                active: true,
                category: 'user'
            });
        });

        expect(result.current.hook_form.name).toBe('CustomHook');
        expect(result.current.editing_hook?.name).toBe('CustomHook');
    });

    it('manages assignment modal state', () => {
        const { result } = renderHook(() => useSkillsManager());

        act(() => {
            result.current.handle_assign('skill', 'AuditSkill');
        });

        expect(result.current.assignment_modal_open).toBe(true);
        expect(result.current.assigning_item).toEqual({ type: 'skill', name: 'AuditSkill' });
    });
});
