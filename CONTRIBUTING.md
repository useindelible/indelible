# Contributing to Indelible

Thanks for considering a contribution.

## Before you start

Open an issue before large or structural changes so we can agree on the
approach. Small fixes can go straight to a pull request.

## Development setup

```bash
docker compose up -d postgres minio minio-init   # dependencies only
cd backend && cargo run -p ind-api               # API on :38473
cd web && pnpm install && pnpm dev               # web app on :5173
```

The backend expects `DATABASE_URL`; copy `.env.example` to `.env` and adjust.
Migrations run automatically at startup.

## Checks

Run these before opening a pull request. CI runs the same commands.

| Area | Commands |
| --- | --- |
| Backend | `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace --all-features` |
| Web | `pnpm check`, `pnpm lint`, `pnpm format:check`, `pnpm test` |
| Extension | `npm run lint`, `npm run check`, `npm test` |
| Mobile | `./gradlew :composeApp:compileCommonMainKotlinMetadata`, `./gradlew :composeApp:jvmTest` |
| Website | `pnpm check`, `pnpm build` |

Backend integration tests use real containers through `ind-test-support`, so
Docker must be running.

## Tests

Integration tests are expected for changes touching persistence, HTTP routes,
object storage, auth, or background jobs. Unit tests are expected for pure
logic, parsers, and error mapping. New behaviour without a test that would fail
without it will be sent back.

## Pull requests

- Branch from `main`, named `<type>/<short-description>`.
- PR titles follow [Conventional Commits](https://www.conventionalcommits.org/)
  (`feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`). CI checks the
  title, and it becomes the squashed commit message.
- Keep one logical change per PR.
- Say what you ran to verify the change, not just what you changed.

## Architecture rules

Backend changes are reviewed against the boundaries in
[contributing.agents.md](contributing.agents.md): module placement, where SQL
and crypto and env access may live, DTO rules, and file-size caps. Violations
block a merge.

## Reporting security issues

Do not open a public issue. See [SECURITY.md](SECURITY.md).
