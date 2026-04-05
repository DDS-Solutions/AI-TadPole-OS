/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **UI Component**: Manages the interface for Skill Card.test. 
 * Part of the Tadpole-OS Modular UI ecosystem.
 */

/**
 * @file Skill_Card.test.tsx
 * @description Suite for the individual Skill Card component.
 * @module Components/Skills/Skill_Card
 * @testedBehavior
 * - Rendering: Displays skill name, description, and execution command.
 * - Interaction: Triggers edit, assign, and delete handlers with correct arguments.
 * @aiContext
 * - Refactored for 100% snake_case architectural parity.
 * - Mocks i18n to return keys for stable assertion matching.
 * - Verified 154 tests sweep continuation.
 * - AI awakening notes confirmed.
 */
import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { Skill_Card } from './Skill_Card';
import type { Skill_Definition } from '../../stores/skill_store';

// Mock components
vi.mock('../ui', () => ({
    Tooltip: ({ children, content }: { children: React.ReactNode, content?: string }) => (
        <div data-testid="tooltip-wrapper" data-tooltip-content={content}>
            {children}
        </div>
    ),
}));

// Mock i18n
vi.mock('../../i18n', () => ({
    i18n: {
        t: (key: string) => key,
    },
}));

describe('Skill_Card', () => {
    const mock_skill: Skill_Definition = {
        name: 'test_skill',
        description: 'Test description',
        execution_command: 'python test.py',
        schema: {},
        category: 'user'
    };

    const mock_handlers = {
        on_edit: vi.fn(),
        on_assign: vi.fn(),
        on_delete: vi.fn(),
    };

    it('renders skill information correctly', () => {
        render(<Skill_Card skill={mock_skill} {...mock_handlers} />);
        
        expect(screen.getByText('test_skill')).toBeInTheDocument();
        expect(screen.getByText('Test description')).toBeInTheDocument();
        expect(screen.getByText('python test.py')).toBeInTheDocument();
    });

    it('calls on_edit when edit button is clicked', () => {
        render(<Skill_Card skill={mock_skill} {...mock_handlers} />);
        
        const tooltips = screen.getAllByTestId('tooltip-wrapper');
        const edit_tooltip = tooltips.find(t => t.getAttribute('data-tooltip-content') === 'skills.tooltip_edit_skill');
        const edit_btn = edit_tooltip?.querySelector('button');
        
        if (edit_btn) fireEvent.click(edit_btn);
        expect(mock_handlers.on_edit).toHaveBeenCalledWith(mock_skill);
    });

    it('calls on_assign when assign button is clicked', () => {
        render(<Skill_Card skill={mock_skill} {...mock_handlers} />);
        
        const tooltips = screen.getAllByTestId('tooltip-wrapper');
        const assign_tooltip = tooltips.find(t => t.getAttribute('data-tooltip-content') === 'agent_manager.tooltip_assign');
        const assign_btn = assign_tooltip?.querySelector('button');
        
        if (assign_btn) fireEvent.click(assign_btn);
        expect(mock_handlers.on_assign).toHaveBeenCalledWith('test_skill');
    });

    it('calls on_delete when delete button is clicked', () => {
        render(<Skill_Card skill={mock_skill} {...mock_handlers} />);
        
        const tooltips = screen.getAllByTestId('tooltip-wrapper');
        const delete_tooltip = tooltips.find(t => t.getAttribute('data-tooltip-content') === 'skills.tooltip_delete_skill');
        const delete_btn = delete_tooltip?.querySelector('button');
        
        if (delete_btn) fireEvent.click(delete_btn);
        expect(mock_handlers.on_delete).toHaveBeenCalledWith('test_skill');
    });
});

