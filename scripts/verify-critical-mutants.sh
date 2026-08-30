#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
backend="$repo_root/backend"
output="$backend/target/critical-mutants"
results="$output/mutants.out"

cd "$backend"
# cargo-mutants creates only the leaf of --output; on a fresh runner target/ does not
# exist yet and the run dies before testing a single mutant.
mkdir -p "$output"
set +e
cargo mutants \
  --workspace \
  --config mutants.toml \
  --in-place \
  --no-shuffle \
  --test-package ind-ai \
  --test-package ind-api \
  --test-package ind-application \
  --test-package ind-auth \
  --test-package ind-persistence \
  --test-package ind-worker \
  --output "$output"
mutants_status=$?
set -e

for outcome in caught missed timeout unviable; do
  file="$results/${outcome}.txt"
  count=0
  if [[ -f "$file" ]]; then
    count="$(grep -cve '^[[:space:]]*$' "$file" || true)"
  fi
  printf '%-10s %s\n' "$outcome" "$count"
done

# `unviable` means the mutated code did not compile, which says nothing about
# the tests: it is noise from the mutation engine, so it is reported but not
# blocking. `missed` and `timeout` are real signals about assertion quality.
for blocking in missed timeout; do
  file="$results/${blocking}.txt"
  if [[ -s "$file" ]]; then
    printf '\nBlocking %s mutants:\n' "$blocking" >&2
    sed -n '1,80p' "$file" >&2
    exit 1
  fi
done

exit "$mutants_status"
