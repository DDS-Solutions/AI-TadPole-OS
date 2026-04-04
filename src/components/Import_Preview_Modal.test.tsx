import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Import_Preview_Modal } from './Import_Preview_Modal';

/**
 * Import_Preview_Modal Unit Tests
 * 
 * These tests verify the structured preview and confirmation logic 
 * for the capability import flow, adhering to industry standard AAA pattern.
 */
describe('Import_Preview_Modal', () => {
    const mockOnClose = vi.fn();
    const mockOnConfirm = vi.fn();
    
    // Sample parsed skill data
    const sampleData = {
        name: 'gather_intel',
        description: 'Collects intelligence from public sources.',
        execution_command: 'python intel.py {query}',
        oversight_required: true,
        schema: {},
        category: 'user' as const
    };

    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('renders correctly when open with skill data', () => {
        // Arrange & Act
        render(
            <Import_Preview_Modal 
                isOpen={true} 
                type="skill" 
                data={sampleData} 
                preview="--- name: gather_intel ---" 
                onClose={mockOnClose} 
                onConfirm={mockOnConfirm} 
            />
        );

        // Assert
        expect(screen.getByText(/Import Preview:/i)).toBeInTheDocument();
        expect(screen.getByText(/SKILL/i)).toBeInTheDocument();
        expect(screen.getByDisplayValue('gather_intel')).toBeInTheDocument();
        expect(screen.getByDisplayValue('Collects intelligence from public sources.')).toBeInTheDocument();
        expect(screen.getByText('--- name: gather_intel ---')).toBeInTheDocument();
    });

    it('allows switching between User and AI categories', () => {
        // Arrange
        render(
            <Import_Preview_Modal 
                isOpen={true} 
                type="skill" 
                data={sampleData} 
                preview="" 
                onClose={mockOnClose} 
                onConfirm={mockOnConfirm} 
            />
        );

        // Act - Click AI Services button
        const aiButton = screen.getByText(/AI SERVICES/i);
        fireEvent.click(aiButton);

        // Assert - The emerald styling indicates selection (checking class or implicit state via confirm)
        // We'll verify the final state during confirmation
        const confirmButton = screen.getByText(/Confirm Registration/i);
        fireEvent.click(confirmButton);
        
        expect(mockOnConfirm).toHaveBeenCalledWith(expect.anything(), 'ai');
    });

    it('updates editable fields and returns them on confirm', () => {
        // Arrange
        render(
            <Import_Preview_Modal 
                isOpen={true} 
                type="skill" 
                data={sampleData} 
                preview="" 
                onClose={mockOnClose} 
                onConfirm={mockOnConfirm} 
            />
        );

        // Act - Edit the name and description
        const nameInput = screen.getByDisplayValue('gather_intel');
        fireEvent.change(nameInput, { target: { value: 'gather_intel_v2' } });

        const descInput = screen.getByDisplayValue('Collects intelligence from public sources.');
        fireEvent.change(descInput, { target: { value: 'Updated description' } });

        const confirmButton = screen.getByText(/Confirm Registration/i);
        fireEvent.click(confirmButton);

        // Assert
        expect(mockOnConfirm).toHaveBeenCalledWith(
            expect.objectContaining({
                name: 'gather_intel_v2',
                description: 'Updated description'
            }),
            'user' // Default category
        );
    });

    it('calls onClose when the close icon or backdrop is clicked', () => {
        // Arrange
        const { container } = render(
            <Import_Preview_Modal 
                isOpen={true} 
                type="skill" 
                data={sampleData} 
                preview="" 
                onClose={mockOnClose} 
                onConfirm={mockOnConfirm} 
            />
        );

        // Act - Click close icon
        const closeIcon = screen.getByText('✕');
        fireEvent.click(closeIcon);
        expect(mockOnClose).toHaveBeenCalledTimes(1);

        // Act - Click backdrop (the outer fixed div)
        const backdrop = container.firstChild as HTMLElement;
        fireEvent.click(backdrop);
        expect(mockOnClose).toHaveBeenCalledTimes(2);
    });

    it('returns null when not open', () => {
        // Arrange & Act
        const { container } = render(
            <Import_Preview_Modal 
                isOpen={false} 
                type="skill" 
                data={sampleData} 
                preview="" 
                onClose={mockOnClose} 
                onConfirm={mockOnConfirm} 
            />
        );

        // Assert
        expect(container.firstChild).toBeNull();
    });
});
