"""@docs ARCHITECTURE:Core

### AI Assist Note
Probe the two authorized practice companies through the real HTTP contract.
### 🔍 Debugging & Observability
Versioned reports retain failed and missing checks; credentials are never reported.
"""

import argparse
import ipaddress
import json
import os
import uuid
from datetime import datetime, timezone
from pathlib import Path
from urllib.error import HTTPError
from urllib.parse import urlsplit
from urllib.request import HTTPRedirectHandler, ProxyHandler, Request, build_opener

COMPANIES = ("Mock-1-Product", "Mock-2-Service")
REQUIRED = (
    "identity", "created", "pending_sop", "duplicate", "payload_conflict",
    "cross_company_read", "cross_company_decision", "cross_company_report",
    "stale_approval", "approved", "verified_evidence", "terminal_replay", "safe_rejection",
)
EXPECTED_EVIDENCE = {
    "completed", "company_scope", "workflow_context", "sop_context", "approval_persisted",
    "ticket_persisted", "receipt_persisted", "fact_backed_response", "source_fresh_at_approval",
}


def local_target(value):
    parsed = urlsplit(value)
    host = parsed.hostname or ""
    try:
        loopback = ipaddress.ip_address(host).is_loopback
    except ValueError:
        loopback = host.lower() == "localhost"
    if (parsed.scheme not in ("http", "https") or not loopback or parsed.username is not None
            or parsed.password is not None or parsed.path not in ("", "/") or parsed.query or parsed.fragment):
        raise ValueError("Target must be a loopback HTTP(S) origin without credentials or a path")
    _ = parsed.port
    return value.rstrip("/")


class NoRedirects(HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


class Client:
    def __init__(self, target, token):
        self.target = local_target(target)
        self.token = token
        self.opener = build_opener(ProxyHandler({}), NoRedirects())

    def call(self, method, path, body=None, authenticated=True):
        headers = {"Content-Type": "application/json"}
        if authenticated:
            headers["Authorization"] = "Bearer " + self.token
        request = Request(self.target + path, data=None if body is None else json.dumps(body).encode(), headers=headers, method=method)
        try:
            response = self.opener.open(request, timeout=15)
        except HTTPError as error:
            response = error
        with response:
            data = response.read(1_000_001)
            if len(data) > 1_000_000:
                raise ValueError("Response exceeds verification limit")
            return response.code, json.loads(data)


def finalized(company, checks, evidence, error=None):
    complete = {name: checks.get(name) is True for name in REQUIRED}
    return {
        "suite_version": "mock-smb-http-1", "company_key": company,
        "passed": error is None and all(complete.values()), "checks": complete, "evidence": evidence,
        "error": error, "scope": "HTTP workflow, SOP binding and saved-record receipts",
        "not_evaluated": ["process restart", "injected connector faults", "model agent reasoning", "semantic response quality", "real connectors"],
    }


def run_path(base, run):
    identity = run["run_id"]
    if not isinstance(identity, str) or str(uuid.UUID(identity)) != identity:
        raise ValueError("Noncanonical run identity")
    return base + "/runs/" + identity


def verify_company(client, company, operation):
    checks, evidence = {}, {}
    try:
        base = "/v1/business/companies/" + company
        status, profile = client.call("GET", base)
        checks["identity"] = (status == 200 and profile["company_key"] == company and profile["display_name"] == company
                              and profile["environment"] == "mock" and profile["sop"]["id"].startswith(company + "-")
                              and len(profile["sop"]["steps"]) == 6 and len(profile["sop"]["roles"]) == 4)
        if not checks["identity"]:
            raise ValueError("Unexpected company identity")
        body = {"idempotency_key": operation, "customer_reference": profile["customer"]["display_reference"],
                "offering_reference": profile["offering"]["display_reference"], "enquiry": "What are the listed price, hours and policy?"}
        status, run = client.call("POST", base + "/runs", body)
        checks["created"] = (status == 201 and run["status"] == "pending_approval" and run["company_key"] == company
                             and run["workspace_id"] == profile["workspace_id"] and run["ticket"] is None)
        path = run_path(base, run)
        status, report = client.call("GET", path + "/report")
        checks["pending_sop"] = (status == 200 and report["sop_evaluation"]["status"] == "pending"
                                 and not report["verification"]["verified"] and run["sop_version"] == profile["sop"]["version"]
                                 and any(step["owner_role"].endswith("-Owner") and step["status"] == "pending" and step["performed_by"] == "not_performed" for step in report["sop_evaluation"]["steps"])
                                 and len(run["sop_digest"]) == 64)
        status, replay = client.call("POST", base + "/runs", body)
        checks["duplicate"] = status == 200 and replay["run_id"] == run["run_id"]
        checks["payload_conflict"] = client.call("POST", base + "/runs", {**body, "enquiry": "A different enquiry"})[0] == 409
        other = COMPANIES[1] if company == COMPANIES[0] else COMPANIES[0]
        other_path = "/v1/business/companies/" + other + "/runs/" + run["run_id"]
        checks["cross_company_read"] = client.call("GET", other_path)[0] == 404
        decision = {"expected_version": run["version"], "decision": "approve"}
        checks["cross_company_decision"] = client.call("POST", other_path + "/decision", decision)[0] == 404
        checks["cross_company_report"] = client.call("GET", other_path + "/report")[0] == 404
        checks["stale_approval"] = client.call("POST", path + "/decision", {**decision, "expected_version": 99})[0] == 409
        status, done = client.call("POST", path + "/decision", decision)
        checks["approved"] = status == 200 and done["status"] == "completed" and done["approval"]["decision"] == "approve"
        status, report = client.call("GET", path + "/report")
        evaluation = report["sop_evaluation"]
        saved = report["run"]
        named = {check["name"]: check["passed"] is True for check in report["verification"]["checks"]}
        checks["verified_evidence"] = (
            status == 200 and report["verification"]["verified"] is True and EXPECTED_EVIDENCE <= named.keys()
            and all(named[name] for name in EXPECTED_EVIDENCE)
            and evaluation["status"] == "passed" and len(evaluation["steps"]) == 6
            and all(step["status"] == "passed" for step in evaluation["steps"])
            and evaluation["agent_execution_evaluated"] is False and evaluation["semantic_response_quality_evaluated"] is False
            and evaluation["sop_digest"] == run["sop_digest"] and saved["run_id"] == run["run_id"]
            and saved["ticket"]["response"] == run["draft_response"]
            and saved["receipt"]["external_record_reference"] == saved["ticket"]["display_reference"]
            and saved["receipt"]["company_key"] == company and saved["receipt"]["run_id"] == run["run_id"]
            and profile["profile"]["policy"] in saved["draft_response"]
        )
        evidence["approved_report"] = report
        checks["terminal_replay"] = client.call("POST", path + "/decision", decision)[0] == 409
        _, denied = client.call("POST", base + "/runs", {**body, "idempotency_key": operation + "-deny"})
        denied_path = run_path(base, denied)
        status, denied = client.call("POST", denied_path + "/decision", {"expected_version": denied["version"], "decision": "reject"})
        _, rejection_report = client.call("GET", denied_path + "/report")
        checks["safe_rejection"] = (status == 200 and denied["status"] == "rejected" and denied["ticket"] is None and denied["receipt"] is None
                                    and rejection_report["sop_evaluation"]["status"] == "passed" and not rejection_report["verification"]["verified"])
        evidence["rejection_report"] = rejection_report
        return finalized(company, checks, evidence)
    except Exception as error:
        return finalized(company, checks, evidence, type(error).__name__ + ": verification could not complete")


def main():
    parser = argparse.ArgumentParser(description="Verify only the two local mock SMB companies; preserves test records.")
    parser.add_argument("--target", required=True)
    parser.add_argument("--report-dir", required=True, type=Path)
    parser.add_argument("--token-env", default="NEURAL_TOKEN")
    args = parser.parse_args()
    try:
        target = local_target(args.target)
        token = os.environ.get(args.token_env, "").strip()
        if not token:
            raise ValueError("Configured token environment variable is empty")
    except ValueError as error:
        parser.error(str(error))
    client = Client(target, token)
    operation = "sop-verification-" + uuid.uuid4().hex
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ") + "-" + operation[-8:]
    args.report_dir.mkdir(parents=True, exist_ok=True)
    results = []
    for company in COMPANIES:
        report = verify_company(client, company, operation)
        report["target"] = target
        (args.report_dir / f"{company}-Verification-{stamp}.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
        results.append(report)
        print(f"{company}: {sum(report['checks'].values())}/{len(REQUIRED)} HTTP checks passed")
    return 0 if all(report["passed"] for report in results) else 1


if __name__ == "__main__":
    raise SystemExit(main())
