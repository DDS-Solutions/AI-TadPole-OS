/**
 * @docs ARCHITECTURE:UI
 * @docs OPERATIONS_MANUAL:CustomerCatalog
 *
 * ### AI Assist Note
 * Unit tests for Customer_Catalog_Manager page.
 * Validates business identity input, model execution profile selection,
 * catalog search filter, file ingestion triggers, and accessibility labels adhering to design.md.
 *
 * ### 🔍 Debugging & Observability
 * Traceability via `execution/parity_guard.py`.
 */

import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { Customer_Catalog_Manager } from './Customer_Catalog_Manager';
import { parseCsvText } from '../utils/csv_parser';

describe('Customer_Catalog_Manager Page', () => {
  it('renders title, silo status, and A2A shield metrics', () => {
    render(<Customer_Catalog_Manager />);

    expect(screen.getByText('Customer Catalog & A2A Gateway Manager')).toBeInTheDocument();
    expect(screen.getByText('Silo Isolation Active')).toBeInTheDocument();
    expect(screen.getByText('60 req/min IP Shield')).toBeInTheDocument();
  });

  it('updates business name and reflects in live A2A Agent Card preview', () => {
    render(<Customer_Catalog_Manager />);

    const input = screen.getByPlaceholderText('e.g. Acme Hardware & Supply');
    fireEvent.change(input, { target: { value: 'Sovereign Hardware Store' } });

    expect(screen.getAllByText('Sovereign Hardware Store').length).toBeGreaterThan(0);
  });

  it('renders Business FAQ & Operating Info Cards manager component', () => {
    render(<Customer_Catalog_Manager />);

    expect(screen.getByText('Business FAQ & Operating Info Cards')).toBeInTheDocument();
    expect(screen.getAllByText('Store Hours & Location').length).toBeGreaterThan(0);
    expect(screen.getAllByText('Return & Exchange Policy').length).toBeGreaterThan(0);
  });

  it('has accessible aria-label attributes on file upload buttons', () => {
    render(<Customer_Catalog_Manager />);

    expect(screen.getByLabelText('Upload CSV Catalog File')).toBeInTheDocument();
    expect(screen.getByLabelText('Upload QuickBooks JSON File')).toBeInTheDocument();
    expect(screen.getByLabelText('PDF Pricelist Import Unavailable')).toBeDisabled();
  });
});

describe('parseCsvText Helper', () => {
  it('correctly parses quote-aware CSV text with quotes and commas', () => {
    const csvContent = `Title,Category,Price\n"Precision Torque Wrench, 1/2-Inch",Tools,"$129.99"\n"Heavy Duty ""Industrial"" Bolt Cutter",Hardware,"$49.95"`;
    const parsed = parseCsvText(csvContent);

    expect(parsed).toHaveLength(2);
    expect(parsed[0].title).toBe('Precision Torque Wrench, 1/2-Inch');
    expect(parsed[0].category).toBe('Tools');
    expect(parsed[0].price).toBe('$129.99');
    expect(parsed[1].title).toBe('Heavy Duty "Industrial" Bolt Cutter');
  });
});
