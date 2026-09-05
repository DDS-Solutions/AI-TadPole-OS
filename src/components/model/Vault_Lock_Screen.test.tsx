/**
 * @docs ARCHITECTURE:Testing
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Model / Vault_Lock_Screen.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Vault_Lock_Screen } from './Vault_Lock_Screen';

describe('Vault_Lock_Screen Component', () => {
    it('renders master password input and unlock button', () => {
        const on_password_change_mock = vi.fn();
        const on_unlock_mock = vi.fn();

        const { container } = render(
            <Vault_Lock_Screen
                password_input=""
                on_password_change={on_password_change_mock}
                on_unlock={on_unlock_mock}
                error={null}
                is_secure={true}
                show_reset_confirm={false}
                on_set_show_reset_confirm={vi.fn()}
                on_reset_vault={vi.fn()}
            />
        );

        const input = container.querySelector('input[id="master-passphrase"]') as HTMLInputElement;
        expect(input).toBeDefined();

        fireEvent.change(input, { target: { value: 'Secret123!' } });
        expect(on_password_change_mock).toHaveBeenCalledWith('Secret123!');
    });

    it('renders error message when error is provided', () => {
        render(
            <Vault_Lock_Screen
                password_input="wrong"
                on_password_change={vi.fn()}
                on_unlock={vi.fn()}
                error="Invalid decryption passphrase"
                is_secure={true}
                show_reset_confirm={false}
                on_set_show_reset_confirm={vi.fn()}
                on_reset_vault={vi.fn()}
            />
        );

        expect(screen.getByText('Invalid decryption passphrase')).toBeDefined();
    });
});
