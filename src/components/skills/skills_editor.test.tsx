/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Skills Editor Unit Tests**: Validates trigger form inputs, skill capability assignments,
 * and visual cards in the Agent Skills interface.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Invalid DOM element queries or form field state changes not firing triggers.
 * - **Telemetry Link**: Search `[skills_editor.test]` in tracing logs.
 */

import '@testing-library/jest-dom';
import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { Hook_Modal } from './Hook_Modal';
import { Assignment_Modal } from './Assignment_Modal';
import { Workflow_Card } from './Workflow_Card';
import { Mcp_Tool_Card } from './Mcp_Tool_Card';
import { Skill_Edit_Modal } from './Skill_Edit_Modal';
import { Workflow_Edit_Modal } from './Workflow_Edit_Modal';
import { Security_Report_Modal } from './Security_Report_Modal';

// Mock i18n
vi.mock('../../i18n', () => ({
    i18n: {
        t: (key: string, options?: any) => {
            if (options && options.name) return `${key}:${options.name}`;
            return key;
        },
    },
}));

describe('Skills Component Group', () => {
    describe('Hook_Modal', () => {
        const mock_props = {
            is_open: true,
            on_close: vi.fn(),
            editing_hook: {},
            hook_form: {
                name: 'test-hook',
                hook_type: 'pre_validation',
                description: 'test description',
                content: '',
                active: true,
                category: 'user' as const,
            },
            set_hook_form: vi.fn(),
            hook_save_error: null,
            is_saving: false,
            on_save: vi.fn(),
        };

        beforeEach(() => {
            vi.clearAllMocks();
        });

        it('renders and allows modifying inputs', () => {
            render(<Hook_Modal {...mock_props} />);

            expect(screen.getByPlaceholderText('chat.placeholder_hook_id')).toBeInTheDocument();
            
            const id_input = screen.getByPlaceholderText('chat.placeholder_hook_id');
            fireEvent.change(id_input, { target: { value: 'new-hook-id' } });
            expect(mock_props.set_hook_form).toHaveBeenCalledWith(expect.objectContaining({ name: 'new-hook-id' }));
        });

        it('triggers save action on submit', () => {
            render(<Hook_Modal {...mock_props} />);

            const save_btn = screen.getByText('chat.btn_initialize_hook');
            fireEvent.click(save_btn);

            expect(mock_props.on_save).toHaveBeenCalled();
        });
    });

    describe('Assignment_Modal', () => {
        const mock_props = {
            is_open: true,
            on_close: vi.fn(),
            assign_target: { type: 'skill' as const, name: 'git_add' },
            agents: [
                { id: 'agent-1', name: 'Agent Alpha', role: 'Developer', department: 'Engineering', status: 'idle' as const, model: 'gemini-2.0-flash', category: 'user', theme_color: '#10b981', skills: ['git_add'], workflows: [], mcp_tools: [] }
            ],
            on_toggle_assignment: vi.fn(),
        };

        it('renders agent list with status indicators', () => {
            render(<Assignment_Modal {...mock_props} />);

            expect(screen.getByText('Agent Alpha')).toBeInTheDocument();
            
            const agent_btn = screen.getByText('Agent Alpha').closest('button');
            expect(agent_btn).toBeInTheDocument();
            
            fireEvent.click(agent_btn!);
            expect(mock_props.on_toggle_assignment).toHaveBeenCalledWith('agent-1');
        });
    });

    describe('Workflow_Card', () => {
        const mock_workflow = {
            name: 'Deploy Pipeline',
            content: 'Step 1\nStep 2',
            category: 'user' as const,
        };
        const mock_on_edit = vi.fn();
        const mock_on_assign = vi.fn();
        const mock_on_delete = vi.fn();

        it('renders workflow information', () => {
            render(
                <Workflow_Card 
                    workflow={mock_workflow} 
                    on_edit={mock_on_edit}
                    on_assign={mock_on_assign} 
                    on_delete={mock_on_delete} 
                />
            );

            expect(screen.getByText('Deploy Pipeline')).toBeInTheDocument();
            expect(screen.getByText('Step 1')).toBeInTheDocument();

            const buttons = screen.getAllByRole('button');
            
            // Trigger edit button
            fireEvent.click(buttons[0]);
            expect(mock_on_edit).toHaveBeenCalledWith(mock_workflow);

            // Trigger assign button
            fireEvent.click(buttons[1]);
            expect(mock_on_assign).toHaveBeenCalledWith('Deploy Pipeline');

            // Trigger delete button
            fireEvent.click(buttons[2]);
            expect(mock_on_delete).toHaveBeenCalledWith('Deploy Pipeline');
        });
    });

    describe('Mcp_Tool_Card', () => {
        const mock_tool = {
            name: 'git_status',
            description: 'Gets local Git repositories status',
            source: 'git',
            stats: { invocations: 10, success_count: 9 }
        } as any;
        const mock_on_edit = vi.fn();

        it('renders mcp tool and metadata details', () => {
            render(
                <Mcp_Tool_Card 
                    tool={mock_tool} 
                    on_edit={mock_on_edit} 
                />
            );

            expect(screen.getByText('git_status')).toBeInTheDocument();
            expect(screen.getByText('Gets local Git repositories status')).toBeInTheDocument();

            const edit_btn = screen.getByRole('button');
            fireEvent.click(edit_btn);
            expect(mock_on_edit).toHaveBeenCalledWith(mock_tool);
        });
    });

    describe('Skill_Edit_Modal', () => {
        const mock_props = {
            is_open: true,
            on_close: vi.fn(),
            editing_skill: { name: 'test_skill', description: 'test desc', execution_command: 'test cmd', schema: {} },
            set_editing_skill: vi.fn(),
            schema_error: null,
            set_schema_error: vi.fn(),
            skill_save_error: null,
            is_saving: false,
            on_save: vi.fn(),
        };

        it('renders editing fields and calls save', () => {
            render(<Skill_Edit_Modal {...mock_props} />);

            expect(screen.getByPlaceholderText('chat.placeholder_skill_name')).toBeInTheDocument();
            const save_btn = screen.getByText('skills.btn_save_skill');
            fireEvent.click(save_btn);
            expect(mock_props.on_save).toHaveBeenCalled();
        });
    });

    describe('Workflow_Edit_Modal', () => {
        const mock_props = {
            is_open: true,
            on_close: vi.fn(),
            editing_wf: { name: 'test_wf', content: 'test content' },
            set_editing_wf: vi.fn(),
            wf_save_error: null,
            is_saving: false,
            on_save: vi.fn(),
        };

        it('renders editing fields and calls save', () => {
            render(<Workflow_Edit_Modal {...mock_props} />);

            expect(screen.getByPlaceholderText('chat.placeholder_protocol')).toBeInTheDocument();
            const save_btn = screen.getByText('skills.btn_save_workflow');
            fireEvent.click(save_btn);
            expect(mock_props.on_save).toHaveBeenCalled();
        });
    });

    describe('Security_Report_Modal', () => {
        const mock_skill = {
            name: 'mock_skill',
            security_score: 15,
            security_severity: 'LOW',
            security_report: {
                filtered_findings: [
                    { rule_id: 'R1', finding: 'Warning Message', explanation: 'My explanation', location: 'line 10', severity: 'HIGH' }
                ]
            }
        } as any;

        it('renders security score and findings list', () => {
            render(<Security_Report_Modal is_open={true} on_close={vi.fn()} skill={mock_skill} />);

            expect(screen.getByText('Warning Message')).toBeInTheDocument();
        });
    });
});

// Metadata: [skills_editor_test]
