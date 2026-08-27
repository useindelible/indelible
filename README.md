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
mkdir indelible && cd indelible
curl -fsSLO https://github.com/useindelible/indelible/releases/latest/download/docker-compose.yml
curl -fsSL https://github.com/useindelible/indelible/releases/latest/download/example.env -o .env

{
  echo "POSTGRES_PASSWORD=$(openssl rand -hex 16)"
  echo "MINIO_ROOT_PASSWORD=$(openssl rand -hex 16)"
  echo "JWT_SECRET=$(openssl rand -hex 32)"
  echo "CSRF_SECRET=$(openssl rand -hex 32)"
  echo "ASSET_COOKIE_SECRET=$(openssl rand -hex 32)"
  echo "AUTH_CREDENTIAL_KEY=$(openssl rand -base64 32)"
} >> .env

docker compose up -d
```

That brings up PostgreSQL, Silo, the renderer, the worker, and the API, which
serves the web interface on the same port. Open http://localhost:38473 and
create the first account. Compose pulls the release images from GHCR; it does
not build Indelible on your machine.

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

## Documentation

Full documentation, including every configuration variable, lives at
[useindelible.com](https://useindelible.com). The source is in `website/`.

## Contributing

Start with [CONTRIBUTING.md](CONTRIBUTING.md), which covers the development
setup, the checks CI runs, and the architecture boundaries backend changes are
reviewed against.

## Licence

[AGPL-3.0](LICENSE).
