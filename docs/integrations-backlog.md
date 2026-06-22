# Integrations backlog

Candidate exports/services beyond the current OneNote / Planner / To Do /
Calendar (Microsoft 365) and Confluence (Atlassian). Ranked by value vs. effort.
Each new service is a Graph **scope** — gate behind the user enabling it in
Settings → Add-ons, and keep least-privilege.

## Microsoft 365 — next
- [ ] **OneDrive / SharePoint file export** — chosen next. Export reviewed
  meeting notes and transcripts as `.docx` first, with PDF after the render path
  is reliable. Re-adds `Files.ReadWrite` (trimmed in the least-privilege pass),
  so it must stay behind explicit Add-ons enablement. Target OneDrive/SharePoint
  cloud files only, not generic SMB/network file shares or broad local-file sync.

## Microsoft 365 — shelved / not next
- [ ] **Email recap to attendees (Outlook `Mail.Send`)** — shelved for now.
  Attendees are already captured from Calendar, but "send summary + action items
  to everyone invited" creates a new `Mail.Send` consent surface and outbound
  communication risk. Do not bundle this with file export scope work.

## Microsoft 365 — more effort / heavier consent
- [ ] **Post recap to a Teams channel/chat** — for Teams meetings, post the summary
  back. Heavier consent (`ChannelMessage.Send` / `Chat.ReadWrite`), fiddlier API.

## Skip / low priority
- Loop components, calendar follow-up events (`Calendars.ReadWrite`), Viva/Bookings
  — niche or API-immature.

## Shipped
- [x] **Microsoft To Do** — personal action items (counterpart to Planner's team
  tasks). Uses the existing `Tasks.ReadWrite` scope and the `/me/todo/...`
  endpoints.

## Notes
- Every remaining addition = a new consent scope. Gate each behind explicit
  enablement.
- Patterns already in place: OneDrive / SharePoint Files = another Export-menu
  item; email / Teams = a new "Send recap" action and a separate consent review.
