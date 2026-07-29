---
title: Security & production checklist
sidebar:
  order: 2
---

Indelible fails closed in production: it refuses to boot with development secrets
and defaults egress protections on. This page covers the settings that matter for a
safe self-hosted deployment.

## Secrets

Generate strong, unique secrets and keep them out of version control:

```bash
openssl rand -base64 32   # JWT_SECRET: signs short-lived access tokens
openssl rand -base64 32   # CSRF_SECRET: signs CSRF tokens
openssl rand -base64 32   # AUTH_CREDENTIAL_KEY: encrypts stored integration tokens
```

- `JWT_SECRET` / `CSRF_SECRET`: required in production; the API will not start with
  the development defaults. To rotate `JWT_SECRET`, deploy the new value in a
  low-traffic window; outstanding access tokens become invalid and clients
  transparently re-authenticate via the refresh cookie.
- `AUTH_CREDENTIAL_KEY`: required in production whenever an integration (e.g. Notion)
  is configured. It encrypts the OAuth tokens Indelible stores on a user's behalf.
  Set the **same** value on `ind-api` and `ind-worker`.

## Accounts

After creating your account(s), close registration:

```yaml
AUTH_ALLOW_SIGNUPS: 'false'
```

The first account on an empty instance can always be created, so you can lock
signups immediately after onboarding yourself.

## Network exposure

- **TLS**: terminate HTTPS at a reverse proxy (Caddy, nginx, Traefik) in front of
  the API. Set `IND_BASE_URL`, `FRONTEND_URL`, and `CORS_ORIGINS` to your real
  `https://` origins. The mobile apps default to HTTPS and show an explicit
  unencrypted-connection warning before they'll talk to a plain-`http://` server;
  see [Use the mobile apps](/docs/how-to/mobile-apps/).
- **`TRUSTED_PROXIES`**: when running behind a proxy or CDN, set this to the
  proxy's IPs/CIDRs so rate limiting and audit logs see the real client IP. Leave it
  empty when the API is exposed directly, otherwise a client can spoof
  `X-Forwarded-For` to evade auth rate limits.
- **`CORS_ORIGINS`**: lock this to the exact origin(s) serving your web app. Never
  use a wildcard.

## Server-side fetch protections

Indelible fetches user-supplied URLs (article capture, feeds, the renderer, AI
providers, webhook delivery). Two guards keep those fetches from reaching your
internal network:

- **`EGRESS_ALLOW_PRIVATE_TARGETS`**: defaults **off** in production. It blocks
  fetches to private, loopback, link-local, and cloud-metadata addresses. Only set
  it `true` if you must reach a local renderer/feed/AI endpoint, and network-isolate
  those services if you do. Set it consistently on `ind-api`, `ind-worker`, and
  `ind-renderer`.
- **`WEBHOOKS_ALLOW_PRIVATE_TARGETS`**: defaults off; only honored in addition to
  the egress guard. Leave off unless you deliberately deliver webhooks to a private
  endpoint.

## Email ingestion

If you enable inbound email (`EMAIL_INGEST_PROVIDER=resend`), set
`EMAIL_INGEST_WEBHOOK_SECRET` so Indelible can verify that inbound webhooks really
came from your provider, and keep `RESEND_API_KEY` secret.

## Object storage

- Keep the bucket **private**. By default (`ASSET_SERVING_MODE=passthrough`) assets
  are streamed through the API, so the object store never needs to be reachable
  from a browser at all. Set `ASSET_COOKIE_SECRET` (64-character hex); the API
  refuses to start in production without it in that mode.
- `ASSET_SERVING_MODE=presigned` makes asset downloads redirect to short-lived
  signed URLs at `S3_ENDPOINT`, offloading download bandwidth to the object
  store. Only use it when that endpoint is reachable from your users' browsers
  and terminates TLS (for example real AWS S3). Uploads go through the API
  either way.
- Restrict the S3 credentials to the single Indelible bucket.

## Backups

Back up two things together:

1. **PostgreSQL**: all metadata, accounts, highlights, and reading state.
2. **Object storage**: the archived content itself (the bucket).

A database restore without its matching bucket (or vice versa) leaves dangling
references. Snapshot both on the same schedule.

## Production checklist

- [ ] `IND_ENV=production`
- [ ] Strong `JWT_SECRET` and `CSRF_SECRET` (not the dev defaults)
- [ ] `AUTH_CREDENTIAL_KEY` set on api + worker if any integration is configured
- [ ] `IND_BASE_URL`, `FRONTEND_URL`, `CORS_ORIGINS` set to your real HTTPS origins
- [ ] TLS terminated at the proxy; `TRUSTED_PROXIES` set to the proxy ranges
- [ ] `AUTH_ALLOW_SIGNUPS=false` after creating your account(s)
- [ ] `EGRESS_ALLOW_PRIVATE_TARGETS` left off (or services network-isolated if on)
- [ ] Object storage bucket private; S3 credentials scoped to that bucket
- [ ] `EMAIL_INGEST_WEBHOOK_SECRET` set if inbound email is enabled
- [ ] PostgreSQL + object-storage backups scheduled together
