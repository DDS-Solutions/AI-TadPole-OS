/**
 * @docs ARCHITECTURE:TestSuites
 * ### AI Assist Note
 * Operator workflow tests cover company isolation, stale responses, safe retries and evidence visibility.
 * ### 🔍 Debugging & Observability
 * Deferred mocked requests reproduce company-switch races without contacting the engine.
 */

import { act, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import SMB_Practice from './SMB_Practice';
import { ApiError } from '../api/errors';
import { business_api, type BusinessCompany, type BusinessReport, type BusinessRun } from '../services/system/business_api';

vi.mock('../services/system/business_api', async importOriginal => {
    const actual = await importOriginal<typeof import('../services/system/business_api')>();
    return { ...actual, business_api: {
        list_companies: vi.fn(), get_company: vi.fn(), renew_company_facts: vi.fn(), list_runs: vi.fn(), create_run: vi.fn(),
        get_run: vi.fn(), decide_run: vi.fn(), get_report: vi.fn(),
    } };
});

const company = (key: string, kind: string): BusinessCompany => ({
    company_key: key, workspace_id: `${key}-workspace`, environment: 'mock', display_name: key, company_kind: kind,
    fixture_version: 'fixture-1', configuration_version: '1', workflow_definition_id: 'customer_enquiry', workflow_version: '1',
    connector_mode: 'mock', agent_display_name: `${key}-Support-Agent`, workflow_display_name: `${key}-Customer-Enquiry`,
    profile: { description: `${key} company facts`, hours: '09:00–17:00', support_email: 'support@example.invalid', policy: 'Human approval is required.' },
    offering: { id: 'offering-id', display_reference: `${key}-Offering-001`, title: `${key}-Offering`, description: 'Fixture offering', price_minor: 4900, currency: 'USD', source_id: `${key}-Source-001`, source_updated_at: '2026-10-23T00:00:00Z', fresh_until: '2027-01-01T00:00:00Z' },
    customer: { id: 'customer-id', display_reference: `${key}-Customer-001`, name: `${key}-Customer` },
    sop: { id: `${key}-Customer-Enquiry-SOP`, version: '1', title: `${key}-Customer-Enquiry-SOP`, document_reference: `${key}-Customer-Enquiry-SOP-v1.md`, document: `${key} captured procedure`, roles: [{ name: `${key}-Owner`, execution: 'human_operator' }], steps: [] },
});
const product = company('Mock-1-Product', 'product');
const service = company('Mock-2-Service', 'service');
const makeRun = (owner = product, status: BusinessRun['status'] = 'pending_approval'): BusinessRun => ({
    run_id: 'run-1', display_reference: `${owner.company_key}-Run-001`, company_key: owner.company_key,
    workspace_id: owner.workspace_id, environment: 'mock', configuration_version: '1', fixture_version: 'fixture-1',
    workflow_definition_id: 'customer_enquiry', workflow_version: '1', connector_mode: 'mock', version: 3, status,
    sop_id: owner.sop.id, sop_version: owner.sop.version, sop_digest: 'saved-sop-digest',
    customer_reference: owner.customer.display_reference, offering_reference: owner.offering.display_reference,
    enquiry: 'What is the price?', draft_response: `${owner.company_key} response`,
    evidence: [{ source_id: owner.offering.source_id, source_updated_at: owner.offering.source_updated_at, fresh_until: owner.offering.fresh_until, content: 'Price is $49.00.' }],
    approval: null, ticket: null, receipt: null, created_at: '2026-10-23T00:00:00Z', updated_at: '2026-10-23T00:00:00Z', blocked_reason: null, latency_ms: 1,
});
const reportFor = (run: BusinessRun): BusinessReport => ({
    report_display_reference: `${run.company_key}-Report-001`, run, verification: { verified: false, checks: [{ name: 'ticket', passed: false, detail: 'Approval is pending.' }] }, latency_ms: 1,
    cost: { provider_charges_usd: 0, estimated_usd: 0, provenance: 'deterministic_local_no_model' },
    sop_evaluation: { sop_id: run.sop_id, sop_version: run.sop_version, sop_digest: run.sop_digest, status: 'pending', scope: 'local_workflow_and_human_approval', agent_execution_evaluated: false, semantic_response_quality_evaluated: false, steps: [{ step_id: `${run.company_key}-SOP-04`, title: 'Obtain the owner decision', owner_role: `${run.company_key}-Owner`, performed_by: 'not_performed', status: 'pending' }] },
});
const deferred = <T,>() => {
    let resolve!: (value: T) => void;
    const promise = new Promise<T>(done => { resolve = done; });
    return { promise, resolve };
};
const chooseService = () => fireEvent.change(screen.getByLabelText('Active practice company'), { target: { value: service.company_key } });

describe('SMB Practice', () => {
    beforeEach(() => {
        vi.resetAllMocks();
        vi.mocked(business_api.list_companies).mockResolvedValue([product, service]);
        vi.mocked(business_api.list_runs).mockResolvedValue([]);
        vi.mocked(business_api.get_run).mockResolvedValue(makeRun());
        vi.mocked(business_api.get_report).mockResolvedValue(reportFor(makeRun()));
    });

    it('shows both exact company names and truthful mock workflow labels', async () => {
        render(<SMB_Practice />);
        const selector = await screen.findByLabelText('Active practice company');
        expect(within(selector).getByRole('option', { name: 'Mock-1-Product' })).toBeInTheDocument();
        expect(within(selector).getByRole('option', { name: 'Mock-2-Service' })).toBeInTheDocument();
        expect(screen.getByText('Mock environment')).toBeInTheDocument();
        expect(screen.getByText(/Local fact-based workflow/)).toBeInTheDocument();
        expect(screen.getByText('Mock-1-Product-Offering')).toBeInTheDocument();
        expect(screen.getByText(/Current company SOP: Mock-1-Product-Customer-Enquiry-SOP/)).toBeInTheDocument();
        expect(screen.getByText(/Model-driven agent performance is not graded yet/)).toBeInTheDocument();
    });

    it('ignores a delayed old-company list and aborts its request after switching', async () => {
        const old = deferred<BusinessRun[]>();
        vi.mocked(business_api.list_runs).mockImplementation(key => key === product.company_key ? old.promise : Promise.resolve([]));
        render(<SMB_Practice />);
        await screen.findByLabelText('Active practice company');
        await waitFor(() => expect(business_api.list_runs).toHaveBeenCalledWith(product.company_key, expect.anything()));
        const old_signal = vi.mocked(business_api.list_runs).mock.calls[0][1]!.signal!;
        chooseService();
        await screen.findByRole('heading', { name: 'Mock-2-Service-Offering' });
        expect(old_signal.aborted).toBe(true);
        await act(async () => old.resolve([makeRun()]));
        expect(screen.queryByText('Mock-1-Product-Run-001')).not.toBeInTheDocument();
        expect(screen.getByRole('region', { name: 'Mock-2-Service practice workspace' })).toBeInTheDocument();
    });

    it('clears old approval context when switching while report loading', async () => {
        const delayed = deferred<BusinessReport>();
        vi.mocked(business_api.list_runs).mockImplementation(key => Promise.resolve(key === product.company_key ? [makeRun()] : []));
        vi.mocked(business_api.get_report).mockReturnValue(delayed.promise);
        render(<SMB_Practice />);
        fireEvent.click(await screen.findByRole('button', { name: /Mock-1-Product-Run-001/ }));
        await waitFor(() => expect(business_api.get_report).toHaveBeenCalled());
        chooseService();
        await act(async () => delayed.resolve(reportFor(makeRun())));
        expect(screen.queryByRole('button', { name: 'Approve response' })).not.toBeInTheDocument();
        expect(screen.queryByText('Mock-1-Product response')).not.toBeInTheDocument();
        expect(business_api.decide_run).not.toHaveBeenCalled();
    });

    it('sends the original approval company, run and version', async () => {
        const pending = makeRun();
        const rejected = { ...pending, version: 4, status: 'rejected' as const, approval: { decision: 'reject' as const, decided_at: pending.created_at } };
        vi.mocked(business_api.list_runs).mockResolvedValue([pending]);
        vi.mocked(business_api.decide_run).mockResolvedValue(rejected);
        render(<SMB_Practice />);
        fireEvent.click(await screen.findByRole('button', { name: /Mock-1-Product-Run-001/ }));
        const reject = await screen.findByRole('button', { name: 'Reject response' });
        expect(screen.getByText('Mock-1-Product-Owner · Not performed')).toBeInTheDocument();
        fireEvent.click(reject);
        await waitFor(() => expect(business_api.decide_run).toHaveBeenCalledWith('Mock-1-Product', 'run-1', 3, 'reject', expect.objectContaining({ signal: expect.any(AbortSignal) })));
    });

    it('retries an uncertain submission with identical body and key after company switches', async () => {
        vi.mocked(business_api.create_run).mockRejectedValueOnce(new Error('Request timed out')).mockResolvedValueOnce(makeRun());
        render(<SMB_Practice />);
        fireEvent.change(await screen.findByLabelText('Customer enquiry'), { target: { value: 'What is the price?' } });
        fireEvent.click(screen.getByRole('button', { name: 'Prepare response' }));
        await screen.findByText(/The outcome is uncertain/);
        expect(screen.getByLabelText('Customer enquiry')).toBeDisabled();
        chooseService();
        await screen.findByRole('heading', { name: 'Mock-2-Service-Offering' });
        fireEvent.change(screen.getByLabelText('Active practice company'), { target: { value: product.company_key } });
        fireEvent.click(await screen.findByRole('button', { name: 'Retry saved enquiry' }));
        await waitFor(() => expect(business_api.create_run).toHaveBeenCalledTimes(2));
        const calls = vi.mocked(business_api.create_run).mock.calls;
        expect(calls[0][0]).toBe(product.company_key);
        expect(calls[1][0]).toBe(product.company_key);
        expect(calls[1][1]).toEqual(calls[0][1]);
        expect(calls[0][1].idempotency_key).toBeTruthy();
    });

    it('does not label a completed ticket verified when source evidence is missing', async () => {
        const incomplete = { ...makeRun(product, 'completed'), evidence: [], ticket: { id: 'ticket-1', display_reference: 'Mock-1-Product-Ticket-001', response: 'Saved reply' } };
        vi.mocked(business_api.list_runs).mockResolvedValue([incomplete]);
        vi.mocked(business_api.get_run).mockResolvedValue(incomplete);
        vi.mocked(business_api.get_report).mockResolvedValue({ ...reportFor(incomplete), verification: { verified: true, checks: [] } });
        render(<SMB_Practice />);
        fireEvent.click(await screen.findByRole('button', { name: /Mock-1-Product-Run-001/ }));
        expect(await screen.findByText('Outcome not verified')).toBeInTheDocument();
        expect(screen.queryByText('Verified outcome')).not.toBeInTheDocument();
        expect(screen.getByText('No source evidence is saved.')).toBeInTheDocument();
    });

    it('unlocks rejected validation input so it can be corrected with a new request key', async () => {
        vi.mocked(business_api.create_run).mockRejectedValueOnce(new ApiError('Invalid enquiry', 'validation', 422)).mockResolvedValueOnce(makeRun());
        render(<SMB_Practice />);
        fireEvent.change(await screen.findByLabelText('Customer enquiry'), { target: { value: 'Invalid request' } });
        fireEvent.click(screen.getByRole('button', { name: 'Prepare response' }));
        await screen.findByText(/The request was rejected/);
        expect(screen.getByLabelText('Customer enquiry')).toBeEnabled();
        fireEvent.change(screen.getByLabelText('Customer enquiry'), { target: { value: 'What is the price?' } });
        fireEvent.click(screen.getByRole('button', { name: 'Prepare response' }));
        await waitFor(() => expect(business_api.create_run).toHaveBeenCalledTimes(2));
        const calls = vi.mocked(business_api.create_run).mock.calls;
        expect(calls[1][1].enquiry).toBe('What is the price?');
        expect(calls[1][1].idempotency_key).not.toBe(calls[0][1].idempotency_key);
    });

    it('keeps a newly created run when an older list request finishes later', async () => {
        const old_list = deferred<BusinessRun[]>();
        vi.mocked(business_api.list_runs).mockReturnValue(old_list.promise);
        vi.mocked(business_api.create_run).mockResolvedValue(makeRun());
        render(<SMB_Practice />);
        fireEvent.change(await screen.findByLabelText('Customer enquiry'), { target: { value: 'What is the price?' } });
        fireEvent.click(screen.getByRole('button', { name: 'Prepare response' }));
        await screen.findByRole('button', { name: 'Approve response' });
        await act(async () => old_list.resolve([]));
        expect(screen.getByRole('button', { name: /Mock-1-Product-Run-001/ })).toBeInTheDocument();
    });

    it('summarizes saved historical source facts instead of replacing them with current facts', async () => {
        const historical = makeRun();
        historical.evidence[0].content = JSON.stringify({ ...product, profile: { ...product.profile, policy: 'Historical return policy: 14 days.' }, offering: { ...product.offering, price_minor: 3900 } });
        vi.mocked(business_api.list_runs).mockResolvedValue([historical]);
        vi.mocked(business_api.get_run).mockResolvedValue(historical);
        vi.mocked(business_api.get_report).mockResolvedValue(reportFor(historical));
        render(<SMB_Practice />);
        fireEvent.click(await screen.findByRole('button', { name: /Mock-1-Product-Run-001/ }));
        expect(await screen.findByText('Historical return policy: 14 days.')).toBeInTheDocument();
        expect(screen.getByText('$39.00')).toBeInTheDocument();
        expect(screen.getByText('$49.00')).toBeInTheDocument();
        expect(screen.getByText('Full captured source record')).toBeInTheDocument();
    });

    it('allows discarding a saved retry to unlock the enquiry input', async () => {
        vi.mocked(business_api.create_run).mockRejectedValueOnce(new Error('Network dropped'));
        render(<SMB_Practice />);
        const input = await screen.findByLabelText('Customer enquiry');
        fireEvent.change(input, { target: { value: 'Stuck enquiry' } });
        fireEvent.click(screen.getByRole('button', { name: 'Prepare response' }));
        await screen.findByText(/The outcome is uncertain/);
        expect(input).toBeDisabled();
        fireEvent.click(screen.getByRole('button', { name: 'Discard saved retry' }));
        expect(input).toBeEnabled();
        expect(input).toHaveValue('');
    });

    it('warns when customer enquiry contains sensitive PII patterns', async () => {
        render(<SMB_Practice />);
        const input = await screen.findByLabelText('Customer enquiry');
        fireEvent.change(input, { target: { value: 'My SSN is 123-45-6789, please check my order.' } });
        expect(await screen.findByText(/PII Warning Detected/i)).toBeInTheDocument();
        expect(screen.getByText(/Social Security Number/i)).toBeInTheDocument();
    });

    it('supports operator response edits, decision notes, and unified workspace tabs', async () => {
        const pending = makeRun();
        const completed = {
            ...pending,
            version: 4,
            status: 'completed' as const,
            draft_response: 'Edited Mock-1-Product response with shipping note',
            approval: { decision: 'approve' as const, decided_at: pending.created_at, operator_note: 'Verified policy and added shipping note', operator_edited: true },
            ticket: { id: 'ticket-1', display_reference: 'Mock-1-Product-Ticket-001', response: 'Edited Mock-1-Product response with shipping note' },
            receipt: { id: 'receipt-1', display_reference: 'Mock-1-Product-Receipt-001', operation: 'record_customer_enquiry', status: 'completed', external_record_reference: 'Mock-1-Product-Ticket-001', evidence_location: 'business://ws/tickets/ticket-1', company_key: 'Mock-1-Product', run_id: 'run-1' },
        };
        vi.mocked(business_api.list_runs).mockResolvedValue([pending]);
        vi.mocked(business_api.decide_run).mockResolvedValue(completed);
        vi.mocked(business_api.get_run).mockResolvedValueOnce(pending).mockResolvedValue(completed);
        vi.mocked(business_api.get_report).mockResolvedValueOnce(reportFor(pending)).mockResolvedValue({
            ...reportFor(completed),
            verification: { verified: true, checks: [{ name: 'completed', passed: true, detail: 'Verified' }] },
            sop_evaluation: { ...reportFor(completed).sop_evaluation, status: 'passed', steps: [{ step_id: 'Mock-1-Product-SOP-04', title: 'Obtain the owner decision', owner_role: 'Mock-1-Product-Owner', performed_by: 'human_operator', status: 'passed' }] },
        });

        render(<SMB_Practice />);
        fireEvent.click(await screen.findByRole('button', { name: /Mock-1-Product-Run-001/ }));
        const editor = await screen.findByLabelText('Editable prepared response');
        fireEvent.change(editor, { target: { value: 'Edited Mock-1-Product response with shipping note' } });
        const noteInput = screen.getByLabelText('Owner decision note');
        fireEvent.change(noteInput, { target: { value: 'Verified policy and added shipping note' } });
        fireEvent.click(screen.getByRole('button', { name: 'Approve response' }));

        await waitFor(() => expect(business_api.decide_run).toHaveBeenCalledWith(
            'Mock-1-Product',
            'run-1',
            3,
            'approve',
            expect.objectContaining({
                edited_response: 'Edited Mock-1-Product response with shipping note',
                operator_note: 'Verified policy and added shipping note',
            })
        ));
        expect(await screen.findByText('Verified outcome')).toBeInTheDocument();

        // Switch to SOP & Live Readiness tab and Catalog & A2A Knowledge tab
        fireEvent.click(screen.getByRole('tab', { name: /SOP & Live Readiness/i }));
        expect(await screen.findByText(/Workspace Promotion Gate/i)).toBeInTheDocument();

        fireEvent.click(screen.getByRole('tab', { name: /Catalog & A2A Knowledge/i }));
        expect(await screen.findByText(/Business FAQ & Operating Info Cards/i)).toBeInTheDocument();
    });

    it('renders promotion readiness checks, renews fact freshness, and supports multiple catalog offerings', async () => {
        const extraOffering = {
            ...product.offering,
            id: 'offering-2',
            display_reference: 'Mock-1-Product-Offering-Cat-002',
            title: 'Mock-1-Product-Insulated Camp Mug',
            price_minor: 2800,
        };
        const productWithContext: BusinessCompany = {
            ...product,
            catalog_offerings: [product.offering, extraOffering],
            readiness: {
                current_stage: 'mock',
                next_stage: 'read_only',
                verified_runs: 1,
                completed_runs: 1,
                blocked_or_failed_runs: 0,
                facts_fresh: true,
                pii_guard_active: true,
                outbox_atomic_ready: true,
                eligible_for_promotion: true,
                checks: [
                    { id: 'company_silo_isolation', label: 'Company Silo & Prefix Isolation', passed: true, detail: 'Isolated workspace enforced.' },
                    { id: 'pii_fail_closed_guard', label: 'Server-Side PII Fail-Closed Guard (FR-11)', passed: true, detail: 'Active' },
                ],
            },
        };
        vi.mocked(business_api.list_companies).mockResolvedValue([productWithContext, service]);
        vi.mocked(business_api.renew_company_facts).mockResolvedValue({
            ...productWithContext,
            offering: { ...productWithContext.offering, fresh_until: '2027-06-01T00:00:00Z' },
        });

        render(<SMB_Practice />);
        const offeringSelect = await screen.findByLabelText('Active catalog offering');
        fireEvent.change(offeringSelect, { target: { value: 'Mock-1-Product-Offering-Cat-002' } });
        expect(screen.getByText('$28.00')).toBeInTheDocument();

        fireEvent.click(screen.getByRole('tab', { name: /SOP & Live Readiness/i }));
        expect(await screen.findByText(/Eligible for Stage 2 \(read_only\) Promotion/i)).toBeInTheDocument();
        expect(screen.getByText('Company Silo & Prefix Isolation')).toBeInTheDocument();

        fireEvent.click(screen.getByRole('button', { name: /Renew Fact Freshness/i }));
        await waitFor(() => expect(business_api.renew_company_facts).toHaveBeenCalledWith('Mock-1-Product', expect.any(Object)));
    });
});
