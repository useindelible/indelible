---
title: Configuration
sidebar:
  order: 1
---

Configuration is via environment variables. This page covers the variables most
self-hosters need. Beyond these, the services read internal tuning knobs (rate
limits, feed scheduling, retention, background job concurrency) that have sensible
defaults and rarely need changing. Set shared variables identically on `ind-api`
and `ind-worker` unless noted.

## Core

| Variable | Required | Description |
| --- | --- | --- |
| `DATABASE_URL` | yes | PostgreSQL connection string (database must have pgvector available) |
| `IND_ENV` | no | `development` (default) or `production`. Production enforces real secrets and stricter egress defaults |
| `IND_HOST` | no | API bind address (default `0.0.0.0`) |
| `IND_PORT` | no | API listen port. The published image listens on `8080` |
| `RUST_LOG` | no | Log verbosity, e.g. `info` (default) or `debug` |
| `IND_BASE_URL` | yes (production) | Public URL of your instance. Defaults to `http://localhost:38473` for local development. In production it must be an `https` URL, and the API refuses to boot otherwise: the refresh cookie is `Secure`, so browsers discard it over plain HTTP |
| `WEB_ROOT` | no | Directory holding the built web app, served on the API's own origin (default `./web`; the published image sets `/app/web`). Ignored when the directory is absent |
| `S3_ENABLED` | no | Turns on object storage. Defaults to `false`, but is enabled automatically when `S3_ENDPOINT` is set. Archived content and uploads require object storage |
| `S3_ENDPOINT` / `S3_ACCESS_KEY` / `S3_SECRET_KEY` / `S3_BUCKET` / `S3_REGION` / `S3_FORCE_PATH_STYLE` | yes | S3-compatible object storage for archived content. Set on `ind-api`, `ind-worker`, and `ind-renderer` |
| `UPLOAD_MAX_BYTES` | no | Maximum upload size for files saved to the library (default 50 MiB) |
| `IMPORT_UPLOAD_MAX_BYTES` | no | Maximum total size of a Readwise import bundle, including optional OPML (default 200 MiB) |

## Object storage

| Variable | Required | Description |
| --- | --- | --- |
| `ASSET_SERVING_MODE` | no | How asset downloads are served. API responses always contain API URLs; the mode only changes how those URLs answer. `passthrough` (default) streams the bytes through the API, so the object store needs no browser-reachable address; `presigned` redirects downloads to short-lived signed URLs at `S3_ENDPOINT`, which must then be reachable from your users' browsers. Uploads go through the API in both modes |
| `ASSET_COOKIE_SECRET` | yes (production) | 64-character hex secret signing asset cookies for the default `passthrough` mode. The API refuses to start in production without it unless you switch to `presigned` |

## Auth and web origin

| Variable | Required | Description |
| --- | --- | --- |
| `JWT_SECRET` | yes (production) | Signs short-lived access tokens (15 min). To rotate, deploy the new secret in a low-traffic window; clients transparently re-auth via the refresh cookie |
| `CSRF_SECRET` | yes (production) | Signs CSRF tokens for cookie-based endpoints |
| `FRONTEND_URL` | yes (production) | Origin serving the web interface. `ind-api` serves it on its own origin, so this is normally the same URL as `IND_BASE_URL`. Must be `https` and must not be the development default in production |
| `EXTENSION_REDIRECT_URIS` | for Edge, forks, or private builds | Comma-separated exact callback URLs returned by `identity.getRedirectURL('indelible')`. Indelible includes the official Chrome and Firefox callbacks by default. Setting this variable replaces those defaults, so include every browser you distribute. HTTPS is required; wildcards, credentials, query strings, fragments, malformed URLs, and duplicates make the API refuse to start |
| `CORS_ORIGINS` | yes (production) | Comma-separated allowed browser origins. List every hostname you reach the instance by (a LAN name and a tailnet name, say); session refresh accepts any of them. Must not be the development default in production |
| `COOKIE_DOMAIN` | no | Cookie domain when the API and web app share a parent domain |
| `AUTH_ALLOW_SIGNUPS` | no | Defaults to `false`. The first account on an empty instance can always be created; set `true` temporarily to allow additional registrations |
| `AUTH_CREDENTIAL_KEY` | conditional | 32-byte base64 key encrypting stored integration OAuth tokens. Required in production whenever an integration is configured. Generate: `openssl rand -base64 32` |

## Single sign-on

| Variable | Required | Description |
| --- | --- | --- |
| `OIDC_ENABLED`, `OIDC_ISSUER_URL`, `OIDC_CLIENT_ID`, `OIDC_CLIENT_SECRET`, `OIDC_PROVIDER_NAME`, `OIDC_SCOPES`, `OIDC_AUTO_CREATE_USERS` | no | Generic OIDC SSO (authentik, ZITADEL, …) |
| `GOOGLE_CLIENT_ID` / `GOOGLE_CLIENT_SECRET` | no | Google sign-in (both required together) |
| `APPLE_CLIENT_ID` / `APPLE_TEAM_ID` / `APPLE_KEY_ID` / `APPLE_PRIVATE_KEY_PEM` | no | Sign in with Apple (all required together) |

## Renderer

| Variable | Required | Description |
| --- | --- | --- |
| `RENDERER_URL` | no | URL the worker uses to reach the renderer (default `http://127.0.0.1:3100`). Set to your renderer service, e.g. `http://renderer:3100` |
| `RENDERER_HOST` / `RENDERER_PORT` | no | Renderer bind address/port (default `127.0.0.1:3100`; the image binds `0.0.0.0:3100`). Set on `ind-renderer` |
| `CHROMIUM_PATH` | no | Override the Chromium binary path (auto-detected in the image) |

## Email ingestion

| Variable | Required | Description |
| --- | --- | --- |
| `EMAIL_FEED_DOMAIN` | no | Domain for newsletter feed addresses (`<token>@<domain>`) |
| `EMAIL_LIBRARY_DOMAIN` | no | Domain for one-off library forward addresses |
| `EMAIL_INGEST_PROVIDER` | no | Inbound email provider (`resend`) |
| `EMAIL_INGEST_WEBHOOK_SECRET` | conditional | Verifies inbound email webhooks; required when email ingestion is enabled |
| `RESEND_API_KEY` | conditional | Resend API key for inbound email |

## Integrations

| Variable | Required | Description |
| --- | --- | --- |
| `NOTION_CLIENT_ID` / `NOTION_CLIENT_SECRET` / `NOTION_REDIRECT_URL` | no | Notion OAuth app for library export/sync (requires `AUTH_CREDENTIAL_KEY` in production) |

## Mila (AI assistant and semantic search)

Mila powers the AI assistant and semantic search, and is off by default. These
platform variables set availability and the default provider. Each user enters
their own provider API key in the app, stored encrypted in the database, and
embeddings for semantic search are generated per user with that key. There is no
platform-wide API key variable.

| Variable | Required | Description |
| --- | --- | --- |
| `MILA_ENABLED` | no | Set `true` to enable Mila and semantic search. Off by default |
| `MILA_CHAT_API_BASE` | no | OpenAI-compatible chat endpoint (default `https://api.openai.com/v1`) |
| `MILA_CHAT_MODEL` | no | Chat model (default `gpt-4.1-mini`) |
| `MILA_EMBEDDING_API_BASE` | no | OpenAI-compatible embeddings endpoint (default `https://api.openai.com/v1`) |
| `MILA_EMBEDDING_MODEL` | no | Embedding model (default `text-embedding-3-small`) |
| `MILA_MODEL_CONTEXT_WINDOW` | no | Chat model token window (default `12000`). Set to match your model |
| `MILA_SUMMARY_MAX_OUTPUT_TOKENS` | no | Maximum summary completion tokens (default `1024`) |
| `MILA_TAGS_MAX_OUTPUT_TOKENS` | no | Maximum tag completion tokens (default `1024`) |
| `MILA_ENTITIES_MAX_OUTPUT_TOKENS` | no | Maximum entity-extraction completion tokens (default `2000`) |
| `MILA_CHAT_MAX_OUTPUT_TOKENS` | no | Maximum interactive chat completion tokens (default `1024`) |

The embedding vector dimension is fixed to match the pgvector storage the database
was built for. The binary rejects a mismatched `MILA_EMBEDDING_DIM` at startup, so
choose an embedding model that matches the built-in dimension rather than overriding it.

## Text-to-speech

Read-aloud is enabled by default but produces audio only once a provider key is
set. Supported providers are DashScope and Unreal Speech.

| Variable | Required | Description |
| --- | --- | --- |
| `TTS_ENABLED` | no | Enabled by default. Set `false` to disable read-aloud |
| `TTS_DASHSCOPE_API_KEY` | conditional | DashScope API key, required to use DashScope voices |
| `TTS_DASHSCOPE_API_BASE` | no | Override the DashScope endpoint |
| `TTS_UNREAL_SPEECH_API_KEY` | conditional | Unreal Speech API key, required to use Unreal Speech voices |
| `TTS_UNREAL_SPEECH_API_BASE` | no | Override the Unreal Speech endpoint |

## Network and proxy

| Variable | Required | Description |
| --- | --- | --- |
| `EGRESS_ALLOW_PRIVATE_TARGETS` | no | Server-side fetches of user URLs refuse private/loopback/link-local/metadata targets. Defaults on in development, off in production. Set `true` only to reach a local renderer/feed/AI endpoint, and network-isolate those services. Set on `ind-api`, `ind-worker`, and `ind-renderer` |
| `WEBHOOKS_ALLOW_PRIVATE_TARGETS` | no | Allow webhook delivery to private endpoints; off by default, honored in addition to the egress guard |
| `TRUSTED_PROXIES` | no | Comma-separated IPs/CIDRs of reverse proxies whose `X-Forwarded-For` / `X-Real-IP` are trusted. Leave empty when exposed directly, or clients can spoof their IP to evade rate limits |
