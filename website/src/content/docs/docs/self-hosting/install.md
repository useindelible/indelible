---
title: Install with Docker
sidebar:
  order: 1
---

Indelible ships three server images: `ind-api` (the HTTP API, which also serves the
web app), `ind-worker` (background jobs), and `ind-renderer` (Chromium renderer for
hard-to-capture pages). They run alongside PostgreSQL (with pgvector) and an
S3-compatible object store. The `ind-worker` reaches the renderer over HTTP, and
both the worker and renderer read and write archived content to object storage.

## One origin

`ind-api` serves the web interface and the API from the same port, and that single
origin is the deployment contract. The browser extension calls `/api/v1/...` and an
auth route on whatever server URL you give it, the web app calls the origin it was
served from, and the refresh cookie is scoped to that origin. Point one hostname at
`ind-api` and set `IND_BASE_URL`, `FRONTEND_URL`, and `CORS_ORIGINS` to it. Serving
the web app from a different origin than the API is not a supported layout.

## Secrets

Generate the required secrets before you start. In production the API refuses to
boot with the development defaults:

```bash
openssl rand -base64 32   # JWT_SECRET
openssl rand -base64 32   # CSRF_SECRET
openssl rand -hex 32      # ASSET_COOKIE_SECRET
```

## Compose file

```yaml
x-app-env: &app-env
  IND_ENV: production
  DATABASE_URL: postgres://indelible:change-me@postgres:5432/indelible
  S3_ENDPOINT: http://minio:9000
  S3_REGION: us-east-1
  S3_BUCKET: indelible
  S3_ACCESS_KEY: minioadmin
  S3_SECRET_KEY: change-me-too
  S3_FORCE_PATH_STYLE: 'true'

services:
  postgres:
    image: pgvector/pgvector:pg18
    environment:
      POSTGRES_USER: indelible
      POSTGRES_PASSWORD: change-me
      POSTGRES_DB: indelible
    volumes:
      - pgdata:/var/lib/postgresql/data
    healthcheck:
      test: ['CMD-SHELL', 'pg_isready -U indelible -d indelible']
      interval: 5s
      timeout: 5s
      retries: 5

  minio:
    image: minio/minio:latest
    command: server /data --console-address ":9001"
    environment:
      MINIO_ROOT_USER: minioadmin
      MINIO_ROOT_PASSWORD: change-me-too
    volumes:
      - miniodata:/data
    healthcheck:
      test: ['CMD', 'curl', '-f', 'http://localhost:9000/minio/health/live']
      interval: 5s
      timeout: 5s
      retries: 5

  # Creates the bucket once MinIO is healthy, then exits.
  minio-init:
    image: minio/mc:latest
    depends_on:
      minio:
        condition: service_healthy
    entrypoint: >
      /bin/sh -c "
      mc alias set minio http://minio:9000 minioadmin change-me-too &&
      mc mb --ignore-existing minio/indelible
      "

  renderer:
    image: ghcr.io/useindelible/ind-renderer:latest
    environment:
      <<: *app-env
      RENDERER_HOST: 0.0.0.0
      RENDERER_PORT: 3100
    depends_on:
      minio:
        condition: service_healthy

  api:
    image: ghcr.io/useindelible/ind-api:latest
    environment:
      <<: *app-env
      IND_HOST: 0.0.0.0
      IND_PORT: '8080'
      IND_BASE_URL: https://indelible.example.com
      FRONTEND_URL: https://indelible.example.com
      CORS_ORIGINS: https://indelible.example.com
      JWT_SECRET: paste-generated-secret
      CSRF_SECRET: paste-generated-secret
      ASSET_COOKIE_SECRET: paste-generated-hex-secret
      AUTH_ALLOW_SIGNUPS: 'true'
    ports:
      - '38473:8080'
    depends_on:
      postgres:
        condition: service_healthy
      minio:
        condition: service_healthy
      minio-init:
        condition: service_completed_successfully

  worker:
    image: ghcr.io/useindelible/ind-worker:latest
    environment:
      <<: *app-env
      RENDERER_URL: http://renderer:3100
    depends_on:
      postgres:
        condition: service_healthy
      minio:
        condition: service_healthy
      renderer:
        condition: service_started
      minio-init:
        condition: service_completed_successfully

volumes:
  pgdata:
  miniodata:
```

The container listens on `8080`; the `38473:8080` mapping keeps the familiar host
port. In production the API refuses to boot while `FRONTEND_URL` or `CORS_ORIGINS`
still hold their development defaults, and it requires `https` for `IND_BASE_URL`
and `FRONTEND_URL`: the refresh cookie is `Secure`, so browsers silently discard it
over plain HTTP and sessions die on their first refresh.

If you reach the same instance through more than one hostname (a LAN name and a
tailnet name, say), list them all in `CORS_ORIGINS`. Session refresh accepts any
configured origin.

## Archived assets

By default the API streams archived files (images, PDFs, saved pages) through its
own origin, which needs no publicly reachable object store: MinIO stays private on
the compose network and only `ind-api` talks to it. That mode signs short-lived
cookies, which is why `ASSET_COOKIE_SECRET` is required in production.

The alternative, `ASSET_SERVING_MODE=presigned`, hands browsers presigned URLs
pointing straight at `S3_ENDPOINT`. Only use it when that endpoint is a
browser-reachable, TLS-terminated address. Setting it to an internal hostname such
as `http://minio:9000` produces links no browser can open, and images break with
no error.

## First run

1. `docker compose up -d`
2. Open your instance and create the first account. On an empty instance the
   first signup always succeeds.
3. After creating your account(s), set `AUTH_ALLOW_SIGNUPS: 'false'` and
   `docker compose up -d` again to close signups.

## TLS

The API requires `https` in production, so put a TLS terminator in front of it.
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
tailscale serve --bg 8080
```

Then set `IND_BASE_URL`, `FRONTEND_URL`, and `CORS_ORIGINS` to your
`https://<host>.ts.net` address.

Whatever sits in front, set `TRUSTED_PROXIES` to your proxy's IPs/CIDRs (e.g.
`10.0.0.0/8`) so rate limiting sees real client IPs. Leave it empty when the API
is exposed directly, otherwise clients can spoof their IP via `X-Forwarded-For`.
On nginx, disable response buffering for `/api/v1` so streamed responses are not
batched; Traefik, Caddy, and `tailscale serve` need no such tuning.

See the [Configuration reference](/docs/reference/configuration/) for the common
environment variables, including OIDC SSO, email ingestion, and integrations.
