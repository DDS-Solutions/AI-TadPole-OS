//! @docs ARCHITECTURE:Registry
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / authority
//! - **Primary Entrypoints**: `RoleAuthorityLevel`
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared

use serde::{Deserialize, Serialize};

/// ### 📡 Protocol: RoleAuthorityLevel
/// Defines the authority level of an agent in the swarm.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    Default,
    specta::Type,
)]
#[serde(rename_all = "lowercase")]
pub enum RoleAuthorityLevel {
    /// Executive level (CEO, COO) - Strategic oversight and delegation.
    Executive,
    /// Management level (Alpha Node) - Tactical coordination.
    Management,
    /// Specialist level - Task execution.
    #[default]
    Specialist,
    /// Observer level - Read-only oversight.
    Observer,
}

const EXECUTIVE_KEYWORDS: &[&str] = &["ceo", "overlord", "executive"];
const MANAGEMENT_KEYWORDS: &[&str] = &[
    "coo",
    "cto",
    "orchestrator",
    "coordinator",
    "commander",
    "alpha",
    "lead",
    "director",
    "manager",
    "pm",
];
const OBSERVER_KEYWORDS: &[&str] = &["observer", "auditor"];

/// Splits a role string into lowercase word tokens.
/// Boundaries are non-alphanumeric characters and camelCase transitions
/// (`ProjectManager` -> `project`, `manager`; `SwarmPM` -> `swarm`, `pm`).
fn tokenize_role(role: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut prev_lower = false;

    for ch in role.chars() {
        if !ch.is_alphanumeric() {
            if !current.is_empty() {
                tokens.push(std::mem::take(&mut current));
            }
            prev_lower = false;
            continue;
        }
        if ch.is_uppercase() && prev_lower && !current.is_empty() {
            tokens.push(std::mem::take(&mut current));
        }
        prev_lower = ch.is_lowercase() || ch.is_numeric();
        current.extend(ch.to_lowercase());
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

/// Short keywords (acronyms like `pm`, `ceo`) must match a whole token, so
/// `development` no longer matches `pm`. Longer keywords may match as a prefix
/// to allow plurals/derivations (`managers`, `leadership`).
fn token_matches(token: &str, keyword: &str) -> bool {
    if keyword.len() <= 3 {
        token == keyword
    } else {
        token.starts_with(keyword)
    }
}

fn any_keyword(tokens: &[String], keywords: &[&str]) -> bool {
    tokens
        .iter()
        .any(|t| keywords.iter().any(|k| token_matches(t, k)))
}

impl RoleAuthorityLevel {
    /// Classifies a free-form role title into an authority level.
    ///
    /// Matching is word-based rather than raw substring-based. Because authority is
    /// always derived through this function (including the recruitment escalation cap),
    /// an unrecognised role safely falls back to `Specialist`.
    pub fn from_role(role: &str) -> Self {
        let tokens = tokenize_role(role);
        if any_keyword(&tokens, EXECUTIVE_KEYWORDS) {
            Self::Executive
        } else if any_keyword(&tokens, MANAGEMENT_KEYWORDS) {
            Self::Management
        } else if any_keyword(&tokens, OBSERVER_KEYWORDS) {
            Self::Observer
        } else {
            Self::Specialist
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Executive => "executive",
            Self::Management => "management",
            Self::Specialist => "specialist",
            Self::Observer => "observer",
        }
    }
}

impl std::fmt::Display for RoleAuthorityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Parses an exact authority level name (case-insensitive). For free-form role
/// titles, use [`RoleAuthorityLevel::from_role`] instead.
impl std::str::FromStr for RoleAuthorityLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "executive" => Ok(Self::Executive),
            "management" => Ok(Self::Management),
            "specialist" => Ok(Self::Specialist),
            "observer" => Ok(Self::Observer),
            other => Err(format!("unknown authority level: '{}'", other)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_canonical_roles() {
        assert_eq!(
            RoleAuthorityLevel::from_role("CEO"),
            RoleAuthorityLevel::Executive
        );
        assert_eq!(
            RoleAuthorityLevel::from_role("Chief Executive Officer"),
            RoleAuthorityLevel::Executive
        );
        assert_eq!(
            RoleAuthorityLevel::from_role("COO"),
            RoleAuthorityLevel::Management
        );
        assert_eq!(
            RoleAuthorityLevel::from_role("Product Manager"),
            RoleAuthorityLevel::Management
        );
        assert_eq!(
            RoleAuthorityLevel::from_role("Team-Lead"),
            RoleAuthorityLevel::Management
        );
        assert_eq!(
            RoleAuthorityLevel::from_role("Alpha Node"),
            RoleAuthorityLevel::Management
        );
        assert_eq!(
            RoleAuthorityLevel::from_role("Swarm Coordinator"),
            RoleAuthorityLevel::Management
        );
        assert_eq!(
            RoleAuthorityLevel::from_role("Security Auditor"),
            RoleAuthorityLevel::Observer
        );
        assert_eq!(
            RoleAuthorityLevel::from_role("Backend Engineer"),
            RoleAuthorityLevel::Specialist
        );
    }

    #[test]
    fn handles_camel_case_and_separators() {
        assert_eq!(
            RoleAuthorityLevel::from_role("ProjectManager"),
            RoleAuthorityLevel::Management
        );
        assert_eq!(
            RoleAuthorityLevel::from_role("SwarmPM"),
            RoleAuthorityLevel::Management
        );
        assert_eq!(
            RoleAuthorityLevel::from_role("cto_office"),
            RoleAuthorityLevel::Management
        );
        assert_eq!(
            RoleAuthorityLevel::from_role("engineering/director"),
            RoleAuthorityLevel::Management
        );
    }

    #[test]
    fn rejects_substring_false_positives() {
        // Previously matched `pm`, `lead`, `cto`, `coo` as raw substrings.
        assert_eq!(
            RoleAuthorityLevel::from_role("Development Analyst"),
            RoleAuthorityLevel::Specialist
        );
        assert_eq!(
            RoleAuthorityLevel::from_role("Equipment Tracker"),
            RoleAuthorityLevel::Specialist
        );
        assert_eq!(
            RoleAuthorityLevel::from_role("Pleadings Clerk"),
            RoleAuthorityLevel::Specialist
        );
        assert_eq!(
            RoleAuthorityLevel::from_role("Doctor"),
            RoleAuthorityLevel::Specialist
        );
        assert_eq!(
            RoleAuthorityLevel::from_role("Cookbook Writer"),
            RoleAuthorityLevel::Specialist
        );
    }

    #[test]
    fn display_and_from_str_round_trip() {
        for lvl in [
            RoleAuthorityLevel::Executive,
            RoleAuthorityLevel::Management,
            RoleAuthorityLevel::Specialist,
            RoleAuthorityLevel::Observer,
        ] {
            assert_eq!(lvl.to_string().parse::<RoleAuthorityLevel>().unwrap(), lvl);
            // Display must agree with the serde wire format.
            assert_eq!(serde_json::to_string(&lvl).unwrap(), format!("\"{}\"", lvl));
        }
        assert!("ceo".parse::<RoleAuthorityLevel>().is_err());
    }
}
