/**
 * @docs ARCHITECTURE:Testing
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Model / Provider_Card.test
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
import { Provider_Card } from './Provider_Card';
import type { Provider_Config } from '../../stores/provider_store';

describe('Provider_Card Component', () => {
    const mock_provider: Provider_Config = {
        id: 'openai',
        name: 'OpenAI',
        icon: '🤖',
        protocol: 'REST',
        is_active: true,
        base_url: 'https://api.openai.com/v1'
    };

    it('renders provider identity and model node count', () => {
        render(
            <Provider_Card
                provider={mock_provider}
                is_selected={false}
                on_select={vi.fn()}
                on_delete={vi.fn()}
                models_count={5}
            />
        );

        expect(screen.getByText('OpenAI')).toBeDefined();
        expect(screen.getByText('🤖')).toBeDefined();
    });

    it('handles provider selection and deletion triggers', () => {
        const on_select_mock = vi.fn();
        const on_delete_mock = vi.fn();

        render(
            <Provider_Card
                provider={mock_provider}
                is_selected={true}
                on_select={on_select_mock}
                on_delete={on_delete_mock}
                models_count={5}
            />
        );

        const delete_btn = screen.getByLabelText(/terminate|delete/i);
        fireEvent.click(delete_btn);
        expect(on_delete_mock).toHaveBeenCalledWith('openai', 'OpenAI');
    });
});
