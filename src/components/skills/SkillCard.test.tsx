import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { SkillCard } from './SkillCard';
import type { SkillDefinition } from '../../stores/skillStore';

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

describe('SkillCard', () => {
    const mockSkill: SkillDefinition = {
        name: 'test_skill',
        description: 'Test description',
        execution_command: 'python test.py',
        schema: {},
        category: 'user'
    };

    const mockHandlers = {
        onEdit: vi.fn(),
        onAssign: vi.fn(),
        onDelete: vi.fn(),
    };

    it('renders skill information correctly', () => {
        render(<SkillCard skill={mockSkill} {...mockHandlers} />);
        
        expect(screen.getByText('test_skill')).toBeInTheDocument();
        expect(screen.getByText('Test description')).toBeInTheDocument();
        expect(screen.getByText('python test.py')).toBeInTheDocument();
    });

    it('calls onEdit when edit button is clicked', () => {
        render(<SkillCard skill={mockSkill} {...mockHandlers} />);
        
        const tooltips = screen.getAllByTestId('tooltip-wrapper');
        const editTooltip = tooltips.find(t => t.getAttribute('data-tooltip-content') === 'skills.tooltip_edit_skill');
        const editBtnInTooltip = editTooltip?.querySelector('button');
        
        if (editBtnInTooltip) fireEvent.click(editBtnInTooltip);
        expect(mockHandlers.onEdit).toHaveBeenCalledWith(mockSkill);
    });

    it('calls onAssign when assign button is clicked', () => {
        render(<SkillCard skill={mockSkill} {...mockHandlers} />);
        
        const tooltips = screen.getAllByTestId('tooltip-wrapper');
        const assignTooltip = tooltips.find(t => t.getAttribute('data-tooltip-content') === 'agent_manager.tooltip_assign');
        const assignBtn = assignTooltip?.querySelector('button');
        
        if (assignBtn) fireEvent.click(assignBtn);
        expect(mockHandlers.onAssign).toHaveBeenCalledWith('test_skill');
    });

    it('calls onDelete when delete button is clicked', () => {
        render(<SkillCard skill={mockSkill} {...mockHandlers} />);
        
        const tooltips = screen.getAllByTestId('tooltip-wrapper');
        const deleteTooltip = tooltips.find(t => t.getAttribute('data-tooltip-content') === 'skills.tooltip_delete_skill');
        const deleteBtn = deleteTooltip?.querySelector('button');
        
        if (deleteBtn) fireEvent.click(deleteBtn);
        expect(mockHandlers.onDelete).toHaveBeenCalledWith('test_skill');
    });
});
