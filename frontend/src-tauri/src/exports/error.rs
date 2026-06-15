//! Sanitized Microsoft Graph error mapping.
//!
//! Graph responses are mapped to a small, stable set of error kinds. Only the
//! HTTP status and the Graph `error.code` string are ever retained — never
//! tokens, auth URLs, or response bodies that could contain sensitive data.
//! See the error-handling sections of the OneNote/Planner design docs.

use serde::{Deserialize, Serialize};

use crate::exports::model::ExportStatus;

/// Sanitized classification of a Graph failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphErrorKind {
    /// 401 — token expired or invalid; reconnect required.
    Unauthorized,
    /// 403 — user lacks access to the resource.
    AccessDenied,
    /// 403 — tenant policy blocks user consent / the app.
    TenantBlocked,
    /// 404 — stored destination no longer exists or is not visible.
    NotFound,
    /// 413 — OneNote payload too large; split or summary-only.
    PayloadTooLarge,
    /// 429 — throttled; honor `Retry-After` / backoff.
    Throttled,
    /// 507 — OneNote section page limit reached.
    SectionFull,
    /// 503 — service unavailable; bounded backoff.
    ServiceUnavailable,
    /// Anything else.
    Unknown,
}

impl GraphErrorKind {
    /// Map an HTTP status and optional Graph error `code` to a sanitized kind.
    ///
    /// 403 is split into [`AccessDenied`](Self::AccessDenied) vs
    /// [`TenantBlocked`](Self::TenantBlocked) using the Graph error code, since
    /// the two demand different UX (reconnect/pick vs admin approval).
    pub fn from_status(status: u16, code: Option<&str>) -> Self {
        match status {
            401 => GraphErrorKind::Unauthorized,
            403 => {
                if is_tenant_block_code(code) {
                    GraphErrorKind::TenantBlocked
                } else {
                    GraphErrorKind::AccessDenied
                }
            }
            404 => GraphErrorKind::NotFound,
            413 => GraphErrorKind::PayloadTooLarge,
            429 => GraphErrorKind::Throttled,
            503 => GraphErrorKind::ServiceUnavailable,
            507 => GraphErrorKind::SectionFull,
            _ => GraphErrorKind::Unknown,
        }
    }

    /// The export status this failure should record in the ledger.
    pub fn export_status(self) -> ExportStatus {
        match self {
            GraphErrorKind::Unauthorized => ExportStatus::FailedAuth,
            GraphErrorKind::AccessDenied => ExportStatus::AccessDenied,
            GraphErrorKind::TenantBlocked => ExportStatus::TenantBlocked,
            GraphErrorKind::NotFound | GraphErrorKind::SectionFull => {
                ExportStatus::DestinationNotFound
            }
            GraphErrorKind::PayloadTooLarge
            | GraphErrorKind::Throttled
            | GraphErrorKind::ServiceUnavailable
            | GraphErrorKind::Unknown => ExportStatus::Failed,
        }
    }

    /// Whether the client may retry this failure automatically (with backoff).
    /// Auth, access, tenant, and destination failures must not be retried
    /// blindly with the same inputs.
    pub fn is_retriable(self) -> bool {
        matches!(
            self,
            GraphErrorKind::Throttled | GraphErrorKind::ServiceUnavailable
        )
    }

    /// A short, stable, log-safe code. Never includes response bodies.
    pub fn code(self) -> &'static str {
        match self {
            GraphErrorKind::Unauthorized => "unauthorized",
            GraphErrorKind::AccessDenied => "access_denied",
            GraphErrorKind::TenantBlocked => "tenant_blocked",
            GraphErrorKind::NotFound => "not_found",
            GraphErrorKind::PayloadTooLarge => "payload_too_large",
            GraphErrorKind::Throttled => "throttled",
            GraphErrorKind::SectionFull => "section_full",
            GraphErrorKind::ServiceUnavailable => "service_unavailable",
            GraphErrorKind::Unknown => "unknown",
        }
    }
}

/// Graph error codes that indicate tenant/admin consent blocking rather than a
/// per-resource access denial. Matched case-insensitively.
fn is_tenant_block_code(code: Option<&str>) -> bool {
    let Some(code) = code else { return false };
    let code = code.to_ascii_lowercase();
    [
        "consent_required",
        "tenantblocked",
        "tenant_blocked",
        "blockedbyconditionalaccess",
        "admin_consent_required",
        "interaction_required",
    ]
    .iter()
    .any(|c| code.contains(c))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_status_codes() {
        assert_eq!(GraphErrorKind::from_status(401, None), GraphErrorKind::Unauthorized);
        assert_eq!(GraphErrorKind::from_status(404, None), GraphErrorKind::NotFound);
        assert_eq!(GraphErrorKind::from_status(413, None), GraphErrorKind::PayloadTooLarge);
        assert_eq!(GraphErrorKind::from_status(429, None), GraphErrorKind::Throttled);
        assert_eq!(GraphErrorKind::from_status(507, None), GraphErrorKind::SectionFull);
        assert_eq!(GraphErrorKind::from_status(418, None), GraphErrorKind::Unknown);
    }

    #[test]
    fn splits_403_by_code() {
        assert_eq!(GraphErrorKind::from_status(403, None), GraphErrorKind::AccessDenied);
        assert_eq!(
            GraphErrorKind::from_status(403, Some("AccessDenied")),
            GraphErrorKind::AccessDenied
        );
        assert_eq!(
            GraphErrorKind::from_status(403, Some("consent_required")),
            GraphErrorKind::TenantBlocked
        );
        assert_eq!(
            GraphErrorKind::from_status(403, Some("BlockedByConditionalAccess")),
            GraphErrorKind::TenantBlocked
        );
    }

    #[test]
    fn retriability_and_status_mapping() {
        assert!(GraphErrorKind::Throttled.is_retriable());
        assert!(GraphErrorKind::ServiceUnavailable.is_retriable());
        assert!(!GraphErrorKind::Unauthorized.is_retriable());
        assert!(!GraphErrorKind::AccessDenied.is_retriable());

        assert_eq!(GraphErrorKind::Unauthorized.export_status(), ExportStatus::FailedAuth);
        assert_eq!(GraphErrorKind::TenantBlocked.export_status(), ExportStatus::TenantBlocked);
        assert_eq!(GraphErrorKind::NotFound.export_status(), ExportStatus::DestinationNotFound);
        assert_eq!(GraphErrorKind::SectionFull.export_status(), ExportStatus::DestinationNotFound);
    }
}
