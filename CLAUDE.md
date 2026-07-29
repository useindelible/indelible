# CLAUDE.md — Indelible

> **MANDATORY:** Before writing any backend code, read [`contributing.agents.md`](contributing.agents.md). It defines the non-negotiable architecture rules (module boundaries, DTO placement, file-size caps, bootstrap layout) that every agent must follow. Violations are blockers, not nits.

## Product
Indelible — open-source, self-hosted read-it-later & knowledge archiver.
AI assistant: Mila.

## Tech Stack
| Layer | Tech | Notes |
|-------|------|-------|
| Backend | Rust + Axum + Tower | Workspace in `backend/` |
| Database | PostgreSQL + pgvector | SQLx compile-time queries |
| Jobs | Apalis (Postgres-backed) | Priority queues, dead letters |
| Storage | S3-compatible (MinIO dev) | Presigned URLs |
| Web | SvelteKit (SPA, no SSR) | `web/` — adapter-static |
| Mobile | Kotlin Multiplatform + Compose | `mobile/` |
| Extension | WXT + TypeScript (MV3) | `extension/` |
| CLI | Rust binary (`ind`) | `backend/apps/ind-cli/` |

## Philosophy
Prefer the simplest solution that fully solves the problem.

## Workspace Layout
```
indelible/
├── backend/                 # Rust workspace
│   ├── Cargo.toml           # [workspace] manifest
│   ├── apps/
│   │   ├── ind-api/         # Axum HTTP server
│   │   ├── ind-worker/      # Background job processor
│   │   └── ind-cli/         # CLI tool
│   └── crates/
│       ├── ind-domain/      # Entities, typed IDs, value objects
│       ├── ind-application/ # Use cases, handlers
│       ├── ind-persistence/ # SQLx repos, migrations
│       ├── ind-auth/        # Auth, sessions, OAuth, tokens
│       ├── ind-search/      # FTS, semantic search
│       ├── ind-ai/          # Mila, provider abstraction, RAG
│       ├── ind-ingest/      # Fetch pipeline, extraction, archival
│       ├── ind-integrations/# Obsidian/Notion sync, email ingest
│       ├── ind-http-api/    # Axum routes, DTOs, error types
│       ├── ind-observability/# Tracing, metrics
│       └── ind-test-support/# Fixtures, testcontainers
├── web/                     # SvelteKit SPA (built into the ind-api image)
├── mobile/                  # KMP + Compose Multiplatform
├── extension/               # Chrome/Firefox MV3 (WXT)
├── obsidian/                # Obsidian plugin
├── website/                 # Astro + Starlight docs site
├── shared/                  # Code shared between web and extension
├── docker/                  # Container support files
└── scripts/                 # CI tooling (test LOC budget, mutation sampling)
```

## Documentation
User-facing and operator documentation lives in `website/` (Astro + Starlight) and
is published at https://useindelible.com. Read it before working on a domain:

| Domain | Doc |
|--------|-----|
| Install and deploy | `website/src/content/docs/docs/self-hosting/install.md` |
| Production hardening | `website/src/content/docs/docs/self-hosting/security.md` |
| Every environment variable | `website/src/content/docs/docs/reference/configuration.md` |
| Feature guides (extension, feeds, Notion, Obsidian, email, imports) | `website/src/content/docs/docs/how-to/` |
| Known limitations | `website/src/content/docs/docs/getting-started/limitations.md` |

Architecture is documented in the code: each crate's `lib.rs` and module docs are
the source of truth, and `contributing.agents.md` defines the boundaries every
change must respect.

## Canonical Naming
| Item | Correct | Wrong |
|------|---------|-------|
| Product | **Indelible** | — |
| CLI | `ind` | — |
| Token prefix | `ind_` | — |
| Email domain | `<token>@feed.useindelible.com` (Feed), `<token>@library.useindelible.com` (Library) — both override via `EMAIL_FEED_DOMAIN` / `EMAIL_LIBRARY_DOMAIN` | `*.indelible.app`, `ingest.*` |
| AI assistant | **Mila** | Ghostreader |
| ID format | UUIDv7, prefixed (`usr_`, `itm_`, `col_`) | — |

## Comment Hygiene
The codebase is not your notebook. Do not write obvious comments.

## Commnication Style
While interacting with the user, do not communcate in a prose long winded format. Go straight to the point, present your facts immmediately.

## Coding Conventions

See `contributing.agents.md` "Hexagonal Boundaries" for the enforceable boundary rules (crypto/SQL/env/HTTP placement, file-size hard reject, `#[allow(dead_code)]` policy).

### Rust
- Edition 2024, MSRV defined in `rust-toolchain.toml`
- `cargo fmt` (rustfmt) — enforced
- `cargo clippy -- -D warnings` — zero warnings policy
- Error handling: `thiserror` for library crates, `anyhow` for app crates
- Async: `tokio` runtime, no blocking in async contexts
- IDs: newtype wrappers (`UserId(Uuid)`) in `ind-domain`
- DB: `sqlx` with compile-time checked queries
- Tests: `#[tokio::test]`, testcontainers for integration tests
- Backend testing policy: integration tests are mandatory for changes touching persistence, HTTP routes, object storage, auth middleware, outbox/jobs, or cross-crate flows. Unit tests are still expected for pure logic, error mapping, parsers, and edge-case combinatorics.
- Backend integration tests must use the shared test shape: `ind_test_support::spawn_app()`, shared `common` modules, scenario helpers, `AuthedClient` sessions for authenticated API calls, real testcontainers through `ind-test-support`, and deterministic job/storage harnesses. Do not freestyle new app harnesses, raw bearer-token request helpers, or per-test container setup.
- Do not hand-roll Rust source parsers, brace matchers, or syntax scanners; use `syn`/rustc tooling or restructure the test/code so parsing is unnecessary.
- Backend coverage target: 75%. Report coverage with `cd backend && cargo llvm-cov --workspace --all-features --summary-only` when backend behavior changes, and say whether the change moves coverage toward the target. Coverage is not a hard CI gate unless the task explicitly says so.
- **No untyped Object schemas in utoipa annotations.** Every `serde_json::Value` field in an API DTO must have a `#[schema(value_type = ...)]` override that produces a concrete OpenAPI type. Bare `serde_json::Value` produces `{"type": "object"}` or `{}` in the spec, which code generators map to `Any` — this breaks kotlinx.serialization on mobile and produces `unknown` on web. The `value_type` must accurately describe the field's actual runtime shape. Do NOT use `HashMap<String, String>` as a lazy default — if the field contains nested objects, arrays, or non-string values, define a dedicated schema-only struct or enum that reflects the real structure. See `FilterExpressionNode` in `smart_lists/dto.rs` for the pattern.
- Before hand-rolling infrastructure or domain logic, check whether a mature open-source library already solves the problem. Prefer the established library when it fits the repository and keeps the solution simpler.

## Attitudes and Principles
- **Clarity over cleverness**: Prioritize readability and maintainability over clever one-liners or over-engineering. Code is read more often than written.
- **YAGNI**: Don't implement features or abstractions until they are actually needed. Avoid speculative generality.
- **No Lazyness**: If something needs to be done, do it. Don't leave TODOs or deferred decisions in the code. No comment like `X issue is unrelated to my changes so I will ignore it for now`. Explore the issue and fix them.
- **Conflict Resolution**: If you encounter a merge conflict, do not try to resolve with hacks like python scripts, regex or sed. Use the standard edit tool. Multiple agents can be trying ot merge to main at the same time, and we need to ensure that all conflicts are resolved properly with human-level understanding, not just blindly accepting one side or the other and deleting code from the other side. If you created a stash while trying to resolve a conflict, make sure to restore the stash and never delete it.

### Repository Mutations
- **Never use generic full-row `update()` methods.** All entity mutations must use targeted, column-scoped update methods (e.g., `toggle_favorite`, `set_triage_state`, `update_profile_fields`), not a generic `update(Entity)` full-row write
- Full-row overwrites cause optimistic locking conflicts (Rust timestamps have nanosecond precision, Postgres has microsecond — the `WHERE updated_at = $old` check fails) and risk overwriting concurrent mutations to unrelated fields
- This applies to **all** repositories, not just users: `UserRepository`, `DocumentRepository`, `LibraryRepository`, `CollectionRepository`, etc.
- When adding a new mutation, add a dedicated method to the repository trait and implement it as a targeted SQL `UPDATE ... SET column = $value` with `RETURNING`
- Library-entry/highlight/tag mutations that need domain events or search reindexing must take `MutationSideEffects`; callers that truly need no side effects pass `MutationSideEffects::none()`. Do not add parallel `*_with_side_effects` methods or default trait impls that silently drop effects.
- Persist the primary mutation, `domain_events`, and required `job_outbox` rows in one repository-managed transaction via the shared write helpers. Search indexing still runs async, but the reindex outbox row must be committed atomically with the data change that requires it.
- Build domain events with `ind_application::event_intents` so payloads stay consistent. For toggles or other state-dependent mutations, construct the event from the `UPDATE ... RETURNING` row inside the transaction, not from a pre-read in the handler.
- Create/save flows must commit document/library lifecycle changes, asset rows, `job_outbox`, and `domain_events` in one repository-managed transaction (`PgDocumentLifecycle::save_to_library` / `materialize_document` with their `side_effects` closures); avoid fire-and-forget `append_event(...).await.ok()` for committed domain state. Browser/email-provided content is staged to object storage before the save transaction (`handlers/provided_content.rs`) so the in-tx attach outbox row references an existing object.

### API Clients
- Generate typed HTTP clients from OpenAPI specs — no freestyle `fetch`/`axios`
- Backend defines the OpenAPI schema; `web/`, `extension/`, and `mobile/` consume generated types
- Use `openapi-typescript` + `openapi-fetch` (web/extension) and Fabrikt (Kotlin, mobile)

### TypeScript (Web + Extension)
- Strict mode, no `any`
- ESLint + Prettier enforced
- Svelte 5 runes syntax (`$state`, `$derived`, `$effect`)
- `eslint-plugin-svelte` with `svelteFeatures: { runes: true }` for parser-level runes awareness
- `eslint-plugin-svelte-runes` for dedicated rules (no-external-rune-calls, no-effect-outside-components, no-props-outside-components, no-direct-rune-assign, etc.)
- `svelte-check` for type checking (catches some runes misuse at compile level)
- No SSR — SvelteKit adapter-static only
- Vitest for unit tests, Playwright for e2e

### CSS / Design Tokens (Web)
- **Never hardcode colour values** in component `<style>` blocks. Every colour must use a CSS custom property from `web/src/lib/styles/tokens.css`.
- Key tokens: `--accent`, `--destructive`, `--success`, `--warning`, `--text-primary/secondary/tertiary/quaternary`, `--bg-primary/secondary/tertiary/elevated/content`, `--border-primary/secondary`, `--fill-hover/selected/selected-strong`, `--fill-success/warning/danger`, `--text-on-color` (white text on coloured backgrounds), `--shadow-1/3`, `--highlight-*-bg/border`
- `background: transparent` is acceptable as a CSS reset. `rgba()` values in `box-shadow` are acceptable. All other colour values must be tokens.
- If you need a colour that has no token, **add the token to `tokens.css`** (both `:root` light and `[data-theme='dark']` dark variants) rather than hardcoding it inline.
- Platform brand colours (YouTube red, Twitter/X black) are the only permitted exception.

### Kotlin (Mobile)
- **Before writing any mobile code, invoke the `/kmp` skill** — it contains required context for Kotlin/Native iOS interop, build/run workflows, and known gotchas
- **Read [`mobile/CONTRIBUTING.md`](mobile/CONTRIBUTING.md)** for design system usage, component conventions, and do's/don'ts
- ktlint for formatting
- detekt for static analysis
- Compose Multiplatform for shared UI
- Coroutines for async
- Design system: all colour/type/spacing/shape decisions live in `mobile/composeApp/src/commonMain/kotlin/app/indelible/ui/theme/`; never freestyle these in screen files
- **Mobile compile verification MUST use `./gradlew :composeApp:compileCommonMainKotlinMetadata`** (or a full assemble). `:composeApp:compileKotlinMetadata` exits 0 without compiling commonMain and must never be used as a gate. Run commonTest via `./gradlew :composeApp:jvmTest`.

### File Size
- **Sweet spot: 500 LOC per file.** Aim to keep every committed source file at or under 500 lines. Past that, the file is harder to review, harder to navigate, surfaces fewer reuse opportunities for other agents, and tends to mix concerns. When a file approaches 500 LOC, plan the split before adding more code.
- **Hard cap: 600 LOC.** Reviewers reject any change that introduces or leaves a non-generated, non-test source file above 600 lines. Split into cohesive sibling files (handler-per-file, by use case, by subdomain) before requesting review.
- **Exemptions:** generated code (`.sqlx/`, OpenAPI output), `app_wiring.rs`, and pure test files (`tests.rs`, `*_tests.rs`, anything under a `/tests/` directory). Wiring entry points like `main.rs` / `config.rs` are not exempt — split into `bootstrap.rs` / `services.rs` / `router.rs` as `contributing.agents.md` describes.
- Per-file-kind caps in `contributing.agents.md` (e.g. `routes/<domain>/mod.rs` 500, `adapters/<domain>.rs` 600) remain authoritative when stricter than this global cap. The 600 LOC reject line never gets relaxed by them.

## Code Comments Policy
- **Only decision-explanation comments are allowed** — explain *why* a non-obvious choice was made
- No emojis in code or comments
- No monologue comments ("now we do X", "this section handles Y")
- No lazy TODOs (`// TODO: fix this later`) — either fix it or file a backlog item
- No nonsensical, restating-the-obvious, or filler comments
- If the code is self-explanatory, don't comment it

## Git Workflow
- Branch naming: `<type>/<backlog-id>-<short-desc>` (e.g., `feat/B-012-user-auth`)
- Conventional commits: `feat:`, `fix:`, `refactor:`, `test:`, `docs:`, `chore:`
- One commit per logical change
- Subagent work happens in worktrees (inside the .claude folder not outside the workdir), merged by orchestrator

## Workflow
- GitHub Issues track work; reference the issue number in the branch name and PR title
- Subagent work happens in isolated worktrees, merged by the orchestrator

## SQLx
- Never try to edit the .sqlx files directly — always use `cargo sqlx prepare --workspace` to regenerate them after changing queries in the Rust code
- Never try to use online mode. Always use offline mode. Postgres is always running locally inside docker so read .env for the connection string and use that for SQLx CLI
- Always use compile time checked queries with `sqlx::query!` or `sqlx::query_as!` macros. Do not use the runtime-checked `sqlx::query()` with manual `.bind()` parameters for queries that are executed in production code.
- Do not use `sqlx::query_file!` or `sqlx::query_file_as!`; inline repository SQL is allowed to exceed file-size soft caps.

<!-- BACKLOG.MD MCP GUIDELINES START -->

<CRITICAL_INSTRUCTION>

## BACKLOG WORKFLOW INSTRUCTIONS

This project uses Backlog.md MCP for all task and project management activities.

**CRITICAL GUIDANCE**

- If your client supports MCP resources, read `backlog://workflow/overview` to understand when and how to use Backlog for this project.
- If your client only supports tools or the above request fails, call `backlog.get_workflow_overview()` tool to load the tool-oriented overview (it lists the matching guide tools).

- **First time working here?** Read the overview resource IMMEDIATELY to learn the workflow
- **Already familiar?** You should have the overview cached ("## Backlog.md Overview (MCP)")
- **When to read it**: BEFORE creating tasks, or when you're unsure whether to track work

These guides cover:
- Decision framework for when to create tasks
- Search-first workflow to avoid duplicates
- Links to detailed guides for task creation, execution, and finalization
- MCP tools reference

You MUST read the overview resource to understand the complete workflow. The information is NOT summarized here.

</CRITICAL_INSTRUCTION>

<!-- BACKLOG.MD MCP GUIDELINES END -->
