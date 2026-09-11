/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / General / Command_Palette.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import React from 'react';
import { Command_Palette } from './Command_Palette';

const mockNavigate = vi.fn();
const mockSetSelectedAgentId = vi.fn();
const mockSetScope = vi.fn();
const mockClearHistory = vi.fn();
const mockOnClose = vi.fn();

vi.mock('react-router-dom', () => ({
    useNavigate: () => mockNavigate,
}));

vi.mock('framer-motion', () => ({
    motion: {
        div: ({ children, ...props }: any) => <div {...props}>{children}</div>,
    },
    AnimatePresence: ({ children }: any) => <>{children}</>,
}));

vi.mock('../stores/agent_store', () => ({
    use_agent_store: () => ({
        agents: [
            { id: 'agent-1', name: 'Alpha Agent', role: 'Security Sentinel' },
            { id: 'agent-2', name: 'Beta Agent', role: 'Data Scout' },
        ],
    }),
}));

vi.mock('../stores/sovereign_store', () => {
    const fn = vi.fn(() => ({
        set_selected_agent_id: mockSetSelectedAgentId,
        set_scope: mockSetScope,
    }));
    (fn as any).getState = () => ({
        clear_history: mockClearHistory,
    });
    return { use_sovereign_store: fn };
});

vi.mock('../services/agent', () => ({
    agent_api_service: {
        search_memory: vi.fn().mockResolvedValue({ status: 'success', entries: [] }),
    },
}));

vi.mock('../i18n', () => ({
    i18n: {
        t: (key: string) => key,
    },
}));

describe('Command_Palette', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('does not render dialog elements when is_open is false', () => {
        render(<Command_Palette is_open={false} on_close={mockOnClose} />);
        expect(screen.queryByPlaceholderText('command.search_placeholder')).toBeNull();
    });

    it('renders input and available options when is_open is true', () => {
        render(<Command_Palette is_open={true} on_close={mockOnClose} />);
        expect(screen.getByPlaceholderText('command.search_placeholder')).toBeDefined();
        expect(screen.getByText('Alpha Agent')).toBeDefined();
        expect(screen.getByText('command.ops_center')).toBeDefined();
    });

    it('handles keyboard navigation with ArrowDown and Enter', () => {
        render(<Command_Palette is_open={true} on_close={mockOnClose} />);
        const input = screen.getByPlaceholderText('command.search_placeholder');

        // Arrow down to move selection
        fireEvent.keyDown(input, { key: 'ArrowDown' });
        // Press enter on the second item (Beta Agent)
        fireEvent.keyDown(input, { key: 'Enter' });

        expect(mockSetSelectedAgentId).toHaveBeenCalledWith('agent-2');
        expect(mockSetScope).toHaveBeenCalledWith('agent');
        expect(mockOnClose).toHaveBeenCalled();
    });

    it('handles ArrowUp to wrap around backwards', () => {
        render(<Command_Palette is_open={true} on_close={mockOnClose} />);
        const input = screen.getByPlaceholderText('command.search_placeholder');

        // Arrow up wraps from index 0 to last index
        fireEvent.keyDown(input, { key: 'ArrowUp' });
        fireEvent.keyDown(input, { key: 'Enter' });

        expect(mockOnClose).toHaveBeenCalled();
    });

    it('safely handles ArrowDown and ArrowUp when no items match (prevents NaN index trap)', () => {
        render(<Command_Palette is_open={true} on_close={mockOnClose} />);
        const input = screen.getByPlaceholderText('command.search_placeholder');

        // Type query that matches nothing
        fireEvent.change(input, { target: { value: 'nonexistent-query-xyz-999' } });
        expect(screen.getByText('command.no_results')).toBeDefined();

        // Arrow navigation with 0 items must not crash or set NaN
        expect(() => {
            fireEvent.keyDown(input, { key: 'ArrowDown' });
            fireEvent.keyDown(input, { key: 'ArrowUp' });
            fireEvent.keyDown(input, { key: 'Enter' });
        }).not.toThrow();
    });

    it('calls on_close on Escape key', () => {
        render(<Command_Palette is_open={true} on_close={mockOnClose} />);
        const input = screen.getByPlaceholderText('command.search_placeholder');

        fireEvent.keyDown(input, { key: 'Escape' });
        expect(mockOnClose).toHaveBeenCalled();
    });

    it('filters items based on user input', () => {
        render(<Command_Palette is_open={true} on_close={mockOnClose} />);
        const input = screen.getByPlaceholderText('command.search_placeholder');

        fireEvent.change(input, { target: { value: 'Alpha' } });
        expect(screen.getByText('Alpha Agent')).toBeDefined();
        expect(screen.queryByText('Beta Agent')).toBeNull();
    });
});
