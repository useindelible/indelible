#!/bin/sh
set -eu

RELEASE_TAG=latest
REPOSITORY=https://github.com/useindelible/indelible

if [ -e .env ]; then
  echo "error: .env already exists; refusing to overwrite your secrets" >&2
  exit 1
fi

for command in curl openssl docker; do
  if ! command -v "$command" >/dev/null 2>&1; then
    echo "error: $command is required" >&2
    exit 1
  fi
done

if ! docker compose version >/dev/null 2>&1; then
  echo "error: Docker Compose is required" >&2
  exit 1
fi

if [ "$RELEASE_TAG" = latest ]; then
  release_url=$REPOSITORY/releases/latest/download
else
  release_url=$REPOSITORY/releases/download/$RELEASE_TAG
fi

curl -fsSL "$release_url/docker-compose.yml" -o docker-compose.yml
curl -fsSL "$release_url/example.env" -o example.env
curl -fsSL "$release_url/self-hosting-checksums-sha256.txt" \
  -o self-hosting-checksums-sha256.txt

if command -v sha256sum >/dev/null 2>&1; then
  sha256sum -c self-hosting-checksums-sha256.txt
elif command -v shasum >/dev/null 2>&1; then
  shasum -a 256 -c self-hosting-checksums-sha256.txt
else
  echo "error: sha256sum or shasum is required to verify the release" >&2
  exit 1
fi

cp example.env .env
{
  echo "POSTGRES_PASSWORD=$(openssl rand -hex 16)"
  echo "MINIO_ROOT_PASSWORD=$(openssl rand -hex 16)"
  echo "JWT_SECRET=$(openssl rand -hex 32)"
  echo "CSRF_SECRET=$(openssl rand -hex 32)"
  echo "ASSET_COOKIE_SECRET=$(openssl rand -hex 32)"
  echo "AUTH_CREDENTIAL_KEY=$(openssl rand -base64 32)"
} >> .env

docker compose pull
docker compose up -d

echo "Indelible is running at http://localhost:38473"
