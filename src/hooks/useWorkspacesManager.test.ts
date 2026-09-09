/**
 * @docs ARCHITECTURE:Testing
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend React Hooks / useWorkspacesManager.test
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
import { renderHook } from '@testing-library/react';
import { useWorkspacesManager } from './useWorkspacesManager';
import { use_workspace_store } from '../stores/workspace_store';

describe('useWorkspacesManager', () => {
    beforeEach(() => {
        vi.restoreAllMocks();
        use_workspace_store.setState({
            clusters: [
                { id: 'c1', name: 'Cluster One' } as any
            ],
            sync_status: { status: 'idle', pending_changes: 0 } as any
        });
    });

    it('initializes and formats byte sizes correctly', () => {
        const { result } = renderHook(() => useWorkspacesManager());
        expect(result.current.clusters.length).toBe(1);
        expect(result.current.format_bytes(0)).toBe('0 B');
        expect(result.current.format_bytes(1024)).toBe('1 KB');
        expect(result.current.format_bytes(1048576)).toBe('1 MB');
    });
});
