# Indelible

Open-source, self-hosted read-it-later and knowledge archiver. Articles,
newsletters, PDFs, and EPUBs are captured in full and stored in your own
library, permanently. Links rot; your library does not.

## What it does

- **Full-content archiving.** Pages are fetched, extracted, and stored at save
  time, with a headless Chromium renderer for the hard ones. The original can
  disappear; your copy stays readable.
- **A focused reader.** Clean typography, highlights, notes, and reading
  progress that syncs across web and mobile.
- **Search that understands you.** Full-text and semantic search across
  everything you have saved.
- **Mila, an optional AI assistant.** Ask questions across your library and
  summarize long reads, using your own provider key.
- **Content in, from anywhere.** Browser extension, personal email-in
  addresses, RSS feeds, and file uploads.
- **Your tools, connected.** Sync to Obsidian and Notion.

## Quick start

You need Docker and a machine with a few GB of RAM.

```bash
git clone https://github.com/useindelible/indelible.git
cd indelible
docker compose up -d
```

That brings up PostgreSQL, MinIO, the renderer, the worker, and the API, which
serves the web interface on the same port. Open http://localhost:38473 and
create the first account.

The compose file at the root is for **local development only**: it runs with
throwaway credentials and without TLS. For a real deployment, follow the
[installation guide](https://useindelible.com/docs/self-hosting/install/), which
ships a production compose file, and the
[security checklist](https://useindelible.com/docs/self-hosting/security/).

## Clients

| Client | Where |
| --- | --- |
| Web | Served by `ind-api`, no separate deployment |
| Browser extension | `extension/` (Chrome, Edge, Firefox) |
| Mobile | `mobile/` (Android and iOS, Kotlin Multiplatform) |
| CLI (`ind`) | `backend/apps/ind-cli/` |
| Obsidian plugin | `obsidian/` |

## Documentation

Full documentation, including every configuration variable, lives at
[useindelible.com](https://useindelible.com). The source is in `website/`.

## Contributing

Start with [CONTRIBUTING.md](CONTRIBUTING.md). Architecture rules for backend
changes are in [contributing.agents.md](contributing.agents.md).

## Licence

[AGPL-3.0](LICENSE).
