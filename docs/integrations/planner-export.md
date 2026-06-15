# Planner Export Design

Date: 2026-06-15
Product: ClawScribe

## Feasibility

Planner export is feasible through Microsoft Graph delegated permissions when
the signed-in user can access the target plan and bucket. The least-privileged
scope for creating Planner tasks is `Tasks.ReadWrite`; ClawScribe should also
request `User.Read` for Microsoft sign-in and `offline_access` only if
background retry needs refresh-token behavior.

Task creation endpoint:

```http
POST https://graph.microsoft.com/v1.0/planner/tasks
Content-Type: application/json
Authorization: Bearer <access-token>
```

Planner task creation requires an existing `planId`. In practice ClawScribe
should also require an explicit `bucketId` because tasks exported from meeting
action items need a predictable destination column. Microsoft documents that
tasks cannot be created without plans, and the task create API requires
`planId` in the request body.

Sources:

- Create plannerTask:
  https://learn.microsoft.com/graph/api/planner-post-tasks?view=graph-rest-1.0
- Use the Planner REST API:
  https://learn.microsoft.com/graph/api/resources/planner-overview?view=graph-rest-1.0
- Microsoft Graph permissions reference:
  https://learn.microsoft.com/graph/permissions-reference

## Required Destination Configuration

Minimum destination config:

```json
{
  "enabled": true,
  "planId": "planner-plan-id",
  "bucketId": "planner-bucket-id",
  "defaultAssignment": "none",
  "reviewBeforeCreate": true
}
```

`planId` and `bucketId` should be user selected or pasted from a known Planner
destination in the first implementation. Do not add broad group or directory
discovery scopes just to make destination picking easier.

Optional later fields:

- `labelIds`: Planner category labels to apply.
- `defaultAssigneeUserId`: signed-in user or configured user.
- `priority`: mapped from ClawScribe action priority.
- `dueDatePolicy`: none, extracted date, or manual review.
- `sourceReferencePolicy`: add OneNote page link or local recording reference
  after OneNote export is available.

## Task Mapping

Action item to Planner task:

```json
{
  "planId": "planner-plan-id",
  "bucketId": "planner-bucket-id",
  "title": "Send revised proposal to Contoso",
  "assignments": {}
}
```

Keep task creation conservative:

- Title from action item text, trimmed to a short single-line value.
- No assignment unless a reviewed mapping exists. Extracted speaker names should
  not be auto-mapped to Azure AD users without confirmation.
- No due date unless explicitly extracted and reviewed.
- Put longer meeting context into task details only after a second call with
  etag handling is implemented and tested.

Planner assignments require the assigned user's ID as the dynamic property name
and an `orderHint`. If ClawScribe adds assignments, it must validate the user ID
and use the documented assignment shape.

## Review And Defaults

Default behavior should be review-first:

1. Extract action items locally.
2. Show proposed Planner tasks with title, optional owner text, optional due
   date, and destination bucket.
3. Let the user deselect items and edit titles.
4. Create only approved tasks.
5. Record per-item results in `exports.json`.

No safe default should bulk-create Planner tasks silently after every recording
until a user explicitly enables automatic export and accepts duplicate handling.

## Idempotency

Planner task creation is not naturally idempotent. ClawScribe should use the
recording folder's `exports.json` ledger.

Recommended per-action dedupe key:

```text
planner:{tenantId}:{userId}:{planId}:{bucketId}:{meetingArtifactHash}:{localActionId}:{actionHash}
```

Ledger example:

```json
{
  "schema": "clawscribe.exports.v1",
  "destinations": {
    "planner": {
      "status": "partial_failure",
      "createdTaskIds": ["task-id-1", "task-id-2"],
      "items": {
        "action-1": {
          "dedupeKey": "planner:tenant:user:plan:bucket:meeting:action-1:hash",
          "status": "succeeded",
          "taskId": "task-id-1",
          "lastAttemptAt": "2026-06-15T10:20:00Z"
        },
        "action-2": {
          "dedupeKey": "planner:tenant:user:plan:bucket:meeting:action-2:hash",
          "status": "failed",
          "statusCode": 403,
          "code": "AccessDenied"
        }
      }
    }
  }
}
```

Duplicate retry behavior:

- If an item is already `succeeded`, do not create another task.
- If the previous attempt failed before submit, retry is safe.
- If the previous attempt failed after submit with unknown result, ask the user
  before creating another task.
- If a batch partially succeeds, retry only failed or unknown items, never the
  whole set blindly.

## Error Handling

- `401`: token expired or invalid. Mark Microsoft connection `expired` and ask
  the user to reconnect.
- `403`: consent missing, tenant policy blocked, user not authorized for the
  plan, or Planner service limit exceeded. Do not retry unless the error is a
  known transient limit case with backoff.
- `404`: `planId`, `bucketId`, or assigned user no longer exists or is not
  visible to the signed-in user. Disable that destination until reviewed.
- `409` or `412`: relevant for later task detail update/delete flows using
  etags. Re-read resource and resolve conflict before patching.
- `429`: follow `Retry-After` when present. If absent, use bounded exponential
  backoff and retry only items still not created.

## Sprint Boundary

Safe implementation without live credentials:

- Mock Graph task creation transport.
- Request builder validation requiring `planId`, `bucketId`, and non-empty
  title.
- Per-item idempotency ledger write/read tests.
- Partial failure handling tests.

Not safe without tenant validation:

- Live Microsoft sign-in.
- Broad plan/bucket discovery.
- Automatic background Planner export.
- Assignment auto-resolution from transcript speaker labels.
