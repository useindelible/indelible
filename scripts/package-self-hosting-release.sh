#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "usage: $0 <vX.Y.Z[-prerelease]> <output-directory>" >&2
  exit 2
fi

release_tag="$1"
output_dir="$2"

if [[ ! "$release_tag" =~ ^v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z][0-9A-Za-z.-]*)?$ ]]; then
  echo "invalid release tag: $release_tag" >&2
  exit 2
fi

if [[ -e "$output_dir" ]]; then
  echo "output path already exists: $output_dir" >&2
  exit 2
fi

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
version="${release_tag#v}"

mkdir "$output_dir"
cp "$repo_root/website/public/quickstart/docker-compose.yml" "$output_dir/docker-compose.yml"
sed "s/^INDELIBLE_VERSION=latest$/INDELIBLE_VERSION=$version/" \
  "$repo_root/website/public/quickstart/env.example" > "$output_dir/example.env"

grep -qx "INDELIBLE_VERSION=$version" "$output_dir/example.env"
(
  cd "$output_dir"
  shasum -a 256 docker-compose.yml example.env > self-hosting-checksums-sha256.txt
)
