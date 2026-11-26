/**
 * @docs ARCHITECTURE:Testing
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Missions / Cluster Manager / Preset_Form.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import React from 'react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Preset_Form } from './Preset_Form';
import { use_department_store } from '../../../stores/department_store';
import { INITIAL_FORM_STATE } from './types';

describe('Preset_Form Component', () => {
    const mock_update_field = vi.fn();
    const mock_reset_form = vi.fn();
    const mock_handle_save = vi.fn();
    const mock_set_agent_search_query = vi.fn();
    const mock_toggle_agent = vi.fn();
    const mock_handle_agent_hover = vi.fn();
    const mock_handle_agent_leave = vi.fn();
    const form_ref = React.createRef<HTMLDivElement>();

    beforeEach(() => {
        vi.clearAllMocks();
        use_department_store.setState({
            departments: ['Engineering', 'Executive', 'Operations', 'Swarm Core']
        });
    });

    it('populates department dropdown with dynamic departments from use_department_store including Swarm Core', () => {
        render(
            <Preset_Form
                form_ref={form_ref}
                form_data={{ ...INITIAL_FORM_STATE, department: 'Engineering' }}
                update_field={mock_update_field}
                reset_form={mock_reset_form}
                handle_save={mock_handle_save}
                is_form_valid={true}
                agent_search_query=""
                set_agent_search_query={mock_set_agent_search_query}
                filtered_selectable_agents={[]}
                toggle_agent={mock_toggle_agent}
                handle_agent_hover={mock_handle_agent_hover}
                handle_agent_leave={mock_handle_agent_leave}
            />
        );

        const select = screen.getByDisplayValue('Engineering');
        expect(select).toBeInTheDocument();

        expect(screen.getByRole('option', { name: 'Engineering' })).toBeInTheDocument();
        expect(screen.getByRole('option', { name: 'Executive' })).toBeInTheDocument();
        expect(screen.getByRole('option', { name: 'Operations' })).toBeInTheDocument();
        expect(screen.getByRole('option', { name: 'Swarm Core' })).toBeInTheDocument();
    });

    it('invokes update_field when selecting Swarm Core as department', () => {
        render(
            <Preset_Form
                form_ref={form_ref}
                form_data={{ ...INITIAL_FORM_STATE, department: 'Engineering' }}
                update_field={mock_update_field}
                reset_form={mock_reset_form}
                handle_save={mock_handle_save}
                is_form_valid={true}
                agent_search_query=""
                set_agent_search_query={mock_set_agent_search_query}
                filtered_selectable_agents={[]}
                toggle_agent={mock_toggle_agent}
                handle_agent_hover={mock_handle_agent_hover}
                handle_agent_leave={mock_handle_agent_leave}
            />
        );

        const select = screen.getByDisplayValue('Engineering');
        fireEvent.change(select, { target: { value: 'Swarm Core' } });

        expect(mock_update_field).toHaveBeenCalledWith('department', 'Swarm Core');
    });
});
