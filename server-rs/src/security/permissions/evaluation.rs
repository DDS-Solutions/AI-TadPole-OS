//! @docs ARCHITECTURE:SecurityModel
//!
//! ### AI Assist Note
//! - **Subsystem**: Security / Policy evaluation
//! - Pure lookup ordering and typed fact interpretation preserve exact-before-domain and agent-before-role precedence.
//!
//! ### 🔍 Debugging & Observability
//! - **Telemetry Targets**: none declared

use super::repository::{Lookup, LookupError, Subject};
use super::{
    CapabilityClass, PermissionDecision, PermissionDecisionSource as Source, PermissionMode,
};

pub(super) fn legacy_lookups<'a>(
    agent: Option<&'a str>,
    role: Option<&'a str>,
    tool: &'a str,
) -> Vec<Lookup<'a>> {
    subjects(agent, role)
        .into_iter()
        .map(|subject| Lookup::Legacy { subject, tool })
        .collect()
}

fn subjects<'a>(agent: Option<&'a str>, role: Option<&'a str>) -> Vec<Subject<'a>> {
    agent
        .map(Subject::Agent)
        .into_iter()
        .chain(role.map(Subject::Role))
        .chain(Some(Subject::Global))
        .collect()
}

pub(super) fn capability_lookups<'a>(
    agent: Option<&'a str>,
    role: Option<&'a str>,
    class: CapabilityClass,
    resource: &'a str,
    domain: Option<&'a str>,
) -> Vec<Lookup<'a>> {
    let targets = std::iter::once((resource, false)).chain(
        domain
            .filter(|domain| *domain != resource)
            .map(|domain| (domain, true)),
    );
    targets
        .flat_map(|(resource, domain)| {
            subjects(agent, role)
                .into_iter()
                .map(move |subject| Lookup::Capability {
                    subject,
                    class,
                    resource,
                    domain,
                })
        })
        .collect()
}

pub(super) fn decision(
    lookup: Lookup<'_>,
    fact: Result<Option<PermissionMode>, LookupError>,
) -> Option<PermissionDecision> {
    let mode = match fact {
        Ok(None) => return None,
        Ok(Some(mode)) => mode,
        Err(error) => {
            let source = match &error {
                LookupError::Data(_) => Source::PolicyDataError,
                LookupError::Database(_) => Source::DatabaseFailure,
            };
            return Some(PermissionDecision::with_reason(
                PermissionMode::Deny,
                source,
                format!("Policy query failure: {}", error),
            ));
        }
    };
    let (source, reason) = match lookup {
        Lookup::Legacy {
            subject: Subject::Agent(_),
            tool,
        } => (
            Source::AgentPolicy,
            format!("Agent legacy override for {}", tool),
        ),
        Lookup::Legacy {
            subject: Subject::Role(_),
            tool,
        } => (
            Source::RolePolicy,
            format!("Role legacy override for {}", tool),
        ),
        Lookup::Legacy {
            subject: Subject::Global,
            tool,
        } => (
            Source::GlobalLegacyPolicy,
            format!("Global legacy policy for {}", tool),
        ),
        Lookup::Capability {
            subject: Subject::Agent(_),
            resource,
            domain,
            ..
        } => (
            Source::AgentPolicy,
            format!(
                "Agent {} policy override for {}",
                if domain { "domain" } else { "exact" },
                resource
            ),
        ),
        Lookup::Capability {
            subject: Subject::Role(_),
            resource,
            domain,
            ..
        } => (
            Source::RolePolicy,
            format!(
                "Role {} policy override for {}",
                if domain { "domain" } else { "exact" },
                resource
            ),
        ),
        Lookup::Capability {
            subject: Subject::Global,
            resource,
            domain,
            ..
        } => (
            if domain {
                Source::DomainPolicy
            } else {
                Source::GlobalCapabilityPolicy
            },
            format!(
                "Global {} policy for {}",
                if domain { "domain" } else { "capability" },
                resource
            ),
        ),
    };
    Some(PermissionDecision::with_reason(mode, source, reason))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_global_precedes_agent_domain_and_missing_facts_continue() {
        let requests = capability_lookups(
            Some("agent"),
            Some("role"),
            CapabilityClass::Modify,
            "path:scripts/a",
            Some("domain:execution"),
        );
        assert_eq!(requests.len(), 6);
        assert!(matches!(
            requests[2],
            Lookup::Capability {
                subject: Subject::Global,
                domain: false,
                ..
            }
        ));
        assert!(matches!(
            requests[3],
            Lookup::Capability {
                subject: Subject::Agent("agent"),
                domain: true,
                ..
            }
        ));
        assert!(decision(requests[0], Ok(None)).is_none());
        assert_eq!(
            decision(requests[2], Ok(Some(PermissionMode::Deny)))
                .unwrap()
                .source,
            Source::GlobalCapabilityPolicy
        );
    }
    #[test]
    fn malformed_fact_is_a_typed_fail_closed_decision() {
        let request = Lookup::Legacy {
            subject: Subject::Global,
            tool: "write_file",
        };
        let result = decision(
            request,
            Err(LookupError::Data(anyhow::anyhow!("invalid mode"))),
        )
        .unwrap();
        assert_eq!(result.mode, PermissionMode::Deny);
        assert_eq!(result.source, Source::PolicyDataError);
    }
}
