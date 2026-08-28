<img alt="An archived article open in the Indelible reader, with Mila's summary, extracted metadata and reading progress in the detail panel beside it" src="https://assets.useindelible.com/readme/reader.webp" width="100%" />

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

<img alt="Four Indelible screens: the Home dashboard, full-text and semantic search, Collections, and the RSS feed" src="https://assets.useindelible.com/readme/wall.webp" width="100%" />

Mila is optional and runs on your own provider key, against any
OpenAI-compatible endpoint. It summarizes, tags, and answers questions across
the library.

<img alt="Indelible's Mila settings, showing the provider toggle, library indexing progress, and the editable summary and tag prompt presets" src="https://assets.useindelible.com/readme/mila.webp" width="100%" />

## Quick start

You need Docker and a machine with a few GB of RAM.

```bash
mkdir indelible && cd indelible && curl -fsSLO https://github.com/useindelible/indelible/releases/latest/download/install.sh && sh install.sh
```

The installer checks the downloads against the release checksums, generates
the required secrets, and brings up PostgreSQL, Silo, the renderer, the worker,
and the API, which serves the web interface on the same port. Open
http://localhost:38473 and create the first account. Compose pulls the release
images from GHCR; it does not build Indelible on your machine.

This quickstart runs on localhost without TLS. For a deployment reachable by
other people, follow the
[installation guide](https://useindelible.com/docs/self-hosting/install/) and
[security checklist](https://useindelible.com/docs/self-hosting/security/).

## Development

Contributors build the images from the checked-out source using the root
Compose file:

```bash
git clone https://github.com/useindelible/indelible.git
cd indelible
docker compose up -d --build
```

## Clients

| Client | Where |
| --- | --- |
| Web | Served by `ind-api`, no separate deployment |
| Browser extension | [Chrome and Edge](https://chromewebstore.google.com/detail/indelible/jidilhjojlgndbpeooeeceohmkedooef), [Firefox](https://addons.mozilla.org/en-GB/firefox/addon/indelible/), or [source](extension/) |
| Mobile | `mobile/` (Android and iOS, Kotlin Multiplatform) |
| Obsidian plugin | `obsidian/` |

Android and iOS are one Kotlin Multiplatform codebase, sharing the library and
reading position with the web.

<img alt="Three Indelible phone screens: the daily home view, an article being highlighted with the native selection toolbar, and saving a URL" src="https://assets.useindelible.com/readme/phones.webp" width="100%" />

The extension saves the page you are on and archives its full text at save
time, without leaving the tab. Highlights travel both ways: text you mark on
the live page shows up in the reader, and highlights made in the reader are
re-anchored onto the original page when you open the extension there again.
Notes attach to a saved page from the extension, or to an individual highlight
in the web app.

<img alt="The Indelible browser extension toolbar injected over an article it has just saved, with a highlight in the page text" src="https://assets.useindelible.com/readme/extension.webp" width="100%" />

## Documentation

Full documentation, including every configuration variable, lives at
[useindelible.com](https://useindelible.com). The source is in `website/`.

## Contributing

Start with [CONTRIBUTING.md](CONTRIBUTING.md), which covers the development
setup, the checks CI runs, and the architecture boundaries backend changes are
reviewed against.

## Licence

[AGPL-3.0](LICENSE).
