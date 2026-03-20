import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { NodeHeader } from './NodeHeader';
import { i18n } from '../../i18n';

vi.mock('../../stores/dropdownStore', () => ({
    useDropdownStore: vi.fn((selector) => selector({ openId: null })),
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
    model: 'gpt-4'
};

describe('NodeHeader', () => {
    it('renders the Brain icon when hasOversight is true and isAlpha is true', () => {
        render(
            <NodeHeader 
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
            <NodeHeader 
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
            <NodeHeader 
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
