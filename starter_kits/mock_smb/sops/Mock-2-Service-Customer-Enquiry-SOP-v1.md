# Mock-2-Service-Customer-Enquiry-SOP

Version: 1
Company: Mock-2-Service
Workflow: customer_enquiry, version 1

Purpose: prepare a traceable response about the practice service, obtain owner
approval, and record the result exactly once.

Roles:
- Mock-2-Service-Support-Agent: confirm the company and prepare the fact sheet.
- Mock-2-Service-Records-Agent: record an approved response and its receipt.
- Mock-2-Service-Review-Agent: check the saved records against this procedure.
- Mock-2-Service-Owner: approve or reject the exact prepared response.

These are assigned role names. In the initial implementation the agent roles
use local rules; the owner decision is made by the human operator. No model
reasoning or independent agent execution is implied by these role names.

1. Confirm company, customer and offering ownership. Use only this company's
   workspace. Preserve the same request key and body when recovering a retry.
2. Retrieve dated source facts. Block stale, missing or mismatched facts. Never
   invent appointment availability, confirmed bookings, travel charges or tax.
3. Prepare a fact sheet using the captured offering, price, hours and policy.
   Cite the source and preserve its version. This fact sheet is not automatically
   a complete answer to every customer question.
4. Ask the owner to review whether the response answers the actual enquiry.
   Do not record a ticket without approval of that exact run version. Rejecting
   a response must produce no ticket. Recheck freshness before recording.
5. Record an approved response once. Save the decision, ticket and receipt
   atomically. A failure or timeout must not leave a partial ticket or claim
   success. External customer messages and payments are outside this procedure.
6. Review the independent saved records. Company, SOP version, facts, approval,
   ticket and receipt must agree. Missing or altered evidence is not success.

Evaluation:
- A pending owner decision remains pending.
- A correctly rejected response can comply with this SOP without producing a
  completed customer outcome.
- A safe stop for unavailable or stale facts is blocked, not a confirmed booking.
- A completed outcome requires every saved-record check to pass.
- Semantic response quality, agent reasoning and real connector behavior require
  separate evaluation; the local rules cannot certify them.
