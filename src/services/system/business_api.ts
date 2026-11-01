/**
 * @docs ARCHITECTURE:UI-Services
 * ### AI Assist Note
 * Company-bound business workflow contracts mirror agent/business/types.rs.
 * ### 🔍 Debugging & Observability
 * Errors propagate to SMB Practice; requests use the shared authenticated API transport.
 */

import { api_request } from '../base_api_service';

export interface BusinessSop {
    id: string;
    version: string;
    title: string;
    document_reference: string;
    document: string;
    roles: Array<{ name: string; execution: 'local_rules' | 'human_operator' }>;
    steps: Array<{ id: string; title: string; owner_role: string; check: string }>;
}

export interface BusinessCompany {
    company_key: string;
    workspace_id: string;
    environment: string;
    display_name: string;
    company_kind: string;
    fixture_version: string;
    configuration_version: string;
    workflow_definition_id: string;
    workflow_version: string;
    connector_mode: string;
    agent_display_name: string;
    workflow_display_name: string;
    profile: { description: string; hours: string; support_email: string; policy: string };
    offering: {
        id: string;
        display_reference: string;
        title: string;
        description: string;
        price_minor: number;
        currency: string;
        source_id: string;
        source_updated_at: string;
        fresh_until: string;
    };
    customer: { id: string; display_reference: string; name: string };
    sop: BusinessSop;
}

export interface BusinessCreateRun {
    idempotency_key: string;
    customer_reference: string;
    offering_reference: string;
    enquiry: string;
}

export type BusinessDecision = 'approve' | 'reject';
export type BusinessRunStatus = 'pending_approval' | 'completed' | 'rejected' | 'blocked' | 'failed';

export interface BusinessRun {
    run_id: string;
    display_reference: string;
    company_key: string;
    workspace_id: string;
    environment: string;
    configuration_version: string;
    fixture_version: string;
    workflow_definition_id: string;
    workflow_version: string;
    sop_id: string;
    sop_version: string;
    sop_digest: string;
    connector_mode: string;
    version: number;
    status: BusinessRunStatus;
    customer_reference: string;
    offering_reference: string;
    enquiry: string;
    draft_response: string;
    evidence: Array<{ source_id: string; source_updated_at: string; fresh_until: string; content: string }>;
    approval: { decision: BusinessDecision; decided_at: string } | null;
    ticket: { id: string; display_reference: string; response: string } | null;
    receipt: {
        id: string;
        display_reference: string;
        operation: string;
        status: string;
        external_record_reference: string;
        evidence_location: string;
        company_key: string;
        run_id: string;
    } | null;
    created_at: string;
    updated_at: string;
    blocked_reason: string | null;
    latency_ms: number;
}

export interface BusinessReport {
    report_display_reference: string;
    run: BusinessRun;
    verification: { verified: boolean; checks: Array<{ name: string; passed: boolean; detail: string }> };
    sop_evaluation: {
        sop_id: string; sop_version: string; sop_digest: string;
        status: 'passed' | 'failed' | 'pending' | 'blocked';
        scope: string; agent_execution_evaluated: boolean; semantic_response_quality_evaluated: boolean;
        steps: Array<{ step_id: string; title: string; owner_role: string; performed_by: string; status: string }>;
    };
    latency_ms: number;
    cost: { provider_charges_usd: number; estimated_usd: number; provenance: string };
}

interface BusinessRequestOptions {
    signal?: AbortSignal;
    timeout?: number;
}

const path_segment = (value: string): string => {
    if (!/^[a-zA-Z0-9_.-]{1,128}$/.test(value) || value === '.' || value === '..') {
        throw new Error('Invalid company or run identifier.');
    }
    return encodeURIComponent(value);
};

const company_path = (company_key: string): string => `/v1/business/companies/${path_segment(company_key)}`;
const run_path = (company_key: string, run_id: string): string => `${company_path(company_key)}/runs/${path_segment(run_id)}`;

export const business_api = {
    async list_companies(options?: BusinessRequestOptions): Promise<BusinessCompany[]> {
        const result = await api_request<{ companies: BusinessCompany[] }>('/v1/business/companies', { method: 'GET', ...options });
        return result.companies;
    },
    get_company(company_key: string, options?: BusinessRequestOptions): Promise<BusinessCompany> {
        return api_request<BusinessCompany>(company_path(company_key), { method: 'GET', ...options });
    },
    async list_runs(company_key: string, options?: BusinessRequestOptions): Promise<BusinessRun[]> {
        const result = await api_request<{ runs: BusinessRun[] }>(`${company_path(company_key)}/runs`, { method: 'GET', ...options });
        return result.runs;
    },
    create_run(company_key: string, request: BusinessCreateRun, options?: BusinessRequestOptions): Promise<BusinessRun> {
        return api_request<BusinessRun>(`${company_path(company_key)}/runs`, {
            method: 'POST', body: JSON.stringify(request), ...options,
        });
    },
    get_run(company_key: string, run_id: string, options?: BusinessRequestOptions): Promise<BusinessRun> {
        return api_request<BusinessRun>(run_path(company_key, run_id), { method: 'GET', ...options });
    },
    decide_run(company_key: string, run_id: string, expected_version: number, decision: BusinessDecision, options?: BusinessRequestOptions): Promise<BusinessRun> {
        return api_request<BusinessRun>(`${run_path(company_key, run_id)}/decision`, {
            method: 'POST', body: JSON.stringify({ expected_version, decision }), ...options,
        });
    },
    get_report(company_key: string, run_id: string, options?: BusinessRequestOptions): Promise<BusinessReport> {
        return api_request<BusinessReport>(`${run_path(company_key, run_id)}/report`, { method: 'GET', ...options });
    },
};

/** Require positive, matching evidence rather than treating a saved ticket as success. */
export function is_business_report_verified(report: BusinessReport): boolean {
    return report.verification.verified && report.verification.checks.length > 0 &&
        report.sop_evaluation?.status === 'passed' && report.sop_evaluation.sop_id === report.run.sop_id &&
        report.sop_evaluation.sop_version === report.run.sop_version && report.sop_evaluation.sop_digest === report.run.sop_digest &&
        report.sop_evaluation.steps.length > 0 && report.sop_evaluation.steps.every(step => step.status === 'passed') &&
        report.verification.checks.every(check => check.passed) && report.run.status === 'completed' &&
        report.run.approval?.decision === 'approve' && !!report.run.ticket && !!report.run.receipt &&
        report.run.evidence.length > 0 && report.run.receipt.company_key === report.run.company_key &&
        report.run.receipt.run_id === report.run.run_id;
}

/** Use an exact known company prefix and an opaque run ID, never an arbitrary download path. */
export function business_report_filename(report: BusinessReport): string {
    if (!['Mock-1-Product', 'Mock-2-Service'].includes(report.run.company_key)) {
        throw new Error('Only practice-company reports may be exported here.');
    }
    return `${report.run.company_key}-Run-${path_segment(report.run.run_id)}.json`;
}
