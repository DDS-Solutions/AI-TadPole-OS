/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Tests the data mapping and validation logic** for the Agent/Mission import preview modal. 
 * Verifies category switching (User vs AI) and confirms specific field edit propagation before database write.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Malformed JSON import payloads causing UI crashes or incorrect 'duplicate ID' warning triggers during confirmation.
 * - **Telemetry Link**: Search `[Import_Preview_Modal.test]` in tracing logs.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Import_Preview_Modal } from './Import_Preview_Modal';

// Mock i18n
vi.mock('../i18n', () => ({
    i18n: {
        t: (key: string, options?: any) => {
            if (key === 'import.title') return `Import Preview: ${options?.type}`;
            return key;
        },
    },
}));

describe('Import_Preview_Modal', () => {
    const mock_on_close = vi.fn();
    const mock_on_confirm = vi.fn();
    
    const sample_data = {
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
        render(
            <Import_Preview_Modal 
                is_open={true} 
                type="skill" 
                data={sample_data} 
                preview="--- name: gather_intel ---" 
                on_close={mock_on_close} 
                on_confirm={mock_on_confirm} 
            />
        );

        expect(screen.getByText(/Import Preview: SKILL/i)).toBeInTheDocument();
        expect(screen.getByDisplayValue('gather_intel')).toBeInTheDocument();
        expect(screen.getByDisplayValue('Collects intelligence from public sources.')).toBeInTheDocument();
        expect(screen.getByText('--- name: gather_intel ---')).toBeInTheDocument();
    });

    it('allows switching between User and AI categories', () => {
        render(
            <Import_Preview_Modal 
                is_open={true} 
                type="skill" 
                data={sample_data} 
                preview="" 
                on_close={mock_on_close} 
                on_confirm={mock_on_confirm} 
            />
        );

        // Click AI Services button
        const ai_button = screen.getByText('import.ai_services');
        fireEvent.click(ai_button);

        const confirm_button = screen.getByText('import.btn_confirm');
        fireEvent.click(confirm_button);
        
        expect(mock_on_confirm).toHaveBeenCalledWith(expect.anything(), 'ai');
    });

    it('updates editable fields and returns them on confirm', () => {
        render(
            <Import_Preview_Modal 
                is_open={true} 
                type="skill" 
                data={sample_data} 
                preview="" 
                on_close={mock_on_close} 
                on_confirm={mock_on_confirm} 
            />
        );

        const name_input = screen.getByDisplayValue('gather_intel');
        fireEvent.change(name_input, { target: { value: 'gather_intel_v2' } });

        const desc_input = screen.getByDisplayValue('Collects intelligence from public sources.');
        fireEvent.change(desc_input, { target: { value: 'Updated description' } });

        const confirm_button = screen.getByText('import.btn_confirm');
        fireEvent.click(confirm_button);

        expect(mock_on_confirm).toHaveBeenCalledWith(
            expect.objectContaining({
                name: 'gather_intel_v2',
                description: 'Updated description'
            }),
            'user'
        );
    });

    it('calls on_close when the close icon is clicked', () => {
        render(
            <Import_Preview_Modal 
                is_open={true} 
                type="skill" 
                data={sample_data} 
                preview="" 
                on_close={mock_on_close} 
                on_confirm={mock_on_confirm} 
            />
        );

        const close_icon = screen.getByText('✕');
        fireEvent.click(close_icon);
        expect(mock_on_close).toHaveBeenCalledTimes(1);
    });

    it('returns null when not open', () => {
        const { container } = render(
            <Import_Preview_Modal 
                is_open={false} 
                type="skill" 
                data={sample_data} 
                preview="" 
                on_close={mock_on_close} 
                on_confirm={mock_on_confirm} 
            />
        );

        expect(container.firstChild).toBeNull();
    });
});

