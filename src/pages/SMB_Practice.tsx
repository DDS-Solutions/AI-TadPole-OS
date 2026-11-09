/**
 * @docs ARCHITECTURE:Interface
 * ### AI Assist Note
 * Practice companies share a local fact-based workflow; company switches remount the scoped operator view.
 * ### 🔍 Debugging & Observability
 * Server evidence drives approval and verification; stale responses are rejected by company and request identity.
 */

import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { CheckCircle2, Download, FlaskConical, RefreshCw, ShieldCheck, Sparkles, Store } from 'lucide-react';
import { ApiError } from '../api/errors';
import {
    business_api, business_report_filename, is_business_report_verified,
    type BusinessCompany, type BusinessCreateRun, type BusinessDecision, type BusinessReport,
    type BusinessRun, type BusinessRunStatus,
} from '../services/system/business_api';
import { scanForPii } from '../utils/agent_card_compiler';
import { Customer_Catalog_Manager } from './Customer_Catalog_Manager';

type WorkspaceTab = 'enquiries' | 'catalog' | 'readiness';

const MOCK_COMPANIES = ['Mock-1-Product', 'Mock-2-Service'];
const panel = 'rounded-xl border border-[color:var(--color-border)] bg-[color:var(--color-surface)]/40 p-5 space-y-4';
const button = 'inline-flex items-center justify-center gap-2 rounded-lg border border-zinc-600 px-3 py-2 text-sm text-zinc-100 hover:bg-zinc-800 focus-visible:outline-2 focus-visible:outline-cyan-400 disabled:opacity-50 disabled:cursor-not-allowed';
const status_labels: Record<BusinessRunStatus, string> = {
    pending_approval: 'Awaiting your approval', completed: 'Recorded — verification required',
    rejected: 'Rejected', blocked: 'Blocked', failed: 'Failed',
};
const error_message = (error: unknown): string => error instanceof Error ? error.message : 'The request could not be completed.';
const rejected_request = (error: unknown): boolean => error instanceof ApiError && [400, 401, 403, 404, 413, 415, 422].includes(error.status);

export default function SMB_Practice() {
    const [companies, setCompanies] = useState<BusinessCompany[]>([]);
    const [company_key, setCompanyKey] = useState('Mock-1-Product');
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [reload, setReload] = useState(0);
    const [pending, setPending] = useState<Record<string, BusinessCreateRun | undefined>>({});

    useEffect(() => {
        const controller = new AbortController();
        void business_api.list_companies({ signal: controller.signal }).then(result => {
            if (controller.signal.aborted) return;
            if (result.length !== MOCK_COMPANIES.length || !MOCK_COMPANIES.every(key =>
                result.some(company => company.company_key === key && company.environment === 'mock' && company.connector_mode === 'mock')
            )) throw new Error('The engine did not return the two isolated practice companies.');
            setCompanies(result);
        }).catch(reason => {
            if (!controller.signal.aborted) setError(error_message(reason));
        }).finally(() => {
            if (!controller.signal.aborted) setLoading(false);
        });
        return () => controller.abort();
    }, [reload]);

    const selected = companies.find(company => company.company_key === company_key);
    const scope = selected ? `${selected.company_key}:${selected.workspace_id}` : '';
    const updatePending = useCallback((key: string, request?: BusinessCreateRun) => {
        setPending(current => ({ ...current, [key]: request }));
    }, []);
    const updateCompany = useCallback((updated: BusinessCompany) => {
        setCompanies(current => current.map(item => item.company_key === updated.company_key ? updated : item));
    }, []);

    return (
        <div data-testid="route-smb-practice" className="mx-auto max-w-7xl space-y-6 p-6 text-zinc-200">
            <header className="flex flex-wrap items-start justify-between gap-4 border-b border-zinc-800 pb-5">
                <div>
                    <h1 className="flex items-center gap-2 text-2xl font-semibold"><FlaskConical className="text-cyan-400" aria-hidden="true" />SMB Practice</h1>
                    <p className="mt-2 text-sm text-zinc-400">Try the same customer enquiry workflow with a product company and a service company.</p>
                    <p className="mt-1 text-sm text-zinc-400">Local fact-based workflow. No model calls or live customer messages.</p>
                </div>
                <span className="rounded-full border border-amber-400/40 bg-amber-400/10 px-3 py-1 text-sm text-amber-200">Mock environment</span>
            </header>
            {loading && <p role="status">Loading practice companies…</p>}
            {error && <div role="alert" className="space-y-3 rounded-lg border border-red-500/40 p-4 text-red-200"><p>{error}</p><button className={button} onClick={() => { setLoading(true); setError(null); setReload(value => value + 1); }}>Try again</button></div>}
            {!loading && !error && selected && <>
                <div className="flex flex-wrap items-end gap-4">
                    <label className="space-y-2 text-sm"><span className="block font-medium">Active practice company</span>
                        <select value={company_key} onChange={event => setCompanyKey(event.target.value)} className="min-w-60 rounded-lg border border-zinc-600 bg-zinc-900 px-3 py-2 text-zinc-100 focus-visible:outline-2 focus-visible:outline-cyan-400">
                            {MOCK_COMPANIES.map(key => <option key={key} value={key}>{key}</option>)}
                        </select>
                    </label>
                    <p className="pb-2 text-sm text-zinc-400">Switching companies keeps existing runs with their original company.</p>
                </div>
                <CompanyPractice key={scope} company={selected} pending={pending[scope]} onPending={request => updatePending(scope, request)} onCompanyUpdated={updateCompany} />
            </>}
        </div>
    );
}

function CompanyPractice({ company, pending, onPending, onCompanyUpdated }: {
    company: BusinessCompany;
    pending?: BusinessCreateRun;
    onPending: (request?: BusinessCreateRun) => void;
    onCompanyUpdated?: (updated: BusinessCompany) => void;
}) {
    const [active_tab, setActiveTab] = useState<WorkspaceTab>('enquiries');
    const [enquiry, setEnquiry] = useState(pending?.enquiry ?? '');
    const [selected_offering_ref, setSelectedOfferingRef] = useState(pending?.offering_reference ?? company.offering.display_reference);
    const [runs, setRuns] = useState<BusinessRun[]>([]);
    const [run, setRun] = useState<BusinessRun | null>(null);
    const [report, setReport] = useState<BusinessReport | null>(null);
    const [verified_runs, setVerifiedRuns] = useState<Record<string, boolean>>({});
    const [status_filter, setStatusFilter] = useState<'all' | BusinessRunStatus>('all');
    const [editable_response, setEditableResponse] = useState('');
    const [operator_note, setOperatorNote] = useState('');
    const [loading, setLoading] = useState(true);
    const [busy, setBusy] = useState(false);
    const [detail_loading, setDetailLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [refresh, setRefresh] = useState(0);
    const controller = useRef(new AbortController());
    const detail_request = useRef(0);
    const list_request = useRef(0);

    const pii_warnings = useMemo(() => scanForPii(enquiry), [enquiry]);

    useEffect(() => {
        const active = new AbortController();
        controller.current = active;
        return () => active.abort();
    }, []);

    const assertRun = useCallback((value: BusinessRun, expected_id?: string): void => {
        if (value.company_key !== company.company_key || value.workspace_id !== company.workspace_id ||
            value.environment !== 'mock' || value.connector_mode !== 'mock' || (expected_id && value.run_id !== expected_id)) {
            throw new Error('The engine returned a run from a different company or environment. Refresh before continuing.');
        }
    }, [company.company_key, company.workspace_id]);

    useEffect(() => {
        const active = new AbortController();
        const request = ++list_request.current;
        void business_api.list_runs(company.company_key, { signal: active.signal }).then(result => {
            if (active.signal.aborted || request !== list_request.current) return;
            result.forEach(value => assertRun(value));
            setRuns(result);
        }).catch(reason => {
            if (!active.signal.aborted && request === list_request.current) setError(error_message(reason));
        }).finally(() => {
            if (!active.signal.aborted && request === list_request.current) setLoading(false);
        });
        return () => active.abort();
    }, [company.company_key, assertRun, refresh]);

    const showRun = useCallback(async (run_id: string) => {
        const request = ++detail_request.current;
        const signal = controller.current.signal;
        setDetailLoading(true);
        setError(null);
        setRun(null);
        setReport(null);
        try {
            const current = await business_api.get_run(company.company_key, run_id, { signal });
            if (signal.aborted || request !== detail_request.current) return;
            assertRun(current, run_id);
            setRun(current);
            setEditableResponse(current.draft_response);
            setOperatorNote(current.approval?.operator_note ?? '');
            const evidence_report = await business_api.get_report(company.company_key, run_id, { signal });
            if (signal.aborted || request !== detail_request.current) return;
            assertRun(evidence_report.run, run_id);
            if (evidence_report.run.version !== current.version) throw new Error('This run changed while loading. Open it again to review its latest version.');
            setReport(evidence_report);
            if (is_business_report_verified(evidence_report)) {
                setVerifiedRuns(prev => ({ ...prev, [run_id]: true }));
            }
        } catch (reason) {
            if (!signal.aborted && request === detail_request.current) setError(error_message(reason));
        } finally {
            if (!signal.aborted && request === detail_request.current) setDetailLoading(false);
        }
    }, [assertRun, company.company_key]);

    const submit = async () => {
        const signal = controller.current.signal;
        const request = pending ?? {
            idempotency_key: crypto.randomUUID(), customer_reference: company.customer.display_reference,
            offering_reference: selected_offering_ref, enquiry: enquiry.trim(),
        };
        if (!request.enquiry || busy) return;
        onPending(request);
        setBusy(true);
        setError(null);
        try {
            const created = await business_api.create_run(company.company_key, request, { signal });
            if (signal.aborted) return;
            assertRun(created);
            ++list_request.current;
            setLoading(false);
            onPending(undefined);
            setEnquiry('');
            setRuns(current => [created, ...current.filter(item => item.run_id !== created.run_id)]);
            await showRun(created.run_id);
        } catch (reason) {
            if (!signal.aborted) {
                if (rejected_request(reason)) {
                    onPending(undefined);
                    setError(`${error_message(reason)} The request was rejected. Correct the enquiry or connection settings and try again.`);
                } else {
                    setError(`${error_message(reason)} The outcome is uncertain. Retry this exact enquiry to recover the same run.`);
                }
            }
        } finally {
            if (!signal.aborted) setBusy(false);
        }
    };

    const discardPendingRetry = () => {
        if (busy) return;
        onPending(undefined);
        setEnquiry('');
        setError(null);
    };

    const decide = async (decision: BusinessDecision) => {
        if (!run || run.status !== 'pending_approval' || busy || detail_loading || !report) return;
        const target = { company_key: run.company_key, run_id: run.run_id, version: run.version };
        const signal = controller.current.signal;
        const decision_options: { signal: AbortSignal; edited_response?: string; operator_note?: string } = { signal };
        const trimmed_response = editable_response.trim();
        if (trimmed_response && trimmed_response !== run.draft_response) {
            decision_options.edited_response = trimmed_response;
        }
        const trimmed_note = operator_note.trim();
        if (trimmed_note) {
            decision_options.operator_note = trimmed_note;
        }
        setBusy(true);
        setError(null);
        setReport(null);
        try {
            const changed = await business_api.decide_run(target.company_key, target.run_id, target.version, decision, decision_options);
            if (signal.aborted) return;
            assertRun(changed, target.run_id);
            setRuns(current => current.map(item => item.run_id === changed.run_id ? changed : item));
            await showRun(target.run_id);
        } catch (reason) {
            if (!signal.aborted) {
                setRun(null);
                setError(`${error_message(reason)} Refresh and review the saved run before making another decision.`);
            }
        } finally {
            if (!signal.aborted) setBusy(false);
        }
    };

    const exportReport = () => {
        if (!report) return;
        try {
            const filename = business_report_filename(report);
            const url = URL.createObjectURL(new Blob([JSON.stringify(report, null, 2)], { type: 'application/json' }));
            const link = document.createElement('a');
            link.href = url;
            link.download = filename;
            link.click();
            window.setTimeout(() => URL.revokeObjectURL(url), 0);
        } catch (reason) {
            setError(error_message(reason));
        }
    };

    const verified = report ? is_business_report_verified(report) : false;
    const catalog_offerings = useMemo(
        () => (company.catalog_offerings && company.catalog_offerings.length > 0 ? company.catalog_offerings : [company.offering]),
        [company.catalog_offerings, company.offering]
    );
    const active_offering = useMemo(
        () => catalog_offerings.find(item => item.display_reference === selected_offering_ref) ?? company.offering,
        [catalog_offerings, selected_offering_ref, company.offering]
    );
    const price = new Intl.NumberFormat('en-US', { style: 'currency', currency: active_offering.currency }).format(active_offering.price_minor / 100);
    const filtered_runs = useMemo(
        () => status_filter === 'all' ? runs : runs.filter(item => item.status === status_filter),
        [runs, status_filter]
    );

    const renewFacts = async () => {
        if (busy) return;
        setBusy(true);
        setError(null);
        try {
            const updated = await business_api.renew_company_facts(company.company_key, { signal: controller.current.signal });
            onCompanyUpdated?.(updated);
        } catch (reason) {
            setError(error_message(reason));
        } finally {
            setBusy(false);
        }
    };

    return <section aria-label={`${company.company_key} practice workspace`} className="space-y-5">
        <div className="flex flex-wrap items-center justify-between gap-3">
            <h2 className="text-xl font-semibold">{company.display_name}</h2>
            <span className="text-sm text-amber-200">Mock · {company.workflow_display_name} · version {company.workflow_version}</span>
        </div>

        <div role="tablist" aria-label={`${company.company_key} digital twin workspace views`} className="flex flex-wrap gap-2 border-b border-zinc-800 pb-3">
            <button
                type="button"
                role="tab"
                aria-selected={active_tab === 'enquiries'}
                onClick={() => setActiveTab('enquiries')}
                className={`${button} ${active_tab === 'enquiries' ? 'border-cyan-400/60 bg-cyan-500/10 text-cyan-200' : ''}`}
            >
                <FlaskConical size={15} aria-hidden="true" />
                Enquiries & Approvals
            </button>
            <button
                type="button"
                role="tab"
                aria-selected={active_tab === 'catalog'}
                onClick={() => setActiveTab('catalog')}
                className={`${button} ${active_tab === 'catalog' ? 'border-cyan-400/60 bg-cyan-500/10 text-cyan-200' : ''}`}
            >
                <Store size={15} aria-hidden="true" />
                Catalog & A2A Knowledge
            </button>
            <button
                type="button"
                role="tab"
                aria-selected={active_tab === 'readiness'}
                onClick={() => setActiveTab('readiness')}
                className={`${button} ${active_tab === 'readiness' ? 'border-cyan-400/60 bg-cyan-500/10 text-cyan-200' : ''}`}
            >
                <ShieldCheck size={15} aria-hidden="true" />
                SOP & Live Readiness
            </button>
        </div>

        {error && <p role="alert" className="rounded-lg border border-red-500/40 p-4 text-sm text-red-200">{error}</p>}

        {active_tab === 'catalog' && (
            <Customer_Catalog_Manager companyKey={company.company_key} company={company} embedded />
        )}

        {active_tab === 'readiness' && (
            <div className="space-y-5">
                <div className={panel}>
                    <div className="flex flex-wrap items-center justify-between gap-2">
                        <h3 className="flex items-center gap-2 font-semibold">
                            <Sparkles size={16} className="text-cyan-400" aria-hidden="true" />
                            Workspace Promotion Gate (FR-10 & FR-11)
                        </h3>
                        <span className="rounded-full border border-emerald-500/30 bg-emerald-500/10 px-2.5 py-0.5 font-mono text-xs text-emerald-300">
                            FR-11 Fail-Closed Guard Active
                        </span>
                    </div>
                    <p className="text-sm text-zinc-400">
                        Promotion from isolated mock practice to live SMB operations requires explicit company-scoped storage, connector credentials, and human owner sign-off at every write boundary.
                    </p>
                    <div className="grid gap-4 md:grid-cols-3">
                        <div className="rounded-lg border border-amber-400/40 bg-amber-400/5 p-4 space-y-1.5">
                            <span className="font-mono text-xs uppercase text-amber-300">Stage 1 · Active</span>
                            <h4 className="font-semibold text-zinc-100">mock</h4>
                            <p className="text-xs text-zinc-400">Isolated SQLite per company ({company.company_key}). Deterministic fact-sheet drafts and local ticket receipts.</p>
                        </div>
                        <div className="rounded-lg border border-zinc-700 bg-zinc-950/40 p-4 space-y-1.5">
                            <span className="font-mono text-xs uppercase text-cyan-300">Stage 2 · Read-Only Live</span>
                            <h4 className="font-semibold text-zinc-100">read_only</h4>
                            <p className="text-xs text-zinc-400">Live catalog & CRM ingestion with freshness window enforcement (valid until {company.offering.fresh_until}). Zero external writes.</p>
                        </div>
                        <div className="rounded-lg border border-zinc-700 bg-zinc-950/40 p-4 space-y-1.5">
                            <span className="font-mono text-xs uppercase text-emerald-300">Stage 3 · Supervised Live</span>
                            <h4 className="font-semibold text-zinc-100">approved_write</h4>
                            <p className="text-xs text-zinc-400">Outbound customer responses require human owner approval, version lock verification, and immutable connector receipts.</p>
                        </div>
                    </div>
                    <div className="flex flex-wrap items-center justify-between gap-3 border-t border-zinc-800 pt-4">
                        <div className="space-y-1">
                            <p className="text-xs font-mono text-zinc-300">
                                Promotion Readiness: {company.readiness?.eligible_for_promotion ? 'Eligible for Stage 2 (read_only) Promotion' : 'Practice Verification Required'}
                            </p>
                            <p className="text-xs text-zinc-400">
                                Facts valid until {company.offering.fresh_until} · Verified runs: {company.readiness?.verified_runs ?? Object.keys(verified_runs).length}
                            </p>
                        </div>
                        <button type="button" className={button} disabled={busy} onClick={() => void renewFacts()}>
                            <RefreshCw size={14} aria-hidden="true" />
                            Renew Fact Freshness (+90d)
                        </button>
                    </div>
                    {company.readiness && (
                        <ul className="space-y-2 text-xs" aria-label="Promotion readiness checks">
                            {company.readiness.checks.map(item => (
                                <li key={item.id} className="flex items-start justify-between gap-3 rounded-lg border border-zinc-800 bg-zinc-950/40 p-3">
                                    <div>
                                        <p className="font-medium text-zinc-200">{item.label}</p>
                                        <p className="text-zinc-400">{item.detail}</p>
                                    </div>
                                    <span className={`rounded-full px-2 py-0.5 font-mono ${item.passed ? 'bg-emerald-500/10 text-emerald-300' : 'bg-amber-500/10 text-amber-300'}`}>
                                        {item.passed ? 'PASS' : 'PENDING'}
                                    </span>
                                </li>
                            ))}
                        </ul>
                    )}
                </div>
                <div className={panel}>
                    <h3 className="font-semibold">Company SOP Contract: {company.sop.title} · version {company.sop.version}</h3>
                    <p className="text-xs font-mono text-zinc-400">Document Reference: {company.sop.document_reference} · Workflow: {company.workflow_definition_id} (v{company.workflow_version})</p>
                    <ul className="space-y-2 text-sm">
                        {company.sop.steps.map(step => (
                            <li key={step.id} className="rounded-lg border border-zinc-800 bg-zinc-950/40 p-3">
                                <p className="font-medium text-zinc-200">{step.title} <span className="font-mono text-xs text-cyan-300">({step.id})</span></p>
                                <p className="text-xs text-zinc-400">Owner Role: {step.owner_role} · Gate Check: {step.check}</p>
                            </li>
                        ))}
                    </ul>
                </div>
            </div>
        )}

        {active_tab === 'enquiries' && <>
        <div className="grid gap-5 lg:grid-cols-2">
            <div className={panel}>
                <h3 className="font-semibold">Company facts</h3>
                <p className="text-sm text-zinc-300">{company.profile.description}</p>
                <dl className="space-y-3 text-sm"><div><dt className="text-zinc-400">Hours</dt><dd>{company.profile.hours}</dd></div><div><dt className="text-zinc-400">Contact</dt><dd>{company.profile.support_email}</dd></div><div><dt className="text-zinc-400">Policy</dt><dd>{company.profile.policy}</dd></div></dl>
                <p className="break-all text-xs text-zinc-400">Configuration {company.configuration_version} · Fixture {company.fixture_version}</p>
            </div>
            <div className={panel}>
                <h3 className="font-semibold">{active_offering.title}</h3>
                <p className="text-sm">{active_offering.description}</p>
                <p className="text-xl font-semibold text-cyan-200">{price}</p>
                <p className="break-all text-xs text-zinc-400">{active_offering.display_reference}</p>
                <p className="text-sm text-zinc-400">Facts valid until {active_offering.fresh_until}</p>
                {catalog_offerings.length > 1 && (
                    <label className="block space-y-1 pt-2 text-xs text-zinc-400">
                        <span>Active catalog offering ({catalog_offerings.length})</span>
                        <select
                            aria-label="Active catalog offering"
                            value={selected_offering_ref}
                            onChange={event => setSelectedOfferingRef(event.target.value)}
                            disabled={busy || !!pending}
                            className="w-full rounded-lg border border-zinc-700 bg-zinc-900 px-2.5 py-1.5 text-xs text-zinc-100"
                        >
                            {catalog_offerings.map(item => (
                                <option key={item.display_reference} value={item.display_reference}>
                                    {item.title} ({item.display_reference})
                                </option>
                            ))}
                        </select>
                    </label>
                )}
            </div>
        </div>
        <details className={panel}>
            <summary className="cursor-pointer text-sm font-semibold focus-visible:outline-2 focus-visible:outline-cyan-400">Current company SOP: {company.sop.title} · version {company.sop.version}</summary>
            <p className="text-sm text-zinc-400">These role assignments currently use local rules and a human owner. Model-driven agent performance is not graded yet.</p>
            <ul className="space-y-2 text-sm">{company.sop.roles.map(role => <li key={role.name}>{role.name} · {role.execution === 'human_operator' ? 'Human decision' : 'Local rules'}</li>)}</ul>
            <p className="whitespace-pre-wrap text-sm leading-relaxed">{company.sop.document}</p>
        </details>
        <form className={panel} onSubmit={event => { event.preventDefault(); void submit(); }}>
            <h3 className="font-semibold">Try a customer enquiry</h3>
            <p className="text-sm text-zinc-400">Customer: {company.customer.name} · {company.customer.display_reference}</p>
            <label className="block space-y-2 text-sm"><span>Customer enquiry</span><textarea value={enquiry} onChange={event => setEnquiry(event.target.value)} required maxLength={4000} rows={3} disabled={busy || !!pending} className="w-full rounded-lg border border-zinc-600 bg-zinc-950/50 p-3 text-zinc-100 focus-visible:outline-2 focus-visible:outline-cyan-400 disabled:opacity-60" placeholder="Ask about the offering, price, hours or policy." /></label>
            {pii_warnings.length > 0 && (
                <p role="status" className="rounded-lg border border-amber-400/40 bg-amber-400/10 px-3 py-2 text-xs text-amber-200">
                    PII Warning Detected in enquiry ({pii_warnings.join(', ')}). Redact personal identifiers before saving to practice workspaces.
                </p>
            )}
            {pending && <p className="text-sm text-amber-200">This enquiry is saved for retry. Its request key and contents stay unchanged until the engine confirms the run.</p>}
            <div className="flex flex-wrap items-center gap-3">
                <button type="submit" className={button} disabled={busy || !enquiry.trim()}>{busy ? 'Saving…' : pending ? 'Retry saved enquiry' : 'Prepare response'}</button>
                {pending && (
                    <button type="button" className={button} disabled={busy} onClick={discardPendingRetry}>
                        Discard saved retry
                    </button>
                )}
            </div>
        </form>
        <div className="grid items-start gap-5 lg:grid-cols-[minmax(240px,1fr)_2fr]">
            <div className={panel}>
                <div className="flex items-center justify-between gap-3"><h3 className="font-semibold">Saved runs</h3><button className={button} disabled={busy || loading} onClick={() => { setLoading(true); setError(null); setRefresh(value => value + 1); if (run) void showRun(run.run_id); }}><RefreshCw size={15} aria-hidden="true" />Refresh</button></div>
                {runs.length > 1 && (
                    <label className="block text-xs text-zinc-400">
                        <span className="sr-only">Filter saved runs by status</span>
                        <select
                            aria-label="Filter saved runs by status"
                            value={status_filter}
                            onChange={event => setStatusFilter(event.target.value as 'all' | BusinessRunStatus)}
                            className="w-full rounded-lg border border-zinc-700 bg-zinc-900 px-2.5 py-1.5 text-xs text-zinc-200"
                        >
                            <option value="all">All runs ({runs.length})</option>
                            <option value="pending_approval">Awaiting approval</option>
                            <option value="completed">Completed</option>
                            <option value="rejected">Rejected</option>
                            <option value="blocked">Blocked</option>
                        </select>
                    </label>
                )}
                {loading ? <p role="status" className="text-sm text-zinc-400">Loading saved runs…</p> : !filtered_runs.length ? <p className="text-sm text-zinc-400">No enquiries saved for this company yet.</p> : <ul className="space-y-2">{filtered_runs.map(item => <li key={item.run_id}><button disabled={busy} aria-pressed={run?.run_id === item.run_id} onClick={() => void showRun(item.run_id)} className="w-full space-y-1 rounded-lg border border-zinc-700 p-3 text-left text-sm hover:bg-zinc-800 focus-visible:outline-2 focus-visible:outline-cyan-400 disabled:opacity-50"><span className="block break-all font-medium">{item.display_reference}</span><span className="block truncate text-xs text-zinc-400">{item.enquiry}</span><span className="block text-xs text-zinc-400">{(verified && run?.run_id === item.run_id) || verified_runs[item.run_id] ? 'Verified' : status_labels[item.status]}</span></button></li>)}</ul>}
            </div>
            <div className={panel} aria-live="polite">
                {detail_loading ? <p role="status">Loading run and evidence…</p> : !run ? <p className="text-sm text-zinc-400">Select a saved run to review its response, approval and evidence.</p> : <>
                    <div className="flex flex-wrap items-start justify-between gap-3"><div><h3 className="break-all font-semibold">{run.display_reference}</h3><p className="mt-1 text-sm text-amber-200">{run.company_key} · Mock · {verified ? 'Verified' : status_labels[run.status]}</p></div>{report && <button className={button} onClick={exportReport}><Download size={16} aria-hidden="true" />Export report</button>}</div>
                    <div><h4 className="text-sm font-semibold">Customer enquiry</h4><p className="mt-1 whitespace-pre-wrap text-sm">{run.enquiry}</p></div>
                    <div><h4 className="text-sm font-semibold">Prepared response</h4><p className="mt-1 whitespace-pre-wrap text-sm">{run.draft_response || 'No response prepared.'}</p></div>
                    {run.blocked_reason && <p className="text-sm text-amber-200">{run.blocked_reason}</p>}
                    {report && <div className="space-y-3 border-t border-zinc-700 pt-4" aria-label="Saved SOP evaluation">
                        <h4 className="text-sm font-semibold">SOP performance: {report.sop_evaluation.status}</h4>
                        <p className="break-all text-xs text-zinc-400">{report.sop_evaluation.sop_id} · version {report.sop_evaluation.sop_version}</p>
                        <p className="text-sm text-zinc-400">Checks cover the local workflow and owner decision. Agent reasoning and response quality need separate evaluation.</p>
                        <ol className="space-y-3 text-sm">{report.sop_evaluation.steps.map(step => <li key={step.step_id}>
                            <p>{step.title}: {step.status.replaceAll('_', ' ')}</p>
                            <p className="break-all text-xs text-zinc-400">{step.owner_role} · {step.performed_by === 'not_performed' ? 'Not performed' : step.performed_by === 'human_operator' ? 'Human decision' : 'Local rules'}</p>
                        </li>)}</ol>
                    </div>}
                    <div><h4 className="text-sm font-semibold">Source facts</h4>{run.evidence.length ? <ul className="mt-2 space-y-3">{run.evidence.map((fact, index) => <li key={`${fact.source_id}-${index}`} className="rounded-lg bg-zinc-950/40 p-3 text-sm"><p className="break-all font-medium">{fact.source_id}</p><CapturedFacts content={fact.content} company_key={run.company_key} /><p className="mt-2 text-xs text-zinc-400">Updated {fact.source_updated_at} · Valid until {fact.fresh_until}</p></li>)}</ul> : <p className="text-sm text-amber-200">No source evidence is saved.</p>}</div>
                    {run.status === 'pending_approval' && <div className="space-y-3 border-t border-zinc-700 pt-4">
                        <p className="text-sm">Approve only after checking the response against this company’s facts. Approval records a mock ticket; no external message is sent.</p>
                        <label className="block space-y-1.5 text-sm">
                            <span className="font-medium text-zinc-200">Editable prepared response</span>
                            <textarea
                                aria-label="Editable prepared response"
                                value={editable_response}
                                onChange={event => setEditableResponse(event.target.value)}
                                maxLength={4000}
                                rows={3}
                                disabled={busy}
                                className="w-full rounded-lg border border-zinc-600 bg-zinc-950/50 p-3 text-sm text-zinc-100 focus-visible:outline-2 focus-visible:outline-cyan-400 disabled:opacity-60"
                            />
                        </label>
                        {editable_response !== run.draft_response && (
                            <button
                                type="button"
                                className="text-xs text-cyan-300 underline hover:text-cyan-200"
                                onClick={() => setEditableResponse(run.draft_response)}
                            >
                                Reset to fact sheet draft
                            </button>
                        )}
                        <label className="block space-y-1.5 text-sm">
                            <span className="font-medium text-zinc-200">Owner decision note (optional)</span>
                            <input
                                type="text"
                                aria-label="Owner decision note"
                                value={operator_note}
                                onChange={event => setOperatorNote(event.target.value)}
                                maxLength={1000}
                                disabled={busy}
                                placeholder="Record context for approving, editing, or rejecting this response."
                                className="w-full rounded-lg border border-zinc-600 bg-zinc-950/50 px-3 py-2 text-sm text-zinc-100 focus-visible:outline-2 focus-visible:outline-cyan-400 disabled:opacity-60"
                            />
                        </label>
                        <div className="flex flex-wrap gap-3"><button className={button} disabled={busy || !report || !run.evidence.length || !editable_response.trim()} onClick={() => void decide('approve')}>Approve response</button><button className={button} disabled={busy || !report} onClick={() => void decide('reject')}>Reject response</button></div>
                    </div>}
                    {run.approval && <div className="space-y-1 text-sm"><p>Decision: {run.approval.decision === 'approve' ? 'Approved' : 'Rejected'} · {run.approval.decided_at}{run.approval.operator_edited ? ' · Operator edited' : ''}</p>{run.approval.operator_note && <p className="text-xs text-zinc-300">Owner note: {run.approval.operator_note}</p>}</div>}
                    {run.ticket && <div className="space-y-1 border-t border-zinc-700 pt-4"><h4 className="break-all text-sm font-semibold">{run.ticket.display_reference}</h4><p className="whitespace-pre-wrap text-sm">{run.ticket.response}</p></div>}
                    {run.receipt && <div className="space-y-1 text-xs text-zinc-400"><p className="break-all">{run.receipt.display_reference}</p><p>Operation: {run.receipt.operation} · {run.receipt.status}</p><p className="break-all">Evidence: {run.receipt.evidence_location}</p></div>}
                    <div className="space-y-3 border-t border-zinc-700 pt-4"><h4 className="flex items-center gap-2 text-sm font-semibold">{verified && <CheckCircle2 size={16} className="text-emerald-300" aria-hidden="true" />}{verified ? 'Verified outcome' : 'Outcome not verified'}</h4>{report ? <><ul className="space-y-1 text-sm">{report.verification.checks.map((check, index) => <li key={`${check.name}-${index}`} className={check.passed ? 'text-zinc-300' : 'text-amber-200'}>{check.passed ? 'Pass' : 'Needs attention'}: {check.detail}</li>)}</ul><p className="text-xs text-zinc-400">{report.latency_ms} ms · Recorded provider charges ${report.cost.provider_charges_usd.toFixed(2)} · Estimated ${report.cost.estimated_usd.toFixed(2)}</p><p className="text-xs text-zinc-400">{report.cost.provenance === 'deterministic_local_no_model' ? 'Local deterministic execution; no model charges.' : `Cost provenance: ${report.cost.provenance}`}</p><p className="break-all text-xs text-zinc-400">{report.report_display_reference}</p></> : <p className="text-sm text-amber-200">A saved ticket alone does not establish success. Reload this run to retrieve its verification report.</p>}</div>
                </>}
            </div>
        </div>
        </>}
    </section>;
}

function CapturedFacts({ content, company_key }: { content: string; company_key: string }) {
    let snapshot: unknown;
    try { snapshot = JSON.parse(content); } catch { return <p className="mt-1 whitespace-pre-wrap">{content}</p>; }
    const record = (value: unknown): value is Record<string, unknown> => !!value && typeof value === 'object' && !Array.isArray(value);
    const source = record(snapshot) ? snapshot : {};
    const valid = source.company_key === company_key && source.environment === 'mock' && record(source.profile) && record(source.offering);
    const text = (value: unknown): string | null => typeof value === 'string' ? value : null;
    const profile = valid ? source.profile as Record<string, unknown> : {};
    const offering = valid ? source.offering as Record<string, unknown> : {};
    const facts = [
        ['Offering', text(offering.title)], ['Description', text(offering.description)],
        ['Hours', text(profile.hours)], ['Contact', text(profile.support_email)], ['Policy', text(profile.policy)],
    ];
    let price: string | null = null;
    if (valid && typeof offering.price_minor === 'number' && Number.isFinite(offering.price_minor) && typeof offering.currency === 'string') {
        try { price = new Intl.NumberFormat('en-US', { style: 'currency', currency: offering.currency }).format(offering.price_minor / 100); } catch { /* Invalid source currencies remain available in the full record. */ }
    }
    return <div className="mt-2 space-y-3">
        {valid ? <dl className="space-y-2">{facts.map(([label, value]) => value ? <div key={label}><dt className="text-xs text-zinc-400">{label}</dt><dd className="whitespace-pre-wrap">{value}</dd></div> : null)}{price && <div><dt className="text-xs text-zinc-400">Recorded price</dt><dd>{price}</dd></div>}</dl> : <p className="text-amber-200">This source record could not be summarized for this company. Review the full record before approval.</p>}
        <details className="text-xs text-zinc-400"><summary className="cursor-pointer focus-visible:outline-2 focus-visible:outline-cyan-400">Full captured source record</summary><pre className="mt-2 max-h-64 overflow-auto whitespace-pre-wrap break-all">{content}</pre></details>
    </div>;
}
