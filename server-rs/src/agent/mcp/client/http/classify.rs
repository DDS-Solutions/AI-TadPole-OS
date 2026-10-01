//! @docs ARCHITECTURE:Registry:Mcp
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / MCP HTTP Failure Classification
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Exact error classification: ConnectionRefused and HostUnreachable allow stdio fallback.
//! - `[Structural]` Timeouts, TLS errors, HTTP 4xx/5xx, and protocol errors MUST fail closed.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none declared
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `agent::mcp::client::http::classify::tests::*`

use std::error::Error as StdError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpDiscoveryFailureKind {
    ConnectionRefused,
    HostUnreachable,
    UnsupportedVersionNoIntersection(Vec<String>),
    ProtocolInconsistency(String),
    FailClosed(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpTransportFailureKind {
    ConnectionRefused,
    HostUnreachable,
    Timeout,
    Other,
}

pub fn classify_io_error(error: &std::io::Error) -> HttpTransportFailureKind {
    match error.kind() {
        std::io::ErrorKind::ConnectionRefused => HttpTransportFailureKind::ConnectionRefused,
        std::io::ErrorKind::HostUnreachable => HttpTransportFailureKind::HostUnreachable,
        std::io::ErrorKind::TimedOut => HttpTransportFailureKind::Timeout,
        _ => match error.raw_os_error() {
            Some(61 | 111 | 10061) => HttpTransportFailureKind::ConnectionRefused,
            Some(65 | 113 | 10065) => HttpTransportFailureKind::HostUnreachable,
            _ => HttpTransportFailureKind::Other,
        },
    }
}

pub fn classify_reqwest_error(error: &reqwest::Error) -> HttpTransportFailureKind {
    if error.is_timeout() {
        return HttpTransportFailureKind::Timeout;
    }

    let mut source = error.source();
    while let Some(cause) = source {
        if let Some(io_error) = cause.downcast_ref::<std::io::Error>() {
            let classified = classify_io_error(io_error);
            if classified != HttpTransportFailureKind::Other {
                return classified;
            }
        }
        source = cause.source();
    }

    HttpTransportFailureKind::Other
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    fn test_transport_failure_classification_is_exact_and_fail_closed() {
        assert_eq!(
            classify_io_error(&std::io::Error::from(std::io::ErrorKind::ConnectionRefused)),
            HttpTransportFailureKind::ConnectionRefused
        );
        assert_eq!(
            classify_io_error(&std::io::Error::from(std::io::ErrorKind::HostUnreachable)),
            HttpTransportFailureKind::HostUnreachable
        );
        assert_eq!(
            classify_io_error(&std::io::Error::from(std::io::ErrorKind::TimedOut)),
            HttpTransportFailureKind::Timeout
        );
        assert_eq!(
            classify_io_error(&std::io::Error::from(std::io::ErrorKind::ConnectionReset)),
            HttpTransportFailureKind::Other
        );
    }
}
