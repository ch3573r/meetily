//! Planner task request builder.
//!
//! Builds `POST /planner/tasks` request bodies from reviewed action items.
//! Conservative by design: requires an explicit `planId` and `bucketId`, a
//! non-empty single-line title, and never auto-assigns a task. See
//! `docs/integrations/planner-export.md`.

use serde::{Deserialize, Serialize};

use crate::exports::model::{ExportActionItem, MeetingExport};

/// A reviewed Planner destination. Both IDs are user-selected/pasted; ClawScribe
/// does not discover plans/buckets (that would need broad directory scopes).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlannerDestination {
    #[serde(rename = "planId")]
    pub plan_id: String,
    #[serde(rename = "bucketId")]
    pub bucket_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlannerBuildError {
    MissingPlanId,
    MissingBucketId,
    EmptyTitle,
}

impl PlannerBuildError {
    pub fn message(self) -> &'static str {
        match self {
            PlannerBuildError::MissingPlanId => "Planner planId is required",
            PlannerBuildError::MissingBucketId => "Planner bucketId is required",
            PlannerBuildError::EmptyTitle => "Planner task title must not be empty",
        }
    }
}

impl std::fmt::Display for PlannerBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message())
    }
}

impl std::error::Error for PlannerBuildError {}

/// Trim a task title to a single line. Planner titles are single-line values.
fn normalize_title(raw: &str) -> String {
    raw.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Build the JSON body for a single Planner task create call.
///
/// Returns an error rather than producing an invalid request when the
/// destination or title is missing. Assignments are always empty: extracted
/// speaker names are never auto-mapped to Azure AD users.
pub fn build_task_request(
    destination: &PlannerDestination,
    action: &ExportActionItem,
) -> Result<serde_json::Value, PlannerBuildError> {
    if destination.plan_id.trim().is_empty() {
        return Err(PlannerBuildError::MissingPlanId);
    }
    if destination.bucket_id.trim().is_empty() {
        return Err(PlannerBuildError::MissingBucketId);
    }
    let title = normalize_title(&action.task);
    if title.is_empty() {
        return Err(PlannerBuildError::EmptyTitle);
    }

    Ok(serde_json::json!({
        "planId": destination.plan_id,
        "bucketId": destination.bucket_id,
        "title": title,
        "assignments": {},
    }))
}

/// Assign deterministic `action-1`, `action-2`, ... ids to action items that
/// don't already carry one. Idempotent for items that already have ids.
pub fn ensure_local_action_ids(items: &mut [ExportActionItem]) {
    for (i, item) in items.iter_mut().enumerate() {
        if item.local_action_id.trim().is_empty() {
            item.local_action_id = format!("action-{}", i + 1);
        }
    }
}

/// Stable hash of an action's title, used in the per-item dedupe key so that an
/// edited action gets a fresh key.
pub fn action_hash(action: &ExportActionItem) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(normalize_title(&action.task).as_bytes());
    let digest = hasher.finalize();
    digest[..8].iter().map(|b| format!("{b:02x}")).collect()
}

/// Build the per-item Planner dedupe key documented in `planner-export.md`:
/// `planner:{tenantId}:{userId}:{planId}:{bucketId}:{meetingHash}:{localActionId}:{actionHash}`.
pub fn dedupe_key(
    tenant_id: &str,
    user_id: &str,
    destination: &PlannerDestination,
    meeting: &MeetingExport,
    action: &ExportActionItem,
) -> String {
    format!(
        "planner:{tenant}:{user}:{plan}:{bucket}:{meeting_hash}:{action_id}:{action_hash}",
        tenant = tenant_id,
        user = user_id,
        plan = destination.plan_id,
        bucket = destination.bucket_id,
        meeting_hash = meeting.artifact_hash(),
        action_id = action.local_action_id,
        action_hash = action_hash(action),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dest() -> PlannerDestination {
        PlannerDestination {
            plan_id: "plan-1".into(),
            bucket_id: "bucket-1".into(),
        }
    }

    fn action(id: &str, task: &str) -> ExportActionItem {
        ExportActionItem {
            local_action_id: id.into(),
            task: task.into(),
            owner: None,
            due_date: None,
        }
    }

    #[test]
    fn builds_minimal_task() {
        let body = build_task_request(&dest(), &action("action-1", "Send  the\nproposal")).unwrap();
        assert_eq!(body["planId"], "plan-1");
        assert_eq!(body["bucketId"], "bucket-1");
        // Title normalized to a single line.
        assert_eq!(body["title"], "Send the proposal");
        // No assignment by default.
        assert_eq!(body["assignments"], serde_json::json!({}));
    }

    #[test]
    fn rejects_missing_destination_and_title() {
        let mut bad = dest();
        bad.plan_id = "  ".into();
        assert_eq!(
            build_task_request(&bad, &action("a", "x")),
            Err(PlannerBuildError::MissingPlanId)
        );

        let mut bad = dest();
        bad.bucket_id = "".into();
        assert_eq!(
            build_task_request(&bad, &action("a", "x")),
            Err(PlannerBuildError::MissingBucketId)
        );

        assert_eq!(
            build_task_request(&dest(), &action("a", "   \n ")),
            Err(PlannerBuildError::EmptyTitle)
        );
    }

    #[test]
    fn assigns_local_ids_deterministically() {
        let mut items = vec![action("", "a"), action("kept", "b"), action("", "c")];
        ensure_local_action_ids(&mut items);
        assert_eq!(items[0].local_action_id, "action-1");
        assert_eq!(items[1].local_action_id, "kept");
        assert_eq!(items[2].local_action_id, "action-3");
    }

    #[test]
    fn dedupe_key_is_deterministic_and_title_sensitive() {
        let meeting = MeetingExport {
            meeting_id: "m1".into(),
            title: "Sync".into(),
            created_at: None,
            executive_summary: "s".into(),
            decisions: vec![],
            action_items: vec![],
            transcript_excerpt: None,
            summary_html: None,
        };
        let a = action("action-1", "Send proposal");
        let k1 = dedupe_key("tenant", "user", &dest(), &meeting, &a);
        let k2 = dedupe_key("tenant", "user", &dest(), &meeting, &a);
        assert_eq!(k1, k2);
        assert!(k1.starts_with("planner:tenant:user:plan-1:bucket-1:"));

        let edited = action("action-1", "Send revised proposal");
        assert_ne!(k1, dedupe_key("tenant", "user", &dest(), &meeting, &edited));
    }
}
