/**
 * @docs ARCHITECTURE:Testing
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Oversight / Params_Cell.test
 * - **Primary Entrypoints**: `Params_Cell.test`
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 */

import { describe, it, expect } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Params_Cell } from './Params_Cell';

describe('Params_Cell Component', () => {
    it('renders empty parameters when params is undefined', () => {
        render(<Params_Cell params={undefined} />);
        expect(screen.getByRole('button')).toBeDefined();
        expect(screen.getByText('{}')).toBeDefined();
    });

    it('renders object parameters as inline JSON and toggles popover on click', () => {
        render(<Params_Cell params={{ key: 'value', num: 42 }} />);
        const button = screen.getByRole('button');
        expect(button).toBeDefined();
        expect(screen.getByText(/key.*value/)).toBeDefined();

        fireEvent.click(button);
        expect(screen.getByText('Params')).toBeDefined();
        expect(screen.getByText('PINNED')).toBeDefined();
    });

    it('renders string parameters safely without crash', () => {
        render(<Params_Cell params="raw string parameter" />);
        expect(screen.getByText('raw string parameter')).toBeDefined();
    });
});
