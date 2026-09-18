> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Documentation / sme_discovery
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[sme_discovery]`)

# SME Discovery & Onboarding SOP

This SOP defines the process for an agent to perform an initial discovery of a small business's digital footprint and suggest a bootstrap plan.

## Initial Footprint Analysis
Investigate the primary website and social media presence of the company. Identify their core product or service and their unique value proposition.
Use the `search` tool or browse directly if you have a URL.

## Technical Stack Identification
Analyze the company's public-facing technology. Look for signs of e-commerce platforms (Shopify, WooCommerce), CRM usage (HubSpot, Salesforce), or generic CMS (WordPress, Webflow).
This helps determine which Tadpole OS connectors will be most valuable.

## Pain Point Synthesis
Based on the discovery, hypothesize 3 major operational pain points the company might be facing (e.g., manual data entry between systems, slow customer response times, fragmented inventory).

## Recommendation Report
Generate a concise report for the user, suggesting which 3 Tadpole OS connectors and 2 automated missions they should start with to see immediate ROI.