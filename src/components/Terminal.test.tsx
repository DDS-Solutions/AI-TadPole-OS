/**
 * @docs ARCHITECTURE:UI-Components
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / General / Terminal.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import '@testing-library/jest-dom';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import Terminal_Component from './Terminal';
import { system_api_service } from '../services/system_api_service';

// Mock system_api_service
vi.mock('../services/system_api_service', () => ({
    system_api_service: {
        workspace: {
            get_workspace_files: vi.fn().mockResolvedValue([
                'src/components/Terminal.tsx',
                'server-rs/src/main.rs',
                'package.json'
            ])
        }
    }
}));

// Mock i18n
vi.mock('../i18n', () => ({
    i18n: {
        t: (key: string) => {
            if (key === 'terminal.toggle_aria') return 'Toggle Terminal';
            if (key === 'terminal.input_aria') return 'Command Input';
            return key;
        },
    },
}));

// Mock use_settings_store
vi.mock('../stores/settings_store', () => ({
    use_settings_store: () => ({
        settings: { is_safe_mode: false }
    })
}));

describe('Terminal Autocomplete UX', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('displays file suggestions dropdown when typing @', async () => {
        render(<Terminal_Component agents={[]} />);
        
        // Open the terminal
        const toggle_btn = screen.getByLabelText('Toggle Terminal');
        fireEvent.click(toggle_btn);
        
        const input = screen.getByLabelText('Command Input') as HTMLInputElement;
        
        // Type '@'
        fireEvent.change(input, { target: { value: '@', selectionStart: 1, selectionEnd: 1 } });

        // Wait for system_api_service to be called
        await waitFor(() => {
            expect(system_api_service.workspace.get_workspace_files).toHaveBeenCalled();
        });

        // Verify file suggestions are rendered
        await waitFor(() => {
            expect(screen.getByText('src/components/Terminal.tsx')).toBeInTheDocument();
            expect(screen.getByText('server-rs/src/main.rs')).toBeInTheDocument();
        });
    });

    it('filters file suggestions as the user types', async () => {
        render(<Terminal_Component agents={[]} />);
        
        const toggle_btn = screen.getByLabelText('Toggle Terminal');
        fireEvent.click(toggle_btn);
        
        // Wait for system_api_service to be called
        await waitFor(() => {
            expect(system_api_service.workspace.get_workspace_files).toHaveBeenCalled();
        });

        const input = screen.getByLabelText('Command Input') as HTMLInputElement;
        
        // Type '@ser'
        fireEvent.change(input, { target: { value: '@ser', selectionStart: 4, selectionEnd: 4 } });

        await waitFor(() => {
            expect(screen.getByText('server-rs/src/main.rs')).toBeInTheDocument();
        });
        
        expect(screen.queryByText('src/components/Terminal.tsx')).not.toBeInTheDocument();
    });

    it('completes the selection when pressing Tab/Enter', async () => {
        render(<Terminal_Component agents={[]} />);
        
        const toggle_btn = screen.getByLabelText('Toggle Terminal');
        fireEvent.click(toggle_btn);
        
        // Wait for system_api_service to be called
        await waitFor(() => {
            expect(system_api_service.workspace.get_workspace_files).toHaveBeenCalled();
        });

        const input = screen.getByLabelText('Command Input') as HTMLInputElement;
        
        // Type '@pack'
        fireEvent.change(input, { target: { value: '@pack', selectionStart: 5, selectionEnd: 5 } });

        await waitFor(() => {
            expect(screen.getByText('package.json')).toBeInTheDocument();
        });

        // Press Tab key
        fireEvent.keyDown(input, { key: 'Tab' });

        // Verify input is updated (replacing '@pack' with 'package.json ')
        expect(input.value).toBe('package.json ');
        
        // Verify dropdown is closed
        expect(screen.queryByText('package.json')).not.toBeInTheDocument();
    });
});
