/**
 * @docs ARCHITECTURE:Testing
 *
 * ### AI Assist Note
 * Regression coverage for the adjacent production module and its public contracts.
 *
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Contract, rendering, state transition, or error-handling regression.
 * - **Trace Scope**: Vitest assertions and test-local mocks.
 */

import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { CognitionSection } from './CognitionSection';

describe('CognitionSection Component', () => {
    const mock_slots = {
        primary: { provider: 'openai', model: 'gpt-4o', system_prompt: 'You are an agent', temperature: 0.7, max_tokens: 4096, skills: [], workflows: [] },
        secondary: { provider: 'ollama', model: 'llama3', system_prompt: '', temperature: 0.7, max_tokens: 4096, skills: [], workflows: [] },
        tertiary: { provider: 'anthropic', model: 'claude-3-sonnet', system_prompt: '', temperature: 0.7, max_tokens: 4096, skills: [], workflows: [] }
    } as any;

    it('renders slot navigation tabs and handles tab changes', () => {
        const on_set_tab_mock = vi.fn();
        render(
            <CognitionSection
                activeTab="primary"
                slots={mock_slots}
                agentStatus="active"
                providers={[{ id: 'openai', name: 'OpenAI', base_url: '', is_active: true } as any]}
                models={[{ name: 'gpt-4o', provider: 'openai' } as any]}
                allSkills={['Audit']}
                allWorkflows={[]}
                manifests={[]}
                scripts={[]}
                mcpTools={[]}
                themeColor="#10b981"
                onSetTab={on_set_tab_mock}
                onUpdateSlotField={vi.fn()}
                onToggleSkill={vi.fn()}
                onProviderChange={vi.fn()}
                onPause={vi.fn()}
                onResume={vi.fn()}
            />
        );

        const buttons = screen.getAllByRole('button');
        expect(buttons.length).toBeGreaterThan(0);

        // Click secondary tab button (second button)
        fireEvent.click(buttons[1]);
        expect(on_set_tab_mock).toHaveBeenCalledWith('secondary');
    });
});
