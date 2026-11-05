/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Assist Note
 * - Keep fixtures isolated and assert the public behavior of the extracted boundaries.
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / General / Provider_Config_Panel.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import '@testing-library/jest-dom';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import Provider_Config_Panel from './Provider_Config_Panel';
import { provider_configuration } from '../services/provider_configuration';

// Mock Stores
vi.mock('../stores/provider_store', () => ({
    use_provider_store: () => ({
        edit_provider: vi.fn(),
        set_provider_config: vi.fn().mockResolvedValue(true),
    })
}));

vi.mock('../stores/vault_store', () => ({
    use_vault_store: () => ({
        encrypted_configs: {},
    })
}));

vi.mock('../stores/model_store', () => ({
    use_model_store: () => ({
        models: [],
        add_model: vi.fn(),
        edit_model: vi.fn(),
        delete_model: vi.fn(),
    })
}));

vi.mock('../services/provider_configuration', () => ({
    provider_configuration: {
        save: vi.fn().mockResolvedValue(undefined),
        test: vi.fn().mockResolvedValue({ status: 'success', message: 'Handshake complete' }),
        discover: vi.fn().mockResolvedValue({ message: 'Models refreshed' })
    }
}));

// Mock i18n
vi.mock('../i18n', () => ({
    i18n: {
        t: (key: string) => key,
    },
}));

describe('Provider_Config_Panel', () => {
    const mock_provider = {
        id: 'openai',
        name: 'OpenAI',
        icon: '🤖',
        base_url: 'https://api.openai.com/v1',
        protocol: 'openai'
    } as any;

    const mock_on_close = vi.fn();

    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('renders all modular sections', () => {
        render(<Provider_Config_Panel provider={mock_provider} on_close={mock_on_close} />);
        
        // Identity Header
        expect(screen.getByDisplayValue('OpenAI')).toBeInTheDocument();
        // Auth Section Label
        expect(screen.getByText('provider.auth_layer')).toBeInTheDocument();
        // Protocol Section Label
        expect(screen.getByText('provider.transmission_protocol')).toBeInTheDocument();
        // Model Forge Label
        expect(screen.getByText('provider.intelligence_forge')).toBeInTheDocument();
    });

    it('shows Local_Server_Module only when localhost is in endpoint', () => {
        const { unmount } = render(<Provider_Config_Panel provider={mock_provider} on_close={mock_on_close} />);
        
        // Should NOT show local orchestra by default for OpenAI
        expect(screen.queryByText('provider.local_server_orchestration')).not.toBeInTheDocument();

        unmount();

        // Update provider to a localhost one and re-render
        const local_provider = { ...mock_provider, base_url: 'http://localhost:11434' };
        render(<Provider_Config_Panel provider={local_provider} on_close={mock_on_close} />);

        // Should now show local orchestra
        expect(screen.getByText('provider.local_server_orchestration')).toBeInTheDocument();
    });

    it('triggers test handshake when Test Trace clicked', async () => {
        render(<Provider_Config_Panel provider={mock_provider} on_close={mock_on_close} />);
        
        const test_button = screen.getByLabelText('provider.aria_test_connection');
        
        // Fill API key to pass validation
        const api_key_input = screen.getByLabelText('provider.field_api_key');
        fireEvent.change(api_key_input, { target: { value: 'sk-test-key' } });
        
        fireEvent.click(test_button);

        await waitFor(() => {
            expect(provider_configuration.test).toHaveBeenCalled();
            expect(screen.getByText('Handshake complete')).toBeInTheDocument();
        });
    });

    it('calls on_close when clicking the background', () => {
        render(<Provider_Config_Panel provider={mock_provider} on_close={mock_on_close} />);
        
        // Find the background overlay by its aria-label (mocked i18n returns key)
        const backdrop = screen.getByLabelText(/common.close_modal/i);
        fireEvent.click(backdrop);
        
        expect(mock_on_close).toHaveBeenCalled();
    });

    it('shows error message for invalid JSON in custom headers', async () => {
        render(<Provider_Config_Panel provider={mock_provider} on_close={mock_on_close} />);
        
        const headers_input = screen.getByPlaceholderText('provider.placeholder_headers');
        fireEvent.change(headers_input, { target: { value: '{ invalid: json }' } });
        
        expect(screen.getByText('provider.err_invalid_json')).toBeInTheDocument();
    });

    it('passes blank input as keep so the command can resolve saved credentials', async () => {
        render(<Provider_Config_Panel provider={mock_provider} on_close={mock_on_close} />);
        
        const test_button = screen.getByLabelText('provider.aria_test_connection');
        fireEvent.click(test_button);
        
        await waitFor(() => expect(provider_configuration.test).toHaveBeenCalledWith('openai', expect.any(Object), { kind: 'keep' }));
    });

    it('allows connection testing without API key for Ollama provider', async () => {
        const ollama_provider = {
            id: 'ollama',
            name: 'Ollama',
            icon: '🦙',
            base_url: 'http://127.0.0.1:11434/v1',
            protocol: 'ollama'
        } as any;
        render(<Provider_Config_Panel provider={ollama_provider} on_close={mock_on_close} />);
        
        const test_button = screen.getByLabelText('provider.aria_test_connection');
        fireEvent.click(test_button);
        
        await waitFor(() => {
            expect(provider_configuration.test).toHaveBeenCalledWith('ollama', expect.objectContaining({ protocol: 'ollama' }), { kind: 'keep' });
        });
    });
});
