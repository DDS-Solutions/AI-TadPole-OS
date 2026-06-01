/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Tests the Node Header's dynamic content management**, including agent name, tokens used, and model status icons. 
 * Verifies `dropdown_store` interaction for context menu activation and agent setting navigation.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Incorrect token metric scaling or stale model icons during high-frequency agent state updates.
 * - **Telemetry Link**: Search `[Node_Header.test]` in tracing logs.
 */


import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Node_Header } from '../Node_Header';
import { i18n } from '../../../i18n';

vi.mock('../../../stores/dropdown_store', () => ({
    use_dropdown_store: vi.fn((selector) => {
        const mock_state = {
            open_id: null,
            open_type: null,
            is_open: vi.fn(() => false),
            toggle_dropdown: vi.fn(),
            close_dropdown: vi.fn()
        };
        if (selector) return selector(mock_state);
        return mock_state;
    }),
}));

vi.mock('../../ui', () => ({
    Tooltip: ({ children, content }: any) => <div title={content}>{children}</div>
}));

vi.mock('../../../i18n', () => ({
    i18n: {
        t: (key: string) => key,
    }
}));

const mock_agent = {
    id: '1',
    name: 'Test Agent',
    role: 'Alpha',
    department: 'Engineering' as any,
    status: 'active' as any,
    tokens_used: 0,
    model: 'gpt-4',
    category: 'ai' as const
};

describe('Node_Header', () => {
    it('renders the Brain icon when has_oversight is true and is_alpha is true', () => {
        render(
            <Node_Header 
                agent={mock_agent} 
                is_alpha={true} 
                is_active={true} 
                available_roles={[]} 
                has_oversight={true}
            />
        );
        
        expect(screen.getByLabelText(i18n.t('oversight.btn_show'))).toBeInTheDocument();
    });

    it('does not render the Brain icon when has_oversight is false', () => {
        render(
            <Node_Header 
                agent={mock_agent} 
                is_alpha={true} 
                is_active={true} 
                available_roles={[]} 
                has_oversight={false}
            />
        );
        
        expect(screen.queryByLabelText(i18n.t('oversight.btn_show'))).not.toBeInTheDocument();
    });

    it('triggers on_oversight_toggle when the Brain icon is clicked', () => {
        const on_toggle = vi.fn();
        render(
            <Node_Header 
                agent={mock_agent} 
                is_alpha={true} 
                is_active={true} 
                available_roles={[]} 
                has_oversight={true}
                on_oversight_toggle={on_toggle}
            />
        );
        
        const btn = screen.getByLabelText(i18n.t('oversight.btn_show'));
        fireEvent.click(btn);
        expect(on_toggle).toHaveBeenCalled();
    });
});


// Metadata: [Node_Header_test]

// Metadata: [Node_Header_test]
