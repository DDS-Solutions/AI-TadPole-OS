import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Node_Header } from './Node_Header';
import { i18n } from '../../i18n';

vi.mock('../../stores/dropdown_store', () => ({
    use_dropdown_store: vi.fn((selector) => selector({ openId: null })),
}));

vi.mock('../ui', () => ({
    Tooltip: ({ children, content }: any) => <div title={content}>{children}</div>
}));

const mockAgent = {
    id: '1',
    name: 'Test Agent',
    role: 'Alpha',
    department: 'Engineering' as any,
    status: 'active' as any,
    tokensUsed: 0,
    model: 'gpt-4',
    category: 'ai' as const
};

describe('Node_Header', () => {
    it('renders the Brain icon when hasOversight is true and isAlpha is true', () => {
        render(
            <Node_Header 
                agent={mockAgent} 
                isAlpha={true} 
                isActive={true} 
                availableRoles={[]} 
                hasOversight={true}
            />
        );
        
        expect(screen.getByLabelText(i18n.t('oversight.btn_show'))).toBeInTheDocument();
    });

    it('does not render the Brain icon when hasOversight is false', () => {
        render(
            <Node_Header 
                agent={mockAgent} 
                isAlpha={true} 
                isActive={true} 
                availableRoles={[]} 
                hasOversight={false}
            />
        );
        
        expect(screen.queryByLabelText(i18n.t('oversight.btn_show'))).not.toBeInTheDocument();
    });

    it('triggers onOversightToggle when the Brain icon is clicked', () => {
        const onToggle = vi.fn();
        render(
            <Node_Header 
                agent={mockAgent} 
                isAlpha={true} 
                isActive={true} 
                availableRoles={[]} 
                hasOversight={true}
                onOversightToggle={onToggle}
            />
        );
        
        const btn = screen.getByLabelText(i18n.t('oversight.btn_show'));
        fireEvent.click(btn);
        expect(onToggle).toHaveBeenCalled();
    });
});
