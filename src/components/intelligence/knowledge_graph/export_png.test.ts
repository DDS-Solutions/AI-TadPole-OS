/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Export PNG Unit Tests**: Validates Canvas data URL generation, memory safety checks,
 * offscreen background compositing, and synchronous browser download trigger flow.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Canvas toDataURL failure or missing canvas in DOM.
 * - **Telemetry Link**: Search `[export_png.test]` in tracing logs.
 */

import { renderHook, act } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { useNodeSelection } from './useNodeSelection';

describe('useNodeSelection - exportPNG', () => {
    let querySelectorSpy: ReturnType<typeof vi.spyOn>;

    beforeEach(() => {
        vi.clearAllMocks();
    });

    afterEach(() => {
        querySelectorSpy?.mockRestore();
    });

    it('handles missing canvas element gracefully without throwing', () => {
        const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

        // No .force-graph-container in DOM
        querySelectorSpy = vi.spyOn(document, 'querySelector').mockReturnValue(null);

        const { result } = renderHook(() =>
            useNodeSelection({
                rawGraphData: null,
                rawOkfData: null,
                rawAnomaliesData: null,
                setBlastRadiusLoading: vi.fn(),
                setBlastRadiusError: vi.fn(),
                viewMode: 'symbols',
            })
        );

        act(() => {
            result.current.exportPNG();
        });

        expect(consoleSpy).toHaveBeenCalledWith('[useNodeSelection] Canvas element not found in .force-graph-container');
        consoleSpy.mockRestore();
    });

    it('composites dark background and initiates synchronous PNG file download when canvas is valid', async () => {
        const mockCanvas = {
            width: 800,
            height: 600,
            toDataURL: vi.fn().mockReturnValue('data:image/png;base64,mockpngdata'),
        };

        const mockOffscreenCtx = {
            fillStyle: '',
            fillRect: vi.fn(),
            drawImage: vi.fn(),
        };

        const mockOffscreenCanvas = {
            width: 0,
            height: 0,
            getContext: vi.fn().mockReturnValue(mockOffscreenCtx),
            toDataURL: vi.fn().mockReturnValue('data:image/png;base64,offscreenpngdata'),
        };

        // Mock the DOM query to return a container with a canvas child
        const mockContainer = {
            querySelector: vi.fn().mockReturnValue(mockCanvas),
        };
        querySelectorSpy = vi.spyOn(document, 'querySelector').mockImplementation((selector: string) => {
            if (selector === '.force-graph-container') return mockContainer as any;
            return null;
        });

        const origCreateElement = document.createElement.bind(document);
        const createElementSpy = vi.spyOn(document, 'createElement').mockImplementation((tagName: string) => {
            if (tagName === 'canvas') {
                return mockOffscreenCanvas as any;
            }
            return origCreateElement(tagName);
        });

        const { result } = renderHook(() =>
            useNodeSelection({
                rawGraphData: null,
                rawOkfData: null,
                rawAnomaliesData: null,
                setBlastRadiusLoading: vi.fn(),
                setBlastRadiusError: vi.fn(),
                viewMode: 'symbols',
            })
        );

        const clickSpy = vi.fn();
        const appendChildSpy = vi.spyOn(document.body, 'appendChild').mockImplementation((node) => {
            (node as HTMLElement).click = clickSpy;
            return node;
        });
        const removeChildSpy = vi.spyOn(document.body, 'removeChild').mockImplementation((node) => node);

        act(() => {
            result.current.exportPNG();
        });

        expect(mockOffscreenCanvas.width).toBe(800);
        expect(mockOffscreenCanvas.height).toBe(600);
        expect(mockOffscreenCtx.fillRect).toHaveBeenCalledWith(0, 0, 800, 600);
        expect(mockOffscreenCtx.drawImage).toHaveBeenCalledWith(mockCanvas, 0, 0);
        expect(clickSpy).toHaveBeenCalled();

        createElementSpy.mockRestore();
        appendChildSpy.mockRestore();
        removeChildSpy.mockRestore();
    });

    it('aborts export if canvas exceeds 16M pixel memory safety limit', () => {
        const consoleWarnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});
        const mockCanvas = {
            width: 5000,
            height: 5000, // 25M pixels (> 16M)
            toDataURL: vi.fn(),
        };

        // Mock DOM with oversized canvas
        const mockContainer = {
            querySelector: vi.fn().mockReturnValue(mockCanvas),
        };
        querySelectorSpy = vi.spyOn(document, 'querySelector').mockImplementation((selector: string) => {
            if (selector === '.force-graph-container') return mockContainer as any;
            return null;
        });

        const { result } = renderHook(() =>
            useNodeSelection({
                rawGraphData: null,
                rawOkfData: null,
                rawAnomaliesData: null,
                setBlastRadiusLoading: vi.fn(),
                setBlastRadiusError: vi.fn(),
                viewMode: 'symbols',
            })
        );

        act(() => {
            result.current.exportPNG();
        });

        expect(consoleWarnSpy).toHaveBeenCalledWith('[useNodeSelection] Canvas too large for export');
        expect(mockCanvas.toDataURL).not.toHaveBeenCalled();
        consoleWarnSpy.mockRestore();
    });
});
