/**
 * @docs ARCHITECTURE:UI
 *
 * ### AI Context Alignment
 * - **Subsystem**: UI Pages / Customer_Catalog_Manager.test
 *
 * ### 🔍 Debugging & Observability
 * - **Telemetry Targets**: none declared
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

  it('scopes business identity, info cards, and catalog items when embedded with a practice company', () => {
    render(
      <Customer_Catalog_Manager
        companyKey="Mock-1-Product"
        embedded
        company={{
          company_key: 'Mock-1-Product',
          workspace_id: 'Mock-1-Product-Workspace',
          environment: 'mock',
          display_name: 'Mock-1-Product',
          company_kind: 'product',
          fixture_version: '2026-04-12.1',
          configuration_version: 'Mock-1-Product-Config-v1',
          workflow_definition_id: 'smb_customer_enquiry',
          workflow_version: '1',
          connector_mode: 'mock',
          agent_display_name: 'Mock-1-Product-Customer-Desk',
          workflow_display_name: 'Mock-1-Product-Customer-Enquiry',
          profile: {
            description: 'North Harbor Goods sells insulated stainless field bottles.',
            hours: 'Monday to Friday, 08:30-17:30 America/Toronto',
            support_email: 'support@mock-1-product.local',
            policy: 'Unused bottles may be returned within 30 days.',
          },
          offering: {
            id: 'offering-1',
            display_reference: 'Mock-1-Product-Offering-001',
            title: 'North Harbor 750ml Field Bottle',
            description: 'Double-wall stainless bottle.',
            price_minor: 3600,
            currency: 'USD',
            source_id: 'Mock-1-Product-Source-Offering-001',
            source_updated_at: '2026-04-12T12:00:00Z',
            fresh_until: '2099-01-01T00:00:00Z',
          },
          customer: {
            id: 'customer-1',
            display_reference: 'Mock-1-Product-Customer-001',
            name: 'Avery Chen',
          },
          sop: {
            id: 'Mock-1-Product-Customer-Enquiry-SOP',
            version: '1',
            title: 'Mock-1-Product Customer Enquiry SOP',
            document_reference: 'Mock-1-Product-SOP-001',
            document: '# SOP',
            roles: [],
            steps: [],
          },
        }}
      />
    );

    expect(screen.getByText('Scope: Mock-1-Product')).toBeInTheDocument();
    expect(screen.getAllByText(/North Harbor 750ml Field Bottle/i).length).toBeGreaterThan(0);
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
