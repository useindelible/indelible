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
volume, `minio` with the `miniodata` volume.

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

If you run the bundled MinIO, mirror the bucket to a local directory with a
one-off `mc` container. The `minio-init` service already has the client image
and network access, so reuse it:

```bash
docker compose run --rm --entrypoint sh -v "$(pwd)/minio-backup:/backup" minio-init \
  -c 'mc alias set src http://minio:9000 minioadmin YOUR_MINIO_PASSWORD && mc mirror src/indelible /backup'
```

Simpler and equally valid: stop the stack and copy the volume directly.

```bash
docker compose stop
docker run --rm -v miniodata:/data -v "$(pwd)":/backup alpine tar czf /backup/miniodata-$(date +%F).tar.gz -C /data .
docker compose start
```

If you pointed Indelible at an external S3 provider instead of MinIO, use that
provider's replication or lifecycle tooling and skip this section.

### Configuration

Keep a copy of your `docker-compose.yml` and every secret in it (`JWT_SECRET`,
`CSRF_SECRET`, `ASSET_COOKIE_SECRET`, database and MinIO passwords). Losing
`ASSET_COOKIE_SECRET` or `JWT_SECRET` does not lose data, but it invalidates
active sessions when you rotate them.

## Restore

On a fresh host, restore configuration first, then data, then start the stack:

```bash
# 1. Bring up only the stateful services
docker compose up -d postgres minio minio-init

# 2. Restore the database
docker compose exec -T postgres pg_restore -U indelible -d indelible --clean --if-exists < indelible-2026-07-29.dump

# 3. Restore the object store volume (if you used the tar method)
docker run --rm -v miniodata:/data -v "$(pwd)":/backup alpine sh -c 'cd /data && tar xzf /backup/miniodata-2026-07-29.tar.gz'

# 4. Start everything
docker compose up -d
```

The database and the object store must come from the same point in time.
Restoring a newer database against an older bucket leaves documents whose
archived assets are missing; the reverse leaves orphaned files, which is
harmless but wastes space.

## Upgrades

Database migrations are embedded in the server binaries and run automatically
at startup, so an upgrade is a pull and a restart:

```bash
docker compose pull
docker compose up -d
```

Take a database dump before upgrading. Migrations are forward-only; rolling
back to an older image after a migration has run is not supported, so the dump
is your way back.

### Pinning versions

Every release publishes semver tags alongside `latest`:

```yaml
image: ghcr.io/useindelible/ind-api:0.1.0   # exact release
image: ghcr.io/useindelible/ind-api:0.1     # latest patch of 0.1
image: ghcr.io/useindelible/ind-api:latest  # tip of main
```

For production, pin `ind-api`, `ind-worker`, and `ind-renderer` to the same
semver tag and upgrade them together. The three images are released as a set;
mixing versions across a migration boundary is untested.

## Health and logs

- `GET /api/health` returns `200` when the API is up; point your uptime monitor
  at it.
- `docker compose logs -f api worker renderer` tails the services. Logs go to
  stdout, so any Docker log driver works.
- `docker compose ps` shows the health state of `postgres` and `minio` from
  their built-in healthchecks.
