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
 * - Mocks window.confirm globally to bypass deletion safety checks.
 */
import { render, screen, fireEvent, act } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import Model_Manager from './Model_Manager';

// Mock state using vi.hoisted for reliable sharing with vi.mock
// Note: This allows us to toggle isLocked mid-test by mutating mockState
const { mockState } = vi.hoisted(() => ({
    mockState: { isLocked: true }
}));

// Mock functions for use_provider_store
const mockUnlock = vi.fn();
const mockLock = vi.fn();
const mockAddProvider = vi.fn();
const mockDeleteProvider = vi.fn();
const mockSetProviderConfig = vi.fn();
const mockEditProvider = vi.fn();
const mockAddModel = vi.fn();
const mockEditModel = vi.fn();
const mockDeleteModel = vi.fn();

vi.mock('../stores/vault_store', () => ({
    use_vault_store: vi.fn(() => ({
        isLocked: mockState.isLocked,
        unlock: mockUnlock,
        lock: mockLock,
        resetVault: vi.fn(),
    }))
}));

vi.mock('../stores/model_store', () => ({
    use_model_store: vi.fn(() => ({
        models: [
            { id: 'm1', name: 'gpt-4', provider: 'openai', modality: 'llm', rpm: 10, tpm: 10000 }
        ],
        addModel: mockAddModel,
        editModel: mockEditModel,
        delete_model: mockDeleteModel,
    })),
    ModelEntry: vi.fn()
}));

vi.mock('../stores/provider_store', () => ({
    use_provider_store: vi.fn(() => ({
        providers: [
            { id: 'openai', name: 'OpenAI', icon: '🤖', protocol: 'openai' }
        ],
        addProvider: mockAddProvider,
        delete_provider: mockDeleteProvider,
        editProvider: mockEditProvider,
        setProviderConfig: mockSetProviderConfig,
    }))
}));

vi.mock('../components/ui', () => ({
    Tooltip: ({ children }: any) => <>{children}</>
}));

vi.mock('../components/ui/Confirm_Dialog', () => ({
    Confirm_Dialog: ({ onConfirm, isOpen }: any) => {
        if (isOpen) {
            // Auto-confirm in tests to bypass the dialog
            onConfirm();
        }
        return null;
    }
}));

describe('Model_Manager Page', () => {
    beforeEach(() => {
        vi.clearAllMocks();
        mockState.isLocked = true;
        // Mock window.confirm
        window.confirm = vi.fn(() => true);
    });

    describe('Vault Locking Logic', () => {
        it('renders the locked vault state initially', () => {
            render(<Model_Manager />);
            expect(screen.getByText(/NEURAL VAULT/i)).toBeInTheDocument();
        });

        it('calls unlock when entering passphrase and clicking unlock', async () => {
            // Arrange
            mockUnlock.mockResolvedValue(true);
            render(<Model_Manager />);

            // Act
            const input = screen.getByPlaceholderText(/MASTER PASSPHRASE/i);
            const button = screen.getByText('Commit Authorization');

            fireEvent.change(input, { target: { value: 'test-pass' } });
            await act(async () => {
                fireEvent.click(button);
            });

            // Assert
            expect(mockUnlock).toHaveBeenCalledWith('test-pass');
        });

        it('shows errors on failed unlock', async () => {
            mockUnlock.mockResolvedValue(false);
            render(<Model_Manager />);
            
            fireEvent.change(screen.getByPlaceholderText(/MASTER PASSPHRASE/i), { target: { value: 'wrong' } });
            await act(async () => {
                fireEvent.click(screen.getByText('Commit Authorization'));
            });
            
            expect(await screen.findByText(/INVALID MASTER KEY/i)).toBeInTheDocument();
        });
    });

    describe('AI Provider Manager (Unlocked)', () => {
        beforeEach(() => {
            mockState.isLocked = false;
        });

        it('renders the infrastructure header', async () => {
            render(<Model_Manager />);
            expect(screen.getByText(/Neural Infrastructure Grid/i)).toBeInTheDocument();
        });

        it('shows node and provider controls', () => {
            render(<Model_Manager />);
            expect(screen.getByText(/Add Provider/i)).toBeInTheDocument();
            expect(screen.getAllByLabelText(/Add Node/i).length).toBeGreaterThan(0);
        });

        it('handles provider addition successfully', async () => {
            render(<Model_Manager />);
            fireEvent.click(screen.getByText(/Add Provider/i));
            
            fireEvent.change(screen.getByPlaceholderText(/NAME/i), { target: { value: 'Anthropic' } });
            fireEvent.change(screen.getByPlaceholderText('⚡'), { target: { value: '❄️' } });
            await act(async () => {
                fireEvent.click(screen.getByText('Initialize'));
            });
            
            expect(mockAddProvider).toHaveBeenCalledWith('Anthropic', '❄️');
            expect(screen.queryByPlaceholderText(/NAME/i)).not.toBeInTheDocument();
        });

        it('handles provider deletion', async () => {
            render(<Model_Manager />);
            await act(async () => {
                fireEvent.click(screen.getByLabelText(/Terminate Provider/i));
            });
            expect(mockDeleteProvider).toHaveBeenCalledWith('openai');
        });

        it('handles node addition successfully', () => {
            render(<Model_Manager />);
            fireEvent.click(screen.getByLabelText(/Add Node/i));
            
            fireEvent.change(screen.getByPlaceholderText(/MODEL NAME/i), { target: { value: 'claude-3-opus' } });
            fireEvent.click(screen.getByLabelText(/Confirm Add Node/i));
            
            expect(mockAddModel).toHaveBeenCalledWith('claude-3-opus', 'openai', 'llm', expect.any(Object));
        });

        it('filters models by modality', () => {
            render(<Model_Manager />);
            const select = screen.getByDisplayValue('All Modalities');
            fireEvent.change(select, { target: { value: 'vision' } });
            
            // Should filter out 'gpt-4' which is 'llm'
            expect(screen.queryByText(/gpt-4/i)).not.toBeInTheDocument();
        });

        it('handles model editing', () => {
            render(<Model_Manager />);
            fireEvent.click(screen.getByLabelText(/Edit Model/i));
            
            const nameInput = screen.getByDisplayValue('gpt-4');
            fireEvent.change(nameInput, { target: { value: 'gpt-4o' } });
            fireEvent.click(screen.getByLabelText(/Save Changes/i));
            
            expect(mockEditModel).toHaveBeenCalledWith('m1', 'gpt-4o', 'openai', 'llm', expect.any(Object));
        });

        it('handles model deletion', async () => {
            render(<Model_Manager />);
            await act(async () => {
                fireEvent.click(screen.getByLabelText(/Delete Model/i));
            });
            expect(mockDeleteModel).toHaveBeenCalledWith('m1');
        });

        it('toggles limit visibility', () => {
            render(<Model_Manager />);
            fireEvent.click(screen.getByText(/Show TPM Limits/i));
            expect(screen.getByText(/Hide TPM Limits/i)).toBeInTheDocument();
            expect(screen.getByText(/Req \/ Minute/i)).toBeInTheDocument();
        });
    });
});
