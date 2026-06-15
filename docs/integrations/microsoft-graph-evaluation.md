# Microsoft Graph Export Evaluation

Date: 2026-06-15
Branch: `feat/clawscribe-productization-auth-theme-exports`
Product: ClawScribe
Baseline: `docs/productization/phase0-inventory.md`

## Summary

Microsoft Graph export is feasible for ClawScribe as a delegated, signed-in-user
integration. It should be separate from OpenAI/OpenClaw auth and should not use
application permissions, app-only tokens, Teams Graph meeting APIs, transcript
APIs, or a tenant-admin-only design.

The safe sprint shape is documentation and mocked exporter tests first. The repo
does not currently contain MSAL, a Microsoft account connection model, or a
token storage abstraction proven for Graph. Adding live Graph code now would
force new credential and consent choices without real tenant validation.

## Delegated Auth Model

Recommended access pattern:

1. The user signs in to Microsoft separately from OpenAI or OpenClaw.
2. ClawScribe requests delegated Microsoft Graph scopes for export only.
3. Graph calls run as the signed-in user and can only access resources that user
   can access.
4. Tokens are never logged. Logs may record `connected`, `expired`,
   `consent_required`, `access_denied`, `not_found`, and Graph
   `client-request-id`, but not bearer tokens, refresh tokens, auth codes, or
   authorization URLs containing codes.

Initial delegated scopes:

- `User.Read`: sign in and read the profile of the signed-in Microsoft user.
- `Notes.Create`: create OneNote pages/sections/notebooks on behalf of the
  signed-in user.
- `Tasks.ReadWrite`: create, read, update, and delete the signed-in user's
  tasks and task lists, including shared tasks the user can access.
- `offline_access`: request refresh-token capability only if ClawScribe needs
  background or post-close retry behavior. For foreground-only export, prefer
  access-token-only and prompt the user to reconnect when needed.

These are delegated permissions that Microsoft lists as not requiring admin
consent by default, but tenant policy can still block user consent.

Sources:

- Microsoft identity platform permissions and consent overview:
  https://learn.microsoft.com/entra/identity-platform/permissions-consent-overview
- Microsoft Graph permissions reference:
  https://learn.microsoft.com/graph/permissions-reference
- Microsoft Graph best practices:
  https://learn.microsoft.com/graph/best-practices-concept

## Consent And Failure Behavior

ClawScribe should treat Microsoft account connection as an optional export
capability, not as app startup auth.

Expected states:

- `not_connected`: no Microsoft account is connected.
- `connecting`: the user is in the Microsoft sign-in/consent flow.
- `connected`: ClawScribe has a usable delegated Graph session.
- `consent_required`: Graph scopes are missing or revoked.
- `tenant_blocked`: the tenant does not allow user consent or blocks the app.
- `access_denied`: the user is signed in but lacks access to the selected
  OneNote section, Planner plan, Planner bucket, or assigned user.
- `expired`: token refresh failed or no refresh token is available.

User-consent behavior:

- If Microsoft shows the consent prompt and the user approves, ClawScribe enables
  export destinations selected by the user.
- If the user cancels or denies consent, ClawScribe keeps recording, summary, and
  OpenClaw handoff working. Export remains disabled and can be retried from the
  destination-specific settings.
- If incremental consent is used, request `Notes.Create` only when enabling
  OneNote and `Tasks.ReadWrite` only when enabling Planner.

Tenant-blocked behavior:

- If the tenant blocks user consent, ClawScribe should report that the Microsoft
  tenant requires administrator approval for this app. It should not claim the
  user entered wrong credentials.
- The product should not require an admin-consent-only setup for the default
  design. Admin consent can be an enterprise deployment option later, using the
  same delegated scopes, not app-only permissions.
- A tenant-blocked result should not disable local recording, transcript
  processing, OpenAI auth, or OpenClaw auth.

Graph call failure behavior:

- `401`: mark Microsoft session expired or invalid, do not retry with the same
  token indefinitely, prompt reconnect.
- `403`: show access denied or tenant policy blocked depending on the error
  context. Do not retry blindly.
- `404`: selected destination no longer exists or is not visible to the user.
  Clear only that destination binding after user confirmation.
- `429`: respect `Retry-After` when present. If absent, use bounded exponential
  backoff. Keep retry metadata local and do not duplicate exports.
- `503`: use the same bounded backoff family as `429`, but surface service
  unavailable if attempts are exhausted.

## Architecture Recommendation

Keep Microsoft Graph integration behind new self-contained modules before any
settings UI wiring:

```text
graph-auth/
  microsoft account connection state
  delegated scope requests
  token refresh abstraction

graph-client/
  authenticated REST client
  correlation ids
  retry/backoff policy
  sanitized error mapping

exports/
  oneNote exporter
  planner exporter
  exports.json idempotency ledger
  mock Graph transport tests
```

The UI should eventually expose Microsoft as an export account, not an auth
provider for summarization. OpenAI API key, custom OpenAI-compatible endpoint,
OpenClaw managed endpoint, and Microsoft Graph export should stay independent.

Suggested local files when implemented:

- Account/session config in the Tauri app config directory.
- Destination config in a Microsoft export config file.
- Export idempotency in each recording folder as `exports.json`.

`exports.json` should not contain tokens. It may contain destination IDs,
dedupe keys, Graph resource IDs, timestamps, status, and sanitized error codes.

Example ledger shape:

```json
{
  "schema": "clawscribe.exports.v1",
  "meetingId": "2026-06-15-weekly-sync",
  "destinations": {
    "onenote": {
      "dedupeKey": "onenote:section-id:meeting-hash",
      "status": "succeeded",
      "pageId": "graph-page-id",
      "webUrl": "https://...",
      "lastAttemptAt": "2026-06-15T10:20:00Z"
    },
    "planner": {
      "dedupeKey": "planner:plan-id:bucket-id:action-item-hash",
      "status": "partial_failure",
      "createdTaskIds": ["graph-task-id-1"],
      "failedItems": [
        {
          "localActionId": "action-2",
          "status": 403,
          "code": "AccessDenied"
        }
      ]
    }
  }
}
```

## Implementation Feasibility

Feasible this sprint without external credentials:

- Mocked Graph transport and error mapper.
- HTML builder for OneNote page payloads.
- Planner task request builder.
- Idempotency ledger parser/writer for `exports.json`.
- Unit tests for 401, 403, 404, 429, duplicate retry, and partial failure.

Not safe this sprint without credentials or tenant validation:

- Production Microsoft sign-in UX.
- Real MSAL token cache integration.
- OS credential storage choice for Microsoft refresh tokens.
- Live OneNote or Planner export.
- Tenant-blocked consent verification against a real Entra tenant.

## Open Questions

- Which Microsoft app registration will ClawScribe use for production:
  single-tenant customer-owned, multi-tenant publisher-owned, or operator-owned?
- Should export run only from an explicit user click, or also automatically after
  recording post-processing completes?
- Should `offline_access` be enabled by default, or only after the user enables
  automatic background export/retry?
- Which UX will collect Planner `planId` and `bucketId` safely without requiring
  broad group discovery scopes?
