# Microsoft Export Mock Test Plan

Date: 2026-06-15
Product: ClawScribe

This plan covers Microsoft Graph export behavior that can be tested without
real Microsoft credentials. Tests should use a fake Graph transport and static
fixture recording folders. No test should log or snapshot bearer tokens.

## Test Fixtures

Fixture meeting folder:

```text
recording-folder/
  metadata.json
  transcripts.json
  summary.md
  exports.json
```

Mock Graph transport inputs:

- Method, URL, headers with token redacted, body hash, and request attempt.
- Configurable response queue per URL.
- Captured `client-request-id` and sanitized error mapping.

Assertions shared by all tests:

- No access token, refresh token, auth code, or full authorization URL is logged.
- `exports.json` is updated atomically.
- A succeeded export is not submitted twice.
- Partial failures preserve successful item IDs.

## Required Mock Cases

### 401 Invalid Or Expired Token

Setup:

- OneNote or Planner mock returns `401 Unauthorized`.

Expected:

- Export status becomes `failed_auth`.
- Microsoft connection state becomes `expired`.
- User-facing message says reconnect Microsoft account.
- No retry with the same token beyond the configured single refresh attempt.
- `exports.json` contains sanitized status and Graph error code only.

### 403 Forbidden

Setup:

- OneNote section or Planner plan mock returns `403 Forbidden`.

Expected:

- Export status becomes `access_denied` or `tenant_blocked` depending on mapped
  error body.
- No automatic retry.
- Recording, summary, OpenAI auth, and OpenClaw handoff remain unaffected.
- `exports.json` records destination failure without tokens.

### 404 Destination Missing

Setup:

- OneNote section mock returns `404 Not Found`, or Planner task create returns
  `404` for stale `planId` or `bucketId`.

Expected:

- Export status becomes `destination_not_found`.
- Destination config is marked needs review, not silently deleted.
- User can choose a new section/plan/bucket.
- Retry is blocked until destination is reviewed.

### 429 Throttled

Setup:

- First mock response returns `429 Too Many Requests` with `Retry-After: 2`.
- Second mock response returns `201 Created`.

Expected:

- Export waits according to retry policy or schedules a retry at that time in a
  deterministic test clock.
- No duplicate item is created.
- Attempt count increments.
- `exports.json` records final success and may keep sanitized retry metadata.

Variant:

- `429` without `Retry-After`.

Expected:

- Bounded exponential backoff is used.
- Attempts stop at the configured maximum.

### Duplicate Retry

Setup:

- `exports.json` already contains a succeeded OneNote page or Planner task for
  the same dedupe key.

Expected:

- Exporter does not call Graph.
- Existing resource ID and URL are returned to the UI.
- Status remains `succeeded`.

Variant:

- Previous attempt is `unknown_after_submit`.

Expected:

- Exporter does not create a second page/task automatically.
- User review is required before duplicate creation.

### Partial Failure

Setup:

- Planner export contains three action items.
- Mock responses: item 1 `201 Created`, item 2 `403 Forbidden`, item 3
  `201 Created`.

Expected:

- Overall Planner export status is `partial_failure`.
- Created task IDs for items 1 and 3 are preserved.
- Item 2 records sanitized failure.
- Retry attempts only item 2.
- No duplicate tasks are created for items 1 and 3.

## Builder Tests

OneNote builder:

- Produces UTF-8 well-formed XHTML.
- Escapes transcript, summary, participant, and title text.
- Splits long transcript content before the 4 MB Graph REST request limit.
- Rejects unsupported active content such as scripts and forms.

Planner builder:

- Rejects missing `planId`.
- Rejects missing `bucketId`.
- Rejects empty titles.
- Defaults to no assignment unless a reviewed user ID exists.
- Produces deterministic local action IDs and dedupe keys.

## Manual Validation Later

When credentials and a test tenant are available:

- Connect a Microsoft account with `User.Read` only.
- Incrementally enable OneNote and consent to `Notes.Create`.
- Create a page in default `sectionName=ClawScribe`.
- Create a page in an explicit section ID.
- Incrementally enable Planner and consent to `Tasks.ReadWrite`.
- Create reviewed tasks in a known plan/bucket.
- Test a tenant where user consent is disabled and confirm the
  `tenant_blocked` UX.
- Confirm no tokens appear in app logs, Tauri logs, crash reports, or
  `exports.json`.
