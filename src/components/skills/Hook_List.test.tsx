/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Verification of the Hook List rendering** and trigger behavior.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Tooltip or button click handler failure.
 * - **Telemetry Link**: Search `[Hook_List.test]` in tracing logs.
 */

import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { Hook_List } from './Hook_List';
import type { Hook_Definition } from '../../stores/skill_store';

// Mock components
vi.mock('../ui', () => ({
    Tooltip: ({ children, content }: { children: React.ReactNode, content?: string }) => (
        <div data-testid="tooltip-wrapper" data-tooltip-content={content}>
            {children}
        </div>
    ),
    Tw_Empty_State: ({ title, description, action }: any) => (
        <div data-testid="empty-state">
            <div>{title}</div>
            <div>{description}</div>
            <div>{action}</div>
        </div>
    ),
}));

// Mock i18n
vi.mock('../../i18n', () => ({
    i18n: {
        t: (key: string) => key,
    },
}));

describe('Hook_List', () => {
    const mock_handlers = {
        on_edit: vi.fn(),
        on_delete: vi.fn(),
        on_create: vi.fn(),
    };

    const mock_hooks: Hook_Definition[] = [
        {
            name: 'test_hook_1',
            description: 'Test description 1',
            hook_type: 'pre_validation',
            content: 'test content 1',
            category: 'user',
            active: true
        },
        {
            name: 'test_hook_2',
            description: 'Test description 2',
            hook_type: 'post_analysis',
            content: 'test content 2',
            category: 'ai',
            active: false
        }
    ];

    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('renders empty state with register tooltip when hooks list is empty', () => {
        render(<Hook_List hooks={[]} {...mock_handlers} />);

        expect(screen.getByTestId('empty-state')).toBeInTheDocument();
        expect(screen.getByText('skills.hooks_empty_title')).toBeInTheDocument();
        
        const tooltip = screen.getByTestId('tooltip-wrapper');
        expect(tooltip).toHaveAttribute('data-tooltip-content', 'skills.tooltip_register_hook_card');
        
        const btn = screen.getByText('skills.add_hook_button');
        fireEvent.click(btn);
        expect(mock_handlers.on_create).toHaveBeenCalled();
    });

    it('renders list of hooks and a registration card with tooltip when hooks list is not empty', () => {
        render(<Hook_List hooks={mock_hooks} {...mock_handlers} />);

        // Verify active hooks rendered (via Hook_Card)
        expect(screen.getByText('test_hook_1')).toBeInTheDocument();
        expect(screen.getByText('test_hook_2')).toBeInTheDocument();

        // Verify registration card rendered
        const tooltips = screen.getAllByTestId('tooltip-wrapper');
        const register_tooltip = tooltips.find(t => t.getAttribute('data-tooltip-content') === 'skills.tooltip_register_hook_card');
        expect(register_tooltip).toBeDefined();

        const register_btn = register_tooltip?.querySelector('button');
        expect(register_btn).toBeInTheDocument();
        
        if (register_btn) fireEvent.click(register_btn);
        expect(mock_handlers.on_create).toHaveBeenCalled();
    });
});

// Metadata: [Hook_List_test]
