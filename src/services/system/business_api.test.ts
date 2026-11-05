/**
 * @docs ARCHITECTURE:TestSuites
 * ### AI Assist Note
 * Business API tests cover company-bound requests, approval versioning and evidence-driven verification.
 * ### 🔍 Debugging & Observability
 * Shared transport is mocked; no network or model requests occur.
 */

import { beforeEach, describe, expect, it, vi } from 'vitest';
import { api_request } from '../base_api_service';
import { business_api, business_report_filename, is_business_report_verified, type BusinessReport } from './business_api';

vi.mock('../base_api_service', () => ({ api_request: vi.fn() }));

describe('business_api', () => {
    beforeEach(() => vi.clearAllMocks());

    it('uses company-scoped authenticated transport and propagates cancellation', async () => {
        const controller = new AbortController();
        vi.mocked(api_request).mockResolvedValueOnce({ companies: [] }).mockResolvedValueOnce({ runs: [] });
        expect(await business_api.list_companies({ signal: controller.signal })).toEqual([]);
        expect(await business_api.list_runs('Mock-2-Service', { signal: controller.signal })).toEqual([]);
        expect(api_request).toHaveBeenLastCalledWith('/v1/business/companies/Mock-2-Service/runs', {
            method: 'GET', signal: controller.signal,
        });
    });

    it('preserves the exact idempotent create body for an explicit retry', async () => {
        const request = { idempotency_key: 'request-1', customer_reference: 'Mock-1-Product-Customer-001', offering_reference: 'Mock-1-Product-Offering-001', enquiry: 'What is the price?' };
        vi.mocked(api_request).mockRejectedValueOnce(new Error('Timeout')).mockResolvedValueOnce({ run_id: 'run-1' });
        await expect(business_api.create_run('Mock-1-Product', request)).rejects.toThrow('Timeout');
        await business_api.create_run('Mock-1-Product', request);
        expect(vi.mocked(api_request).mock.calls[0]).toEqual(vi.mocked(api_request).mock.calls[1]);
    });

    it('binds decisions to the company, run and expected version', async () => {
        vi.mocked(api_request).mockResolvedValue({});
        await business_api.decide_run('Mock-2-Service', 'run-1', 7, 'reject');
        expect(api_request).toHaveBeenCalledWith('/v1/business/companies/Mock-2-Service/runs/run-1/decision', {
            method: 'POST', body: JSON.stringify({ expected_version: 7, decision: 'reject' }),
        });
        expect(() => business_api.get_run('Mock-2-Service', '../other')).toThrow('Invalid company or run');
        expect(() => business_api.get_company('..')).toThrow('Invalid company or run');
    });

    it('does not accept a success flag without actual positive evidence', () => {
        const report = {
            run: { company_key: 'Mock-1-Product', run_id: 'run-1', status: 'completed', sop_id: 'Mock-1-Product-Customer-Enquiry-SOP', sop_version: '1', sop_digest: 'digest-1', approval: { decision: 'approve' }, ticket: {}, receipt: { company_key: 'Mock-1-Product', run_id: 'run-1' }, evidence: [{}] },
            verification: { verified: true, checks: [{ name: 'receipt', passed: true, detail: 'saved' }] },
            sop_evaluation: { sop_id: 'Mock-1-Product-Customer-Enquiry-SOP', sop_version: '1', sop_digest: 'digest-1', status: 'passed', steps: [{ status: 'passed' }] },
        } as BusinessReport;
        expect(is_business_report_verified(report)).toBe(true);
        expect(is_business_report_verified({ ...report, sop_evaluation: { ...report.sop_evaluation, sop_digest: 'different-procedure' } })).toBe(false);
        expect(is_business_report_verified({ ...report, sop_evaluation: { ...report.sop_evaluation, steps: [] } })).toBe(false);
        expect(is_business_report_verified({ ...report, verification: { verified: true, checks: [] } })).toBe(false);
        expect(is_business_report_verified({ ...report, run: { ...report.run, evidence: [] } })).toBe(false);
        expect(is_business_report_verified({ ...report, run: { ...report.run, receipt: { ...report.run.receipt!, company_key: 'Mock-2-Service' } } })).toBe(false);
        expect(business_report_filename(report)).toBe('Mock-1-Product-Run-run-1.json');
        expect(() => business_report_filename({ ...report, run: { ...report.run, run_id: '../../bad' } })).toThrow();
        expect(() => business_report_filename({ ...report, run: { ...report.run, company_key: 'Real-Company' } })).toThrow();
    });
});
