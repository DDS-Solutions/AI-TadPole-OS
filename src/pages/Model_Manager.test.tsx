/**
 * @docs ARCHITECTURE:Interface
 * 
 * ### AI Assist Note
 * **Root View**: Manages the Model Manager.test page. 
 * Connects the global state stores to the layout orchestration.
 */

/**
 * @file Model_Manager.test.tsx
 * @description Suite for the AI Provider Manager (Model_Manager) page.
 * @module Pages/Model_Manager
 * @testedBehavior
 * - Vault Locking/Unlocking: Verifies the neural vault security layer.
 * - Provider Management: Infrastructure CRUD (Create, Read, Update, Delete) via use_provider_store.
 * - Model Management: Node-level configuration updates and limits (RPM/TPM).
 * @aiContext
 * - Intensive use of vi.hoisted to share mutable mockState across closure boundaries.
 * - Mocks i18n to return keys for stable assertion matching.
 * - Refactored for 100% snake_case architectural parity.
 * - Verified 154 tests sweep continuation.
 */
import { render, screen, fireEvent, act } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import Model_Manager from './Model_Manager';

/// Mock state using vi.hoisted for reliable sharing with vi.mock
const { mock_state } = vi.hoisted(() => ({
    mock_state: { 
        is_locked: true,
        encrypted_configs: {
            p1: 'sk-mock-key' // Default saved key for testing
        },
        models: [
            { id: 'm1', name: 'gpt-4', provider: 'openai', modality: 'llm', rpm: 10, tpm: 10000 }
        ],
        providers: [
            { id: 'p1', name: 'openai', endpoint: 'https://api.openai.com/v1', protocol: 'openai' }
        ],
        loading: false
    }
}));

// Mock functions
const mock_unlock = vi.fn();
const mock_lock = vi.fn();
const mock_add_provider = vi.fn();
const mock_delete_provider = vi.fn();
const mock_edit_provider = vi.fn();
const mock_add_model = vi.fn();
const mock_edit_model = vi.fn();
const mock_delete_model = vi.fn();

vi.mock('../stores/vault_store', () => ({
    use_vault_store: vi.fn((selector) => {
        const state = {
            is_locked: mock_state.is_locked,
            encrypted_configs: mock_state.encrypted_configs,
            unlock: mock_unlock,
            lock: mock_lock,
            reset_vault: vi.fn(),
            reset_inactivity_timer: vi.fn(),
        };
        return selector ? selector(state) : state;
    })
}));

vi.mock('../stores/model_store', () => ({
    use_model_store: vi.fn((selector) => {
        const state = {
            models: mock_state.models,
            add_model: mock_add_model,
            edit_model: mock_edit_model,
            delete_model: mock_delete_model,
            loading: mock_state.loading,
            get_models: vi.fn(),
        };
        return selector ? selector(state) : state;
    }),
    ModelEntry: vi.fn()
}));

vi.mock('../stores/provider_store', () => ({
    use_provider_store: vi.fn((selector) => {
        const state = {
            providers: mock_state.providers,
            add_provider: mock_add_provider,
            edit_provider: mock_edit_provider,
            delete_provider: mock_delete_provider,
            loading: mock_state.loading,
            get_providers: vi.fn(),
        };
        return selector ? selector(state) : state;
    })
}));

vi.mock('../stores/header_store', () => ({
    use_header_store: vi.fn((selector) => {
        const state = {
            set_header_actions: vi.fn(),
            clear_header_actions: vi.fn(),
        };
        return selector ? selector(state) : state;
    }),
}));

vi.mock('../i18n', () => ({
    i18n: {
        t: (key: string) => key,
    },
}));

vi.mock('../components/ui', () => ({
    Tooltip: ({ children }: any) => <>{children}</>
}));

vi.mock('../components/ui/Confirm_Dialog', () => ({
    Confirm_Dialog: ({ on_confirm, is_open }: any) => {
        if (is_open) {
            on_confirm();
        }
        return null;
    }
}));

describe('Model_Manager Page', () => {
    beforeEach(() => {
        vi.clearAllMocks();
        mock_state.is_locked = true;
        // Mock window.confirm
        window.confirm = vi.fn(() => true);
    });

    describe('Vault Locking Logic', () => {
        it('renders the locked vault state initially', () => {
            render(<Model_Manager />);
            expect(screen.getByText('model_manager.vault.title')).toBeInTheDocument();
        });

        it('calls unlock when entering passphrase and clicking unlock', async () => {
            // Arrange
            mock_unlock.mockResolvedValue({ success: true });
            render(<Model_Manager />);

            // Act
            const input = screen.getByPlaceholderText('model_manager.vault.placeholder_passphrase');
            const button = screen.getByText('model_manager.vault.btn_unlock');

            fireEvent.change(input, { target: { value: 'test-pass' } });
            await act(async () => {
                fireEvent.click(button);
            });

            // Assert
            expect(mock_unlock).toHaveBeenCalledWith('test-pass');
        });

        it('shows errors on failed unlock', async () => {
            mock_unlock.mockResolvedValue({ success: false, error: 'INVALID MASTER KEY' });
            render(<Model_Manager />);
            
            fireEvent.change(screen.getByPlaceholderText('model_manager.vault.placeholder_passphrase'), { target: { value: 'wrong' } });
            await act(async () => {
                fireEvent.click(screen.getByText('model_manager.vault.btn_unlock'));
            });
            
            expect(await screen.findByText('INVALID MASTER KEY')).toBeInTheDocument();
        });
    });

    describe('AI Provider Manager (Unlocked)', () => {
        beforeEach(() => {
            mock_state.is_locked = false;
        });

        it('renders the infrastructure header', async () => {
            render(<Model_Manager />);
            expect(screen.getByText('model_manager.grid.title')).toBeInTheDocument();
        });

        it('shows node and provider controls', () => {
            render(<Model_Manager />);
            expect(screen.getByText('model_manager.grid.btn_add')).toBeInTheDocument();
            expect(screen.getAllByLabelText('model_manager.aria_manage_provider')).toHaveLength(1);
        });

        it('handles provider addition successfully', async () => {
            render(<Model_Manager />);
            fireEvent.click(screen.getByText('model_manager.grid.btn_add'));

            fireEvent.change(screen.getByPlaceholderText('model_manager.add_provider.placeholder_name'), { target: { value: 'anthropic' } });
            
            await act(async () => {
                fireEvent.click(screen.getByText('model_manager.add_provider.btn_init'));
            });

            expect(mock_add_provider).toHaveBeenCalledWith('anthropic', '⚡');
        });

        it('handles provider deletion', async () => {
            render(<Model_Manager />);
            await act(async () => {
                fireEvent.click(screen.getByLabelText('model_manager.aria_terminate_provider'));
            });
            expect(mock_delete_provider).toHaveBeenCalledWith('p1');
        });

        it('handles node addition successfully', async () => {
            render(<Model_Manager />);
            
            // Open model sheet (actually just shown in the inventory table)
            fireEvent.click(screen.getByLabelText('model_manager.aria_manage_provider'));
            
            fireEvent.click(screen.getByLabelText('model_manager.inventory.btn_add'));
            
            fireEvent.change(screen.getByPlaceholderText('model_manager.add_node.placeholder_name'), { target: { value: 'claude-3' } });
            
            await act(async () => {
                fireEvent.click(screen.getByLabelText('Confirm Add Node'));
            });
            
            expect(mock_add_model).toHaveBeenCalledWith('claude-3', 'p1', 'llm', expect.any(Object));
        });

        it('filters models by modality', () => {
            render(<Model_Manager />);
            fireEvent.click(screen.getByLabelText('model_manager.aria_manage_provider'));
            
            const vision_filter = screen.getByText('provider.label_modality_vision');
            fireEvent.click(vision_filter);
            
            // Should hide our llm model
            expect(screen.queryByDisplayValue('gpt-4')).not.toBeInTheDocument();
        });

        it('handles model editing', async () => {
            render(<Model_Manager />);
            fireEvent.click(screen.getByLabelText('model_manager.aria_manage_provider'));
            
            // Enter edit mode
            fireEvent.click(screen.getByLabelText('model_manager.row.tooltip_edit'));
            
            const name_input = screen.getByDisplayValue('gpt-4');
            fireEvent.change(name_input, { target: { value: 'gpt-4o' } });
            
            await act(async () => {
                fireEvent.click(screen.getByLabelText('provider.forge_item.tooltip_save'));
            });
            
            expect(mock_edit_model).toHaveBeenCalledWith('m1', 'gpt-4o', 'openai', 'llm', expect.any(Object));
        });

        it('handles model deletion', async () => {
            render(<Model_Manager />);
            fireEvent.click(screen.getByLabelText('model_manager.aria_manage_provider'));
            
            await act(async () => {
                fireEvent.click(screen.getByLabelText('model_manager.row.tooltip_delete'));
            });
            expect(mock_delete_model).toHaveBeenCalledWith('m1');
        });

        it('toggles limit visibility', () => {
            render(<Model_Manager />);
            fireEvent.click(screen.getByLabelText('model_manager.aria_manage_provider'));
            
            fireEvent.click(screen.getByText('model_manager.row.show_limits'));
            expect(screen.getByText('model_manager.row.hide_limits')).toBeInTheDocument();
            expect(screen.getByText('model_manager.row.req_min')).toBeInTheDocument();
        });
    });
});

