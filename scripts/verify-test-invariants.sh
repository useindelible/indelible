#!/usr/bin/env bash
set -euo pipefail

# backend/test-invariants.toml pins each high-risk behaviour to the test that
# proves it. Removing such a test must be a deliberate, reviewed act rather
# than a silent casualty of a refactor, so this fails when a pinned test is
# gone. It checks the compiled test list instead of the source, which also
# catches a test that still exists but no longer runs.

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
backend="$repo_root/backend"
registry="$backend/test-invariants.toml"

cd "$backend"

listed="$(mktemp)"
trap 'rm -f "$listed"' EXIT

cargo test --workspace --all-features -- --list \
  | sed -n 's/: test$//p' \
  | awk -F'::' '{print $NF}' \
  | sort -u >"$listed"

total="$(grep -c '^\[\[invariants\]\]' "$registry")"
missing=0

while IFS= read -r name; do
  if ! grep -qxF "$name" "$listed"; then
    printf 'missing: %s\n' "$name" >&2
    missing=$((missing + 1))
  fi
done < <(awk -F'"' '/^test = /{print $2}' "$registry")

if ((missing > 0)); then
  printf '\n%d of %d pinned invariant tests are missing. Reassign the invariant to a surviving test in %s, in the same change.\n' \
    "$missing" "$total" "test-invariants.toml" >&2
  exit 1
fi

printf 'all %d pinned invariant tests present\n' "$total"
