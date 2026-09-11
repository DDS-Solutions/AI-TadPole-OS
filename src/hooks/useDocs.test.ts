/**
 * @docs ARCHITECTURE:Testing
 *
 * ### AI Context Alignment
 * - **Subsystem**: Frontend React Hooks / useDocs.test
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
import { renderHook, act, waitFor } from '@testing-library/react';
import { useDocs } from './useDocs';
import { system_api_service } from '../services/system_api_service';

vi.mock('../services/system_api_service', () => ({
    system_api_service: {
        docs: {
            get_knowledge_docs: vi.fn(),
            get_knowledge_doc: vi.fn(),
            get_operations_manual: vi.fn()
        }
    }
}));

describe('useDocs', () => {
    beforeEach(() => {
        vi.restoreAllMocks();
        vi.mocked(system_api_service.docs.get_knowledge_docs).mockResolvedValue([
            { category: 'Architecture', name: 'architecture_core', title: 'Architecture Core' },
            { category: 'Guides', name: 'getting_started', title: 'Getting Started' }
        ]);
        vi.mocked(system_api_service.docs.get_knowledge_doc).mockResolvedValue(
            '---\ntitle: Arch\n---\n# Architecture\n\n## Subsystem Overview\nCore details.'
        );
        vi.mocked(system_api_service.docs.get_operations_manual).mockResolvedValue(
            '# Operations Manual\nOperational SOP.'
        );
    });

    it('fetches document list and auto-selects architecture document', async () => {
        const { result } = renderHook(() => useDocs());

        await waitFor(() => {
            expect(result.current.docs).toHaveLength(2);
        });

        expect(result.current.selectedDoc?.name).toBe('architecture_core');
        expect(result.current.isLoading).toBe(false);
    });

    it('strips frontmatter with both LF and CRLF line breaks', async () => {
        vi.mocked(system_api_service.docs.get_knowledge_doc).mockResolvedValueOnce(
            '---\r\ntitle: Windows Doc\r\ncategory: Core\r\n---\r\n# Windows Guide\r\nContent here.'
        );

        const { result } = renderHook(() => useDocs());

        await waitFor(() => {
            expect(result.current.content).toContain('# Windows Guide');
        });
        expect(result.current.content).not.toContain('title: Windows Doc');
    });

    it('extracts table of contents items from content headers', async () => {
        const { result } = renderHook(() => useDocs());

        await waitFor(() => {
            expect(result.current.toc.length).toBeGreaterThanOrEqual(2);
        });

        expect(result.current.toc[0].text).toBe('Architecture');
        expect(result.current.toc[0].level).toBe(1);
        expect(result.current.toc[1].text).toBe('Subsystem Overview');
        expect(result.current.toc[1].level).toBe(2);
    });

    it('switches between knowledge docs and operations manual', async () => {
        const { result } = renderHook(() => useDocs());

        await waitFor(() => {
            expect(result.current.docs).toHaveLength(2);
        });

        act(() => {
            result.current.set_active_tab('manual');
        });

        await waitFor(() => {
            expect(system_api_service.docs.get_operations_manual).toHaveBeenCalled();
            expect(result.current.content).toContain('# Operations Manual');
        });
    });

    it('handles document fetch failure gracefully', async () => {
        vi.mocked(system_api_service.docs.get_knowledge_doc).mockRejectedValueOnce(new Error('Network offline'));

        const { result } = renderHook(() => useDocs());

        await waitFor(() => {
            expect(result.current.content).toContain('# Connection Failed');
        });
        expect(result.current.content).toContain('Network offline');
    });
});
