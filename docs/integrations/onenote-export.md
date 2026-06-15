# OneNote Export Design

Date: 2026-06-15
Product: ClawScribe

## Feasibility

OneNote export is feasible through Microsoft Graph delegated permissions. The
least-privileged scope for creating pages is `Notes.Create`; ClawScribe should
also request `User.Read` for Microsoft sign-in and `offline_access` only if
background retry needs refresh-token behavior.

The preferred endpoint is a user-scoped OneNote page create call:

```http
POST https://graph.microsoft.com/v1.0/me/onenote/sections/{section-id}/pages
Content-Type: application/xhtml+xml
Authorization: Bearer <access-token>
```

For a simpler first configuration, ClawScribe can use:

```http
POST https://graph.microsoft.com/v1.0/me/onenote/pages?sectionName=ClawScribe
```

That route targets the signed-in user's default notebook and creates the
top-level section if it does not exist. It is convenient for a default, but a
stored `sectionId` is more predictable for enterprise use.

Sources:

- Create OneNote pages:
  https://learn.microsoft.com/graph/onenote-create-page
- Add images and files to OneNote pages:
  https://learn.microsoft.com/graph/onenote-images-files
- Microsoft Graph permissions reference:
  https://learn.microsoft.com/graph/permissions-reference

## Content Strategy

Use editable XHTML for the first pass. Avoid binary attachments and page
snapshots until live Graph validation exists.

Suggested page structure:

```html
<!DOCTYPE html>
<html>
  <head>
    <title>Weekly sync - 2026-06-15</title>
    <meta name="created" content="2026-06-15T10:00:00Z" />
  </head>
  <body>
    <h1>Weekly sync</h1>
    <p><b>Recorded by:</b> ClawScribe</p>
    <h2>Summary</h2>
    <p>Short summary text...</p>
    <h2>Decisions</h2>
    <ul>
      <li>Decision text...</li>
    </ul>
    <h2>Action Items</h2>
    <ul>
      <li>Owner - action item text</li>
    </ul>
    <h2>Transcript</h2>
    <pre>Timestamped transcript excerpt...</pre>
  </body>
</html>
```

OneNote Graph input HTML requirements that matter for ClawScribe:

- UTF-8 encoded and well-formed XHTML.
- Use supported semantic elements: headings, paragraphs, lists, tables, `pre`,
  bold, and italic.
- Do not rely on JavaScript, forms, included CSS, or unsupported HTML.
- Prefer direct editable text over `data-render-src` snapshots because users
  should be able to edit the exported meeting notes.
- Use a multipart library only when binary parts are added later.

## Size Strategy

ClawScribe should keep the first export payload below 4 MB because the Microsoft
Graph REST API request limit applies before the underlying OneNote API's larger
limits. This points to a text-first export:

- Include title, metadata, summary, decisions, action items, and a transcript
  excerpt by default.
- For long transcripts, include a compact transcript section with speaker and
  timestamp summaries, then attach or link local artifacts only in a later phase.
- If raw transcript export is requested, split across multiple pages using a
  deterministic page series title.
- Avoid audio attachments in Graph OneNote export. Local audio paths can be
  sensitive and are not useful once uploaded from another user's machine.

Page splitting recommendation:

```text
Weekly sync - Notes
Weekly sync - Transcript 1
Weekly sync - Transcript 2
```

Each page should have its own idempotency entry in `exports.json`.

## Idempotency

OneNote page creation is not naturally idempotent. ClawScribe should use a local
`exports.json` ledger in the recording folder.

Recommended dedupe key:

```text
onenote:{tenantId}:{userId}:{sectionId}:{meetingArtifactHash}:{pageKind}:{pageIndex}
```

Before creating a page:

1. Load `exports.json`.
2. If the dedupe key is `succeeded`, show the existing OneNote URL or ask the
   user whether to create another copy.
3. If the key is `pending` and fresh, block duplicate in-flight export.
4. If the key is `pending` and stale, retry only after marking a new attempt.
5. If the key failed with a retriable status, retry with the same dedupe key.

Because Graph does not support a client-provided idempotency key for OneNote
page creation, a network timeout after a successful create can produce an
unknown state. The safe default is to mark `unknown_after_submit` and ask before
creating another page.

## Error Handling

- `401`: token expired or invalid. Mark Microsoft connection `expired` and ask
  the user to reconnect.
- `403`: consent missing, tenant policy blocked, or user lacks access to the
  target notebook/section. Show access denied and keep local artifacts intact.
- `404`: stored `sectionId` no longer exists or is not visible. Offer to pick a
  new section or use the default `sectionName=ClawScribe` route.
- `413`: page payload too large. Split transcript pages or export summary-only.
- `429`: follow `Retry-After` if present. If missing, use bounded exponential
  backoff. Do not create duplicate pages while retrying.
- `507`: section page limit reached. Offer another section rather than retrying
  the same section.

## Safe Defaults

- Export only after explicit user action until live behavior is proven.
- Create summary/action pages first; keep raw full transcript optional.
- Never include bearer tokens or auth URLs in logs.
- Store only OneNote page IDs, URLs, destination IDs, dedupe keys, and sanitized
  errors in `exports.json`.
- Keep Microsoft export independent from OpenAI and OpenClaw provider settings.
