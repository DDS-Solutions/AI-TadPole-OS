/**
 * @docs ARCHITECTURE:TestSuites
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Template_Store / Model_Resolver_Select.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: `Model_Resolver_Select.test.tsx`
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import { Model_Resolver_Select } from './Model_Resolver_Select';
import { use_settings_store } from '../../stores/settings_store';
import type { ModelMappingSelection } from './types';

vi.mock('../../stores/settings_store', () => ({
    use_settings_store: vi.fn()
}));

describe('Model_Resolver_Select Component', () => {
    const mockSettings = {
        default_provider: 'ollama-cloud',
        default_model: 'gemma4:31b-cloud'
    };

    beforeEach(() => {
        vi.clearAllMocks();
        vi.mocked(use_settings_store).mockReturnValue({
            settings: mockSettings
        } as any);
    });

    it('renders all mapping options and highlights the active strategy', () => {
        const initialValue: ModelMappingSelection = { strategy: 'system' };
        const onChange = vi.fn();

        render(
            <Model_Resolver_Select
                value={initialValue}
                onChange={onChange}
                templateOriginalModel="gemini-pro-latest"
            />
        );

        expect(screen.getByText('Use System Default')).toBeInTheDocument();
        expect(screen.getByText('Route to Local Ollama')).toBeInTheDocument();
        expect(screen.getByText('Keep Template Models')).toBeInTheDocument();
        expect(screen.getByText(/Target:/)).toHaveTextContent('ollama-cloud (gemma4:31b-cloud)');
    });

    it('triggers onChange with ollama config when Route to Local Ollama is clicked', () => {
        const initialValue: ModelMappingSelection = { strategy: 'system' };
        const onChange = vi.fn();

        render(
            <Model_Resolver_Select
                value={initialValue}
                onChange={onChange}
            />
        );

        const ollamaBtn = screen.getByText('Route to Local Ollama').closest('button');
        expect(ollamaBtn).toBeInTheDocument();
        fireEvent.click(ollamaBtn!);

        expect(onChange).toHaveBeenCalledWith({
            strategy: 'ollama',
            provider: 'ollama',
            modelId: 'gemma4:e4b',
            baseUrl: 'http://127.0.0.1:11434'
        });
    });

    it('triggers onChange with template strategy when Keep Template Models is clicked', () => {
        const initialValue: ModelMappingSelection = { strategy: 'system' };
        const onChange = vi.fn();

        render(
            <Model_Resolver_Select
                value={initialValue}
                onChange={onChange}
            />
        );

        const templateBtn = screen.getByText('Keep Template Models').closest('button');
        expect(templateBtn).toBeInTheDocument();
        fireEvent.click(templateBtn!);

        expect(onChange).toHaveBeenCalledWith({
            strategy: 'template'
        });
    });

    it('triggers onChange with system default when Use System Default is clicked', () => {
        const initialValue: ModelMappingSelection = { strategy: 'ollama' };
        const onChange = vi.fn();

        render(
            <Model_Resolver_Select
                value={initialValue}
                onChange={onChange}
            />
        );

        const systemBtn = screen.getByText('Use System Default').closest('button');
        expect(systemBtn).toBeInTheDocument();
        fireEvent.click(systemBtn!);

        expect(onChange).toHaveBeenCalledWith({
            strategy: 'system',
            provider: 'ollama-cloud',
            modelId: 'gemma4:31b-cloud'
        });
    });

    it('allows custom provider selection and model configuration', () => {
        const initialValue: ModelMappingSelection = { strategy: 'custom', provider: 'groq', modelId: 'llama-3.3-70b' };
        const onChange = vi.fn();

        render(
            <Model_Resolver_Select
                value={initialValue}
                onChange={onChange}
            />
        );

        const customSelect = screen.getByLabelText('Custom Provider') as HTMLSelectElement;
        expect(customSelect).toBeInTheDocument();

        fireEvent.change(customSelect, { target: { value: 'anthropic' } });
        expect(onChange).toHaveBeenCalledWith(expect.objectContaining({
            strategy: 'custom',
            provider: 'anthropic'
        }));

        const modelInput = screen.getByLabelText('Model Identifier') as HTMLInputElement;
        fireEvent.change(modelInput, { target: { value: 'claude-3-7-sonnet' } });
        expect(onChange).toHaveBeenCalledWith(expect.objectContaining({
            strategy: 'custom',
            modelId: 'claude-3-7-sonnet'
        }));
    });
});
