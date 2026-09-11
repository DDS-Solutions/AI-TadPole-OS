/**
 * @docs ARCHITECTURE:UI
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Components / Outward / A2A_Card_Preview.test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Component state and props flow adhere strictly to unidirectional UI data bindings.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import { render, screen } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { A2A_Card_Preview } from './A2A_Card_Preview';

describe('A2A_Card_Preview Component', () => {
  it('renders agent name and gemma4:e4b model profile', () => {
    render(
      <A2A_Card_Preview
        name="Acme Customer Agent"
        modelProfile="gemma4:e4b"
      />
    );

    expect(screen.getByText('Acme Customer Agent')).toBeInTheDocument();
    expect(screen.getByText('gemma4:e4b')).toBeInTheDocument();
  });

  it('renders exposed A2A skills', () => {
    const customSkills = [
      {
        id: 'order_status',
        name: 'Order Status Check',
        description: 'Lookup order by ID',
        tags: ['shipping', 'order'],
      },
    ];

    render(<A2A_Card_Preview skills={customSkills} />);

    expect(screen.getByText('Order Status Check')).toBeInTheDocument();
    expect(screen.getByText('ID: order_status')).toBeInTheDocument();
  });

  it('renders empty skills state when no skills are provided', () => {
    render(<A2A_Card_Preview skills={[]} />);

    expect(
      screen.getByText('No skills currently exposed in this agent card.')
    ).toBeInTheDocument();
  });
});
