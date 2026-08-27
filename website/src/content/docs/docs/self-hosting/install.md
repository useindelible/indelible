---
title: Install with Docker
sidebar:
  order: 1
---

Two paths below. The first gets Indelible running on your own machine in about a
minute so you can try it. The second puts it on a domain with TLS, which is what
you want for anything other people will reach.

Indelible ships three server images: `ind-api` (the HTTP API, which also serves
the web app), `ind-worker` (background jobs), and `ind-renderer` (Chromium
renderer for hard-to-capture pages). They run alongside PostgreSQL with pgvector
and an S3-compatible object store, both included in the compose files below.

## Try it locally (about a minute)

Plain HTTP on `localhost`, for evaluating on your own machine. Do not expose this
to a network: cookies are not `Secure` over HTTP, so sessions break the moment
you put it behind a real hostname anyway. When you like it, jump to the
production path.

```bash
mkdir indelible && cd indelible
curl -fsSLO https://github.com/useindelible/indelible/releases/latest/download/docker-compose.yml
curl -fsSL https://github.com/useindelible/indelible/releases/latest/download/example.env -o .env

# Fill in the generated secrets
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

Those two files come from the latest stable GitHub release. Its `.env` pins all
three Indelible containers to that release, and Compose pulls the images from
GHCR instead of building them on your machine.

Open <http://localhost:38473> and create your account. That account is yours
alone: signups close automatically once it exists, so there is nothing to switch
off afterwards.

## Put it on a domain (about five minutes)

You need a hostname pointing at the machine and a TLS terminator. The API
refuses to boot in production without `https`, because the refresh cookie is
`Secure` and browsers silently discard it over plain HTTP, which kills every
session on its first refresh.

Create `.env` next to the compose file:

```bash
INDELIBLE_HOST=indelible.example.com

POSTGRES_PASSWORD=change-me
MINIO_ROOT_PASSWORD=change-me-too

JWT_SECRET=paste-openssl-rand-hex-32
CSRF_SECRET=paste-openssl-rand-hex-32
ASSET_COOKIE_SECRET=paste-openssl-rand-hex-32
AUTH_CREDENTIAL_KEY=paste-openssl-rand-base64-32
```

Generate the four secrets with:

```bash
openssl rand -hex 32     # JWT_SECRET, CSRF_SECRET, ASSET_COOKIE_SECRET
openssl rand -base64 32  # AUTH_CREDENTIAL_KEY, exactly 32 bytes
```

Then the compose file. Every service reads the same `.env`, so there is one
place to edit:

```yaml
x-app-env: &app-env
  IND_ENV: production
  DATABASE_URL: postgres://indelible:${POSTGRES_PASSWORD}@postgres:5432/indelible
  S3_ENDPOINT: http://silo:9000
  S3_REGION: us-east-1
  S3_BUCKET: indelible
  S3_ACCESS_KEY: indelible
  S3_SECRET_KEY: ${MINIO_ROOT_PASSWORD}
  S3_FORCE_PATH_STYLE: 'true'
  AUTH_CREDENTIAL_KEY: ${AUTH_CREDENTIAL_KEY}

services:
  postgres:
    image: pgvector/pgvector:pg18
    restart: unless-stopped
    environment:
      POSTGRES_USER: indelible
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}
      POSTGRES_DB: indelible
    volumes:
      - pgdata:/var/lib/postgresql
    healthcheck:
      test: ['CMD-SHELL', 'pg_isready -U indelible -d indelible']
      interval: 5s
      timeout: 5s
      retries: 5

  silo:
    image: docker.io/pgsty/silo:RELEASE.2026-08-06T00-00-00Z
    restart: unless-stopped
    command: server /data --console-address ":9001"
    environment:
      MINIO_ROOT_USER: indelible
      MINIO_ROOT_PASSWORD: ${MINIO_ROOT_PASSWORD}
    volumes:
      - silodata:/data
    healthcheck:
      test: ['CMD', 'silo', 'healthcheck', 'ready']
      interval: 5s
      timeout: 5s
      retries: 5

  # Creates the bucket once Silo is healthy, then exits.
  silo-init:
    image: docker.io/pgsty/silo:RELEASE.2026-08-06T00-00-00Z
    environment:
      MINIO_ROOT_PASSWORD: ${MINIO_ROOT_PASSWORD}
    depends_on:
      silo:
        condition: service_healthy
    entrypoint: >
      /bin/sh -c "
      mcli alias set silo http://silo:9000 indelible ${MINIO_ROOT_PASSWORD} &&
      mcli mb --ignore-existing silo/indelible
      "

  renderer:
    image: ghcr.io/useindelible/ind-renderer:latest
    restart: unless-stopped
    environment:
      <<: *app-env
      RENDERER_HOST: 0.0.0.0
      RENDERER_PORT: 3100
    depends_on:
      silo:
        condition: service_healthy

  api:
    image: ghcr.io/useindelible/ind-api:latest
    restart: unless-stopped
    environment:
      <<: *app-env
      IND_HOST: 0.0.0.0
      IND_PORT: '8080'
      IND_BASE_URL: https://${INDELIBLE_HOST}
      FRONTEND_URL: https://${INDELIBLE_HOST}
      CORS_ORIGINS: https://${INDELIBLE_HOST}
      JWT_SECRET: ${JWT_SECRET}
      CSRF_SECRET: ${CSRF_SECRET}
      ASSET_COOKIE_SECRET: ${ASSET_COOKIE_SECRET}
      OIDC_ENABLED: ${OIDC_ENABLED:-false}
      OIDC_ISSUER_URL: ${OIDC_ISSUER_URL:-}
      OIDC_CLIENT_ID: ${OIDC_CLIENT_ID:-}
      OIDC_CLIENT_SECRET: ${OIDC_CLIENT_SECRET:-}
      OIDC_PROVIDER_NAME: ${OIDC_PROVIDER_NAME:-OpenID Connect}
      OIDC_SCOPES: ${OIDC_SCOPES:-openid,email,profile}
      OIDC_AUTO_CREATE_USERS: ${OIDC_AUTO_CREATE_USERS:-true}
    ports:
      - '38473:8080'
    depends_on:
      postgres:
        condition: service_healthy
      silo:
        condition: service_healthy
      silo-init:
        condition: service_completed_successfully

  worker:
    image: ghcr.io/useindelible/ind-worker:latest
    restart: unless-stopped
    environment:
      <<: *app-env
      RENDERER_URL: http://renderer:3100
    depends_on:
      postgres:
        condition: service_healthy
      silo:
        condition: service_healthy
      renderer:
        condition: service_started
      silo-init:
        condition: service_completed_successfully

volumes:
  pgdata:
  silodata:
```

`docker compose up -d`, then add TLS in front of port `38473` using one of the
options below. Migrations run automatically at startup, so there is no separate
database step.

## Upgrading

Release downloads pin `INDELIBLE_VERSION` in `.env`, so upgrades are explicit
and all three Indelible services move together. Take a database dump, download
the Compose file from the new release, then change only that version line in
your existing `.env`:

```bash
curl -fsSL https://github.com/useindelible/indelible/releases/latest/download/docker-compose.yml \
  -o docker-compose.yml

# Edit the existing line to the version shown by the new GitHub release:
# INDELIBLE_VERSION=0.2.0

docker compose pull
docker compose up -d
```

Do not replace your existing `.env` with `example.env`: it contains your
database password, object-store password, and authentication secrets. Database
migrations run automatically and are forward-only, so keep the pre-upgrade dump
until you have verified the new release.

If you reach the same instance through more than one hostname, a LAN name and a
tailnet name for example, list them all in `CORS_ORIGINS`. Session refresh
accepts any configured origin.

## Your first account

Open your instance and sign up. On an empty instance the first registration
always succeeds, and **signups close by themselves once that account exists**.
There is no setting to remember to switch off.

To add more people later, set `AUTH_ALLOW_SIGNUPS=true`, restart, let them
register, then set it back. There is no admin user-management screen yet.

## Bringing your own PostgreSQL and object storage

Already running Postgres and S3? Drop the `postgres`, `silo`, and `silo-init`
services and point the app at yours:

- `DATABASE_URL` needs a database with the **pgvector** extension available.
  Indelible creates the extension itself, so the role needs rights to do that,
  and migrations run at startup.
- `S3_ENDPOINT`, `S3_BUCKET`, `S3_ACCESS_KEY`, `S3_SECRET_KEY`, and `S3_REGION`
  accept any S3-compatible provider. Set `S3_FORCE_PATH_STYLE=true` for Silo,
  Ceph, and Garage; leave it off for AWS S3 and most hosted providers.
- Keep `AUTH_CREDENTIAL_KEY` identical on `api` and `worker`. The API encrypts
  stored integration tokens and the worker decrypts them.

Setting `S3_ENDPOINT` implicitly enables object storage, so commenting it out
also disables it.

## Archived assets

By default the API streams archived files (images, PDFs, saved pages) through
its own origin, which needs no publicly reachable object store: Silo stays
private on the compose network and only `ind-api` talks to it. That mode signs
short-lived cookies, which is why `ASSET_COOKIE_SECRET` is required in
production.

The alternative, `ASSET_SERVING_MODE=presigned`, makes asset downloads redirect
to short-lived signed URLs at `S3_ENDPOINT`, offloading download bandwidth to
the object store. Only use it when that endpoint is a browser-reachable,
TLS-terminated address, such as real AWS S3. Uploads go through the API in both
modes, and API responses always contain API URLs.

## TLS

Pick whichever matches your network:

| Setup | Certificate | Notes |
| --- | --- | --- |
| Traefik (recommended) | Let's Encrypt, automatic | Routes by container labels, renews on its own, and fits the compose stack above |
| Tailscale | `tailscale serve` issues a ts.net certificate | No DNS and no certificate work; trusted by every client out of the box |
| Caddy | Let's Encrypt, automatic | Smallest possible config if you would rather not run Traefik |
| Private CA (step-ca, mkcert, Caddy internal) | Your own root | Install the root on each device. Android and iOS both accept user-installed roots |

### Traefik

Add Traefik to the compose file, drop the `ports:` mapping from the `api` service
so it is reachable only through the proxy, and label the service instead:

```yaml
  traefik:
    image: traefik:v3
    command:
      - --providers.docker=true
      - --providers.docker.exposedbydefault=false
      - --entryPoints.web.address=:80
      - --entryPoints.websecure.address=:443
      - --certificatesresolvers.le.acme.email=you@example.com
      - --certificatesresolvers.le.acme.storage=/letsencrypt/acme.json
      - --certificatesresolvers.le.acme.tlschallenge=true
    ports:
      - '80:80'
      - '443:443'
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock:ro
      - letsencrypt:/letsencrypt

  api:
    # ...as above, without the ports: mapping
    labels:
      - traefik.enable=true
      - traefik.http.routers.indelible.rule=Host(`indelible.example.com`)
      - traefik.http.routers.indelible.entrypoints=websecure
      - traefik.http.routers.indelible.tls.certresolver=le
      - traefik.http.services.indelible.loadbalancer.server.port=8080
```

Add `letsencrypt:` to the `volumes:` block. Traefik streams server-sent events
without buffering, so Mila's replies and the realtime feed arrive incrementally
with no extra tuning.

### Caddy

If you want the smallest possible configuration, a two-line Caddyfile does the
same job, including automatic certificates:

```caddyfile
indelible.example.com {
	reverse_proxy api:8080
}
```

### Tailscale

One command replaces the certificate work entirely:

```bash
tailscale serve --bg 38473
```

Then set `IND_BASE_URL`, `FRONTEND_URL`, and `CORS_ORIGINS` to your
`https://<host>.ts.net` address.

Whatever sits in front, set `TRUSTED_PROXIES` to your proxy's IPs/CIDRs (e.g.
`10.0.0.0/8`) so rate limiting sees real client IPs. Leave it unset when the API
is exposed directly, otherwise clients can spoof their IP via `X-Forwarded-For`.
On nginx, disable response buffering for `/api/v1` so streamed responses are not
batched; Traefik, Caddy, and `tailscale serve` need no such tuning.

## One origin

`ind-api` serves the web interface and the API from the same port, and that
single origin is the deployment contract. The browser extension calls
`/api/v1/...` and an auth route on whatever server URL you give it, the web app
calls the origin it was served from, and the refresh cookie is scoped to that
origin. Point one hostname at `ind-api` and set `IND_BASE_URL`, `FRONTEND_URL`,
and `CORS_ORIGINS` to it. Serving the web app from a different origin than the
API is not a supported layout.

See the [Configuration reference](/docs/reference/configuration/) for everything
else, including OIDC SSO, email ingestion, Mila, and integrations, and
[Backup, restore & upgrades](/docs/self-hosting/operations/) for day-two
operations.
