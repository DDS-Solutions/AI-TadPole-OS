> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Documentation / support_triage
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: Information drift, legacy terminology, or documentation mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[support_triage]`)

# Support Triage Workflow

1.  **Ticket Ingest**: User pastes a batch of customer emails or tickets.
2.  **Classification**: Triage Bot categorizes each ticket (Bug/Feature/Help).
3.  **Drafting**: Triage Bot drafts a response using internal knowledge base.
4.  **Sentiment Check**: Feedback Synthesizer identifies "Critical" or "Angry" sentiment.
5.  **Oversight**: Success Lead reviews "Critical" drafts before final approval.