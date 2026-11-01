//! @docs ARCHITECTURE:Core
//! ### AI Assist Note
//! The first connector owns mock-local writes only. Future external connectors require
//! certified idempotency and reconciliation before they may be registered for live use.
//! ### 🔍 Debugging & Observability
//! Connector failures become durable blocked/failed runs; receipts identify company and run.

use super::types::{Company, Offering, Receipt, Run, Ticket};
use async_trait::async_trait;
use sqlx::SqliteConnection;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ConnectorFailure {
    MissingFacts,
    Timeout,
    Persistence,
    Unavailable,
}

impl ConnectorFailure {
    pub fn message(self) -> &'static str {
        match self {
            Self::MissingFacts => "The requested offering has no verified source facts.",
            Self::Timeout => "The business connector timed out; no ticket was committed.",
            Self::Persistence => "The business connector could not persist its evidence.",
            Self::Unavailable => "The selected connector capability is unavailable.",
        }
    }
}

#[async_trait]
pub(super) trait BusinessConnector: Send + Sync {
    fn mode(&self) -> &'static str;
    async fn retrieve_offering(
        &self,
        company: &Company,
        reference: &str,
    ) -> Result<Offering, ConnectorFailure>;
    /// Implementations must bind the operation to run_id and return verifiable evidence.
    /// This local transaction cannot roll back remote systems; live registration remains disabled.
    async fn record_ticket(
        &self,
        connection: &mut SqliteConnection,
        company: &Company,
        run: &Run,
    ) -> Result<(Ticket, Receipt), ConnectorFailure>;
}

pub(super) struct MockConnector;

#[async_trait]
impl BusinessConnector for MockConnector {
    fn mode(&self) -> &'static str {
        "mock"
    }

    async fn retrieve_offering(
        &self,
        company: &Company,
        reference: &str,
    ) -> Result<Offering, ConnectorFailure> {
        if company.environment != "mock" || company.connector_mode != "mock" {
            return Err(ConnectorFailure::Unavailable);
        }
        if reference != company.offering.display_reference {
            return Err(ConnectorFailure::MissingFacts);
        }
        Ok(company.offering.clone())
    }

    async fn record_ticket(
        &self,
        connection: &mut SqliteConnection,
        company: &Company,
        run: &Run,
    ) -> Result<(Ticket, Receipt), ConnectorFailure> {
        if company.environment != "mock"
            || company.connector_mode != "mock"
            || run.company_key != company.company_key
            || run.workspace_id != company.workspace_id
            || run.environment != "mock"
        {
            return Err(ConnectorFailure::Unavailable);
        }
        let ticket_id = Uuid::new_v4().to_string();
        let ticket = Ticket {
            display_reference: format!("{}-Ticket-{}", company.company_key, ticket_id),
            id: ticket_id,
            response: run.draft_response.clone(),
        };
        let receipt_id = Uuid::new_v4().to_string();
        let receipt = Receipt {
            display_reference: format!("{}-Receipt-{}", company.company_key, receipt_id),
            id: receipt_id,
            operation: "record_customer_enquiry".into(),
            status: "completed".into(),
            external_record_reference: ticket.display_reference.clone(),
            evidence_location: format!("business://{}/tickets/{}", company.workspace_id, ticket.id),
            company_key: company.company_key.clone(),
            run_id: run.run_id.clone(),
        };
        let ticket_json =
            serde_json::to_string(&ticket).map_err(|_| ConnectorFailure::Persistence)?;
        let receipt_json =
            serde_json::to_string(&receipt).map_err(|_| ConnectorFailure::Persistence)?;
        sqlx::query("INSERT INTO business_tickets(id,run_id,company_key,payload) VALUES(?,?,?,?)")
            .bind(&ticket.id)
            .bind(&run.run_id)
            .bind(&company.company_key)
            .bind(ticket_json)
            .execute(&mut *connection)
            .await
            .map_err(|_| ConnectorFailure::Persistence)?;
        sqlx::query("INSERT INTO business_receipts(id,run_id,company_key,payload) VALUES(?,?,?,?)")
            .bind(&receipt.id)
            .bind(&run.run_id)
            .bind(&company.company_key)
            .bind(receipt_json)
            .execute(&mut *connection)
            .await
            .map_err(|_| ConnectorFailure::Persistence)?;
        Ok((ticket, receipt))
    }
}
