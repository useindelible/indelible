---
title: Export to Notion
sidebar:
  order: 5
---

Connect Notion and Indelible exports your saved documents and highlights into a
managed Notion database. It's a one-way export: Indelible writes to Notion and
does not read your Notion workspace back.

## Connect (cloud)

1. Open **Settings → Integrations → Notion**.
2. Click **Connect Notion** and approve the OAuth request in your Notion workspace.
3. Choose whether to export automatically and how highlights are laid out, or trigger
   an export manually. New saves export in the background once connected.

## Self-hosting: register a Notion app

The hosted service ships with its own Notion app. Self-hosters must register one and
configure these variables. The three Notion OAuth variables are read by `ind-api` only;
`AUTH_CREDENTIAL_KEY` must be set identically on both `ind-api` and `ind-worker`:

| Variable | Set on | Value |
| --- | --- | --- |
| `NOTION_CLIENT_ID` | `ind-api` | From your Notion integration (OAuth) |
| `NOTION_CLIENT_SECRET` | `ind-api` | From your Notion integration |
| `NOTION_REDIRECT_URL` | `ind-api` | `https://your-instance/api/v1/integrations/notion/callback` |
| `AUTH_CREDENTIAL_KEY` | `ind-api` + `ind-worker` | 32-byte base64 key that encrypts stored Notion tokens (required in production) |

Create the integration at [notion.so/my-integrations](https://www.notion.so/my-integrations),
set its redirect URI to the value above, then restart your instance. The Notion
option appears in Settings once `NOTION_CLIENT_ID` is set. See the
[Configuration reference](/docs/reference/configuration/) for details.
