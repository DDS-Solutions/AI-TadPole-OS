//! @docs ARCHITECTURE:SecurityModel
//!
//! ### AI Assist Note
//! - **Subsystem**: Security / Policy repository
//! - SQLite returns typed missing, malformed, and infrastructure outcomes; it never decides precedence.
//!
//! ### 🔍 Debugging & Observability
//! - **Telemetry Targets**: `[Security]`

use super::{CapabilityClass, PermissionMode};
use sqlx::SqlitePool;

#[derive(Clone, Copy, Debug)]
pub(super) enum Subject<'a> {
    Agent(&'a str),
    Role(&'a str),
    Global,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum Lookup<'a> {
    Legacy {
        subject: Subject<'a>,
        tool: &'a str,
    },
    Capability {
        subject: Subject<'a>,
        class: CapabilityClass,
        resource: &'a str,
        domain: bool,
    },
}

#[derive(Debug, thiserror::Error)]
pub(super) enum LookupError {
    #[error("PolicyDataError: {0}")]
    Data(anyhow::Error),
    #[error("DatabaseFailure: {0}")]
    Database(sqlx::Error),
}

pub(super) struct PolicyRepository {
    pool: SqlitePool,
}

impl PolicyRepository {
    pub(super) fn new(pool: &SqlitePool) -> Self {
        Self { pool: pool.clone() }
    }

    pub(super) async fn lookup(
        &self,
        lookup: Lookup<'_>,
    ) -> Result<Option<PermissionMode>, LookupError> {
        let (query, binds): (&str, Vec<&str>) = match lookup {
            Lookup::Legacy { subject: Subject::Agent(id), tool } => ("SELECT mode FROM agent_permission_policies WHERE agent_id = ?1 AND tool_name = ?2", vec![id, tool]),
            Lookup::Legacy { subject: Subject::Role(role), tool } => ("SELECT mode FROM role_permission_policies WHERE role = ?1 AND tool_name = ?2", vec![role, tool]),
            Lookup::Legacy { subject: Subject::Global, tool } => ("SELECT mode FROM permission_policies WHERE tool_name = ?1", vec![tool]),
            Lookup::Capability { subject: Subject::Agent(id), class, resource, .. } => ("SELECT mode FROM agent_capability_policies WHERE agent_id = ?1 AND LOWER(capability_class) = LOWER(?2) AND resource_pattern = ?3", vec![id, class.as_canonical_str(), resource]),
            Lookup::Capability { subject: Subject::Role(role), class, resource, .. } => ("SELECT mode FROM role_capability_policies WHERE role = ?1 AND LOWER(capability_class) = LOWER(?2) AND resource_pattern = ?3", vec![role, class.as_canonical_str(), resource]),
            Lookup::Capability { subject: Subject::Global, class, resource, .. } => ("SELECT mode FROM capability_policies WHERE LOWER(capability_class) = LOWER(?1) AND resource_pattern = ?2", vec![class.as_canonical_str(), resource]),
        };
        let mut request = sqlx::query_as::<_, (String,)>(query);
        for value in binds {
            request = request.bind(value);
        }
        match request.fetch_one(&self.pool).await {
            Ok((raw,)) => raw.parse().map(Some).map_err(|error| {
                tracing::error!("⚠️ [Security] Malformed permission mode in DB: {}", error);
                LookupError::Data(error)
            }),
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(error) => {
                tracing::error!(
                    "🚨 [Security] Database error during permission evaluation: {}",
                    error
                );
                Err(LookupError::Database(error))
            }
        }
    }
    pub(super) async fn signed_active(&self, capability: CapabilityClass, resource: &str) -> bool {
        let cap_str = capability.as_canonical_str();
        let res: Result<Option<(i32,)>, _> = sqlx::query_as(
            "SELECT 1 FROM signed_capability_manifests \
             WHERE LOWER(risk_class) = LOWER(?1) \
               AND (resource_pattern = ?2 OR ?2 LIKE resource_pattern ESCAPE '\\') \
               AND status = 'active' \
               AND (expiration IS NULL OR expiration > CURRENT_TIMESTAMP) \
             LIMIT 1",
        )
        .bind(cap_str)
        .bind(resource)
        .fetch_optional(&self.pool)
        .await;

        match res {
            Ok(Some(_)) => true,
            Ok(None) => false,
            Err(e) => {
                tracing::warn!(
                    "⚠️ [Security] Failed to verify signed capability manifest in DB: {}",
                    e
                );
                false
            }
        }
    }
}
