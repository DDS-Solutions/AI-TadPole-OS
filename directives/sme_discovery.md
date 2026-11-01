> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Core Directives / sme_discovery
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[sme_discovery]`)

# SME Discovery & Onboarding SOP

This SOP defines the process for an agent to perform an initial discovery of a small business's digital footprint and suggest a bootstrap plan.

> 💡 **Bound Skills**:
> - Use `@[skills/smb-data-ingestion]` to validate synthetic or operator-approved sample exports and prepare a preview; do not import business records.
> - Use `@[skills/digital-twin-voice]` to draft founder-voice guidance for preview; do not persist a company persona in a shared workspace.
> - Use `@[skills/pii-redaction]` as redaction guidance; the skill alone does not guarantee end-to-end sanitization.

> [!NOTE]
> **Plane 2 discovery is preview-only** *(IDENTITY.md Directives #1 & #9)*: Company-scoped runtime implementation is pending. The generic runner rejects only identities whose stored `department` exactly matches `Plane 2: SMB-Digital-Twin` (case-insensitive); other labels do not trigger the guard. Use public or synthetic information for a discovery preview, do not ingest customer records or dispatch live business workflows, and stop at preview. Before any future execution, pass the Plane 2 Socratic Gate: ask 3+ discovery questions covering brand tone, data schemas, and financial/PII boundaries; surface connector trade-offs; and document edge cases (e.g., businesses with no web presence, multi-location operators).

## Initial Footprint Analysis
Investigate the primary website and social media presence of the company. Identify their core product or service and their unique value proposition.
Use the `search` tool or browse directly if you have a URL.

## Technical Stack Identification
Analyze the company's public-facing technology. Look for signs of e-commerce platforms (Shopify, WooCommerce), CRM usage (HubSpot, Salesforce), or generic CMS (WordPress, Webflow).
This helps recommend which connectors could be valuable once company-scoped runtime support is available; do not connect live systems during discovery preview.

## Pain Point Synthesis
Based on the discovery, hypothesize 3 major operational pain points the company might be facing (e.g., manual data entry between systems, slow customer response times, fragmented inventory).

## Recommendation Report
Generate a concise preview report suggesting up to 3 potential connectors and 2 candidate missions for later evaluation. Do not claim they can be run or deliver ROI until company-scoped runtime support is implemented.
