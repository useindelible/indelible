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

The backend follows hexagonal architecture, and these boundaries block a merge:

- `ind-domain` and `ind-application` stay pure: no SQL, no HTTP types, no
  `std::env::var`, no direct filesystem or network I/O. Reach outside through a
  port in `ind-application::ports`.
- SQL lives only in `ind-persistence`. Add a method to the repository trait
  rather than a query anywhere else.
- Security-critical crypto (HMAC, AEAD, key derivation, constant-time compares)
  lives only in `ind-auth`.
- Environment reads live only in `apps/*/src/config*`. Library crates take a
  typed config struct.
- DTOs live in `routes/<domain>/dto.rs`, never in `state.rs` or a handler.
- Entity mutations are targeted column updates, never a full-row `update()`.
- Presigned object-store URLs never leave `routes/asset_proxy.rs`, which is the
  only place allowed to call `ObjectStorage::presigned_url`. Response bodies
  always carry API-origin asset URLs built by `routes/asset_urls.rs`, in every
  serving mode, and uploads always stream through the API as multipart. A
  self-hosted object store is usually unreachable from the browser, so a
  presigned URL in a response body is a broken link.
- Non-generated, non-test source files stay at or under 600 lines.

Ask in an issue if a change seems to require breaking one of these.

## Reporting security issues

Do not open a public issue. See [SECURITY.md](SECURITY.md).
