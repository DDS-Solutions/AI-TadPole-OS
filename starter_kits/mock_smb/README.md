> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: SMB Practice / Local Enquiry Workflows
> - **Architecture**: `@docs ARCHITECTURE:Core`
> - **Observability**: Business run, SOP and evidence reports retain company identity.

# SMB Practice

Open **SMB Practice** in the application sidebar, or visit `/smb-practice`.
Use your normal engine connection and authentication settings.

The two practice companies share one customer-enquiry workflow:

| Company | Initial offering | SOP |
|---|---|---|
| Mock-1-Product | Rechargeable desk lamp | [Mock-1-Product-Customer-Enquiry-SOP](sops/Mock-1-Product-Customer-Enquiry-SOP-v1.md) |
| Mock-2-Service | Two-hour office cleaning visit | [Mock-2-Service-Customer-Enquiry-SOP](sops/Mock-2-Service-Customer-Enquiry-SOP-v1.md) |

Select a company, prepare an enquiry, review its fact sheet, and approve or reject
the response. Approval records a local mock ticket and receipt. No external
message, appointment, payment, or model call is made.

Switching companies changes the viewed workspace. Existing runs, pending owner
decisions, retry keys and saved evidence retain their original company identity.
Exported reports keep the exact company prefix.

Each company has prefixed Support-Agent, Records-Agent and Review-Agent role
assignments plus a human Owner. In this first slice the three agent roles are
implemented with local rules. They are not active model agents in the global
registry. Reports evaluate the SOP steps and identify human versus local-rule
work; agent reasoning and semantic response quality remain unevaluated.

A correctly rejected response can follow the SOP without completing a customer
outcome. Pending, blocked and failed results remain visible. Saved tickets alone
do not establish success: the independent decision, ticket, receipt, facts and
SOP digest must agree.

## Repeatable verification

With the engine running, supply its token through an environment variable and run:

```powershell
python execution/verify_mock_smb.py --target http://localhost:8000 --report-dir reports/mock-smb
```

The default token variable is `NEURAL_TOKEN`; use `--token-env` to select another.
Do not place credentials on the command line. The verifier rejects non-loopback
targets and redirects, creates unique mock enquiries, and preserves all attempts.
It records thirteen HTTP checks per company. Process restart, fault injection,
agent reasoning and real integrations are outside this HTTP suite.

Domain and production-router checks run with:

```powershell
cargo test --manifest-path server-rs/Cargo.toml --bin server-rs business
python -m unittest discover -s tests -p test_verify_mock_smb.py
```

Fixtures contain synthetic source facts dated 2026-10-23, expiring 2027-01-21.
The runtime blocks stale facts. Refreshing these embedded fixtures or their SOPs
requires explicit version changes and a rebuild. Historical snapshots are retained.
The current fixture directory has no secrets or real customer data.

## Moving toward a real company

Keep the shared domain workflow and replace its registered profile and connector
in a fresh isolated workspace. Live profile registration, real connector adapters,
outbox/reconciliation and agent tool/memory isolation still need implementation
and certification. Passing this local suite does not certify those capabilities.
