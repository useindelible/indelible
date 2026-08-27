---
title: Backup, restore & upgrades
sidebar:
  order: 3
---

An Indelible instance keeps state in exactly two places: PostgreSQL (documents,
highlights, users, jobs) and the object store (archived pages, images, PDFs,
audio). Back up both, plus your compose file and secrets, and the instance is
fully recoverable.

Service and volume names below match the compose file from the
[install guide](/docs/self-hosting/install/): `postgres` with the `pgdata`
volume, `silo` with the `silodata` volume.

## Backup

### PostgreSQL

Dump the database from the running container:

```bash
docker compose exec postgres pg_dump -U indelible -Fc indelible > indelible-$(date +%F).dump
```

`-Fc` produces a compressed custom-format dump that `pg_restore` can restore
selectively. Schedule this with cron and rotate old dumps; nightly is a sensible
default.

### Object store

If you run the bundled Silo, mirror the bucket to a local directory with a
one-off `mcli` client. The `silo-init` service already has the client image
and network access, so reuse it:

```bash
docker compose run --rm --entrypoint sh -v "$(pwd)/silo-backup:/backup" silo-init \
  -c 'mcli alias set src http://silo:9000 indelible ${MINIO_ROOT_PASSWORD} && mcli mirror src/indelible /backup'
```

Simpler and equally valid: stop the stack and copy the volume directly.

```bash
docker compose stop
docker run --rm --volumes-from "$(docker compose ps -aq silo)" -v "$(pwd)":/backup alpine tar czf /backup/silodata-$(date +%F).tar.gz -C /data .
docker compose start
```

If you pointed Indelible at an external S3 provider instead of Silo, use that
provider's replication or lifecycle tooling and skip this section.

### Configuration

Keep a copy of your `docker-compose.yml` and every secret in it (`JWT_SECRET`,
`CSRF_SECRET`, `ASSET_COOKIE_SECRET`, database and Silo passwords). Losing
`ASSET_COOKIE_SECRET` or `JWT_SECRET` does not lose data, but it invalidates
active sessions when you rotate them.

## Restore

On a fresh host, restore configuration first, then data, then start the stack:

```bash
# 1. Start PostgreSQL and create (but do not start) the Silo container and volume
docker compose up -d postgres
docker compose create silo

# 2. Restore the database
docker compose exec -T postgres pg_restore -U indelible -d indelible --clean --if-exists < indelible-2026-07-29.dump

# 3. Restore the object store volume (if you used the tar method)
docker run --rm --volumes-from "$(docker compose ps -aq silo)" -v "$(pwd)":/backup alpine sh -c 'cd /data && tar xzf /backup/silodata-2026-07-29.tar.gz'

# 4. Start everything
docker compose up -d
```

The database and the object store must come from the same point in time.
Restoring a newer database against an older bucket leaves documents whose
archived assets are missing; the reverse leaves orphaned files, which is
harmless but wastes space.

## Upgrades

Database migrations are embedded in the server binaries and run automatically
at startup. The release bundle pins all three Indelible images through
`INDELIBLE_VERSION`, so first download the new release's Compose file and edit
that one line in your existing `.env`. Do not replace `.env`; it contains the
secrets for your installation.

```bash
curl -fsSL https://github.com/useindelible/indelible/releases/latest/download/docker-compose.yml \
  -o docker-compose.yml
# Edit INDELIBLE_VERSION in .env to the new release version.
docker compose pull
docker compose up -d
```

Take a database dump before upgrading. Migrations are forward-only; rolling
back to an older image after a migration has run is not supported, so the dump
is your way back.

### Release channels

Every stable release stages its exact semver images before publication. Once
the release is published, the same verified digests move `latest` and the
minor-version channel. Merges to `main` publish `dev` and a short commit SHA and
never move stable channels.

```yaml
image: ghcr.io/useindelible/ind-api:0.1.0   # exact release
image: ghcr.io/useindelible/ind-api:0.1     # latest patch of 0.1
image: ghcr.io/useindelible/ind-api:latest  # newest stable release
image: ghcr.io/useindelible/ind-api:dev     # tip of main; unreleased, may break
```

The downloadable release Compose uses the exact tag. Floating tags are for
people who deliberately want channel-style updates.

For production, pin `ind-api`, `ind-worker`, and `ind-renderer` to the same
semver tag and upgrade them together. The three images are released as a set;
mixing versions across a migration boundary is untested.

## Health and logs

- `GET /api/health` returns `200` when the API is up; point your uptime monitor
  at it.
- `docker compose logs -f api worker renderer` tails the services. Logs go to
  stdout, so any Docker log driver works.
- `docker compose ps` shows the health state of `postgres` and `silo` from
  their built-in healthchecks.
