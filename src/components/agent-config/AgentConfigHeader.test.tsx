/**
 * @docs ARCHITECTURE:TestSuites
 * 
 * ### AI Assist Note
 * **Integrated verification of Agent Config Header component.**
 * Verifies rendering of name, role, department and interaction with settings modals.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Modal opening failure due to click handler or incorrect state.
 * - **Telemetry Link**: Search `[AgentConfigHeader_test]` in tracing logs.
 */

import '@testing-library/jest-dom';
import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { AgentConfigHeader } from './AgentConfigHeader';

// Mock tadpole_os_service
vi.mock('../../services/tadpoleos_service', () => ({
    tadpole_os_service: {
        save_role_blueprint: vi.fn().mockResolvedValue(true),
        delete_role_blueprint: vi.fn().mockResolvedValue(true),
    }
}));

// Mock event_bus
vi.mock('../../services/event_bus', () => ({
    event_bus: {
        emit_log: vi.fn(),
        subscribe_logs: vi.fn(() => () => {}),
        subscribe_traces: vi.fn(() => () => {}),
        get_history: vi.fn(() => []),
    }
}));

// Mock i18n
vi.mock('../../i18n', () => ({
    i18n: {
        t: (key: string) => key,
    },
}));

// Mock Zustand stores
const mock_roles = {
    executive: {
        id: 'executive',
        name: 'Executive',
        department: 'Operations',
        description: 'Executive role',
        skills: [],
        workflows: [],
    }
};

vi.mock('../../stores/role_store', () => ({
    use_role_store: Object.assign(
        (fn: any) => fn({ roles: mock_roles }),
        {
            getState: () => ({
                roles: mock_roles,
                add_role: vi.fn(),
                update_role: vi.fn(),
                delete_role: vi.fn()
            })
        }
    )
}));

const mock_dept_state = {
    departments: ['Executive', 'Engineering'],
    add_department: vi.fn(),
    edit_department: vi.fn(),
    delete_department: vi.fn(),
};

vi.mock('../../stores/department_store', () => ({
    use_department_store: () => mock_dept_state
}));

describe('AgentConfigHeader', () => {
    const mock_on_close = vi.fn();
    const mock_on_update_identity = vi.fn();
    const mock_on_update_theme_color = vi.fn();
    const mock_on_role_change = vi.fn();

    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('renders basic props correctly', () => {
        render(
            <AgentConfigHeader
                name="Test Agent"
                role="executive"
                department="Executive"
                themeColor="#10b981"
                isNew={false}
                availableRoles={['executive']}
                onClose={mock_on_close}
                onUpdateIdentity={mock_on_update_identity}
                onUpdateThemeColor={mock_on_update_theme_color}
                onRoleChange={mock_on_role_change}
            />
        );

        expect(screen.getByDisplayValue('Test Agent')).toBeInTheDocument();
        expect(screen.getAllByText('EXECUTIVE')[0]).toBeInTheDocument();
    });

    it('opens Manage Roles modal when clicking Manage Roles button', () => {
        render(
            <AgentConfigHeader
                name="Test Agent"
                role="executive"
                department="Executive"
                themeColor="#10b981"
                isNew={false}
                availableRoles={['executive']}
                onClose={mock_on_close}
                onUpdateIdentity={mock_on_update_identity}
                onUpdateThemeColor={mock_on_update_theme_color}
                onRoleChange={mock_on_role_change}
            />
        );

        const manageBtn = screen.getAllByRole('button').find(
            btn => btn.closest('.group\\/role')
        );
        expect(manageBtn).toBeInTheDocument();
        fireEvent.click(manageBtn!);

        expect(screen.getByText('Manage Swarm Roles')).toBeInTheDocument();
    });

    it('opens Manage Departments modal when clicking Manage Departments button', () => {
        render(
            <AgentConfigHeader
                name="Test Agent"
                role="executive"
                department="Executive"
                themeColor="#10b981"
                isNew={false}
                availableRoles={['executive']}
                onClose={mock_on_close}
                onUpdateIdentity={mock_on_update_identity}
                onUpdateThemeColor={mock_on_update_theme_color}
                onRoleChange={mock_on_role_change}
            />
        );

        const manageBtn = screen.getAllByRole('button').find(
            btn => btn.closest('.group\\/dept')
        );
        expect(manageBtn).toBeInTheDocument();
        fireEvent.click(manageBtn!);

        expect(screen.getByText('Manage Swarm Departments')).toBeInTheDocument();
    });
});

// Metadata: [AgentConfigHeader_test]
