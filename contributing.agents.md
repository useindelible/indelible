# Agent Contributing Guide — Backend Architecture Rules

These rules are **mandatory** for any agent (Claude Code, Codex, Copilot, etc.) working in the Indelible backend. They exist because the pre-OSS audit (`docs/audits/2026-05-20-api-crates-audit.md`) surfaced 6.8K-LOC adapter files, 1.1K-LOC `main.rs`, 2.0K-LOC `state.rs`, inline env reads scattered across the binary, and crypto/SQL leaking across crate boundaries. Don't reintroduce any of that.

Each rule below names a root cause from the audit. If you're about to write code that violates one, stop and split / move it instead.

---

## Hexagonal Boundaries

The Indelible backend follows hexagonal architecture (ports & adapters):

- **Inside the hexagon:** `ind-domain` (entities, IDs, value objects, domain errors) and `ind-application` (use cases, ports, orchestration). Pure logic; no I/O, no framework types.
- **Driving adapters:** `apps/ind-api` (Axum HTTP), `apps/ind-worker` (Apalis jobs), `apps/ind-cli`, `apps/ind-renderer`. They translate transport-level concerns into application calls.
- **Driven adapters:** `ind-persistence` (SQL/storage), `ind-auth` (crypto, sessions, OAuth), `ind-ai`, `ind-search`, `ind-ingest`, `ind-integrations`. They implement application ports against external systems.

Boundary enforcement is documented + reviewed (see `.claude/skills/review/SKILL.md`), not CI-scanned. The rules below are mandatory.

### 1. `ind-domain` is pure

No `sqlx::*` derives or `sqlx::Type` annotations. No SQL strings. No HTTP types (`axum`, `http`, `utoipa`). No `serde_json::Value` in entity fields except the explicit Rule 11 allowlist — model the shape with a typed struct/enum instead. No I/O — domain code never reaches the database, network, or filesystem directly.

### 2. `ind-application` orchestrates, does not adapt

No `sqlx::*` macros. No HTTP types (`axum::*`, `http::*`, `utoipa::*`, route attributes). No direct `std::env::var(...)` reads. No direct filesystem or network I/O — go through a port. Crypto is restricted to content addressing only (see rule 5).

Forbidden direct dependencies in `ind-application` (use the corresponding port in `ind-application::ports::*`, with the concrete adapter in `ind-ingest`): `reqwest::*` (use `HttpFetcher`), `feed_rs::*` (use `FeedParser`), `quick_xml::*` (use `OpmlParser`), `scraper::*` (use `HtmlExtractor`), `html2md::*` (HTML→text/Markdown helpers live in `ind-html`; depend on that focused crate, not on the ingestion pipeline). Audit queries (both must return zero hits): `rg "(reqwest|feed_rs|quick_xml|scraper|html2md)::" backend/crates/ind-application/src/` and `cargo tree --manifest-path backend/Cargo.toml -p ind-application -e normal | rg "reqwest|feed-rs|quick-xml|scraper|html2md"` (the cargo workspace lives under `backend/`; the repo root has no `Cargo.toml`).

### 3. SQL lives only in `ind-persistence`

No `sqlx::query!`, `query_as!`, `query_scalar!`, `.execute()`, `.fetch_*()`, or raw SQL strings outside `crates/ind-persistence/`. API crates (`ind-api`, `ind-http-api`), driving adapters, and application code consume repository traits from `ind-application::repos`. If a new query is needed, add a method to the repo trait and implement it in `ind-persistence`. This rule replaces the deleted `scripts/check-api-sql-boundary.sh`.

### 4. Security-critical crypto lives only in `ind-auth`

HMAC signing, AEAD encryption, key derivation, and constant-time comparisons live only in `crates/ind-auth/`. No `hmac::*`, no `aes_gcm::*`, no `subtle::*`, no signing / sealing / hashing of credentials, sessions, tokens, or outbound webhook payloads outside `ind-auth`. Consumers in driving adapters and other driven adapters call `ind-auth` helpers; they do not implement primitives.

### 5. Content-addressing hashes are allowlisted

`sha2::*` is permitted in `ind-application` and `ind-integrations` only for stable content hashes (cache keys, outbox dedup keys, export hashes) — never for credentials, sessions, signatures, or anything that protects access. Explicit allowlist:

- `crates/ind-application/src/recovery_keys.rs` — job-recovery dedup
- `crates/ind-application/src/services/tts/cache_key.rs` — TTS cache keys
- `crates/ind-application/src/services/tts/synthesis/hash.rs` — TTS synthesis hashes
- `crates/ind-application/src/services/tts/persona/mod.rs` — persona content hashes
- `crates/ind-application/src/handlers/provided_content.rs` — content-addressed staging keys for provided-content asset uploads
- `crates/ind-application/src/handlers/article_toc.rs` — content-addressed keys for prepared readable HTML and derived ToC payloads
- `crates/ind-application/src/handlers/library_upload.rs` — content-addressed manual upload identity and storage keys
- `crates/ind-integrations/src/obsidian/hash.rs` — Obsidian export content hashes

Any new `sha2` site outside the allowlist requires either an allowlist entry in this section (with a one-line rationale) or moving the function into `ind-auth`.

### 6. `std::env::var(...)` lives only in app config modules

Env reads are restricted to `apps/*/src/config.rs` and `apps/*/src/config/*.rs`. Library crates parse typed config structs passed in at construction time. Every runtime toggle is a typed field with a default and a unit test that exercises the override path.

`std::env::temp_dir()` is not covered by this rule (it is a filesystem-location lookup, not a configuration read).

**Test-only exception:** `#[ignore]`d integration tests in library crates that exercise external services (MinIO, Postgres-backed adapters) may read env vars locally to assemble a test config struct, because they are not production code paths. The exception covers `#[cfg(test)] mod tests` blocks gated by `#[ignore]`, nothing else.

### 7. HTTP types stay outside the hexagon

`axum::*`, `http::Method`, `http::StatusCode`, `utoipa::*`, route attributes, and any other transport-layer type are not allowed in `ind-domain` or `ind-application`. Application ports (trait definitions) live in `ind-application::ports` and are consumed by handlers. Errors cross the boundary via the `AppError` enum, not `StatusCode`. `AppState` only holds `Arc<dyn Port>` references — no `pub trait XxxOperations` in `ind-http-api/src/state.rs` or any route module.

### 8. File-size hard reject

Any non-generated, non-test source file over 600 LOC is a reviewer reject. Existing offenders are grandfathered; do not add new ones. See the "File-Size Discipline" section below for per-file-kind caps and split strategies.

### 9. `#[allow(dead_code)]` requires a backlog ID

Every occurrence of `#[allow(dead_code)]` must reference a real backlog ticket that resolves in `.backlog/` - either a comment (`// allow: TASK-123 - wired up in cluster D`) or an attribute argument. Bare allows with no ticket are a reject. The same rule applies to any `pub` item in `ind-api` / `ind-http-api`: if a field is unread it gets deleted or wired up, never silenced.

### 10. Ports return application outputs, not HTTP views

Application port methods return domain entities or transport-neutral application read models (`*Output` in `ind-application`). They must not return HTTP DTO/view types, import `ind-http-api`, or re-export `views::*` through `ind-application::ports`. HTTP response projection belongs in `ind-http-api/src/routes/<domain>/dto*`.

### 11. `serde_json::Value` field allowlist

A `serde_json::Value` field on an `ind-domain` type is acceptable only when it wraps a foreign payload at the edge of an event/job envelope, captures sparse extension/provider data with no stable shape, or is the value slot of a validated expression DSL whose external JSON shape must remain compatible. Every occurrence must appear in this allowlist with a one-line rationale. New `Value` fields without an allowlist entry are a blocker.

Allowlist:

- `crates/ind-domain/src/ai.rs` — `AiActionKind::job_payload` returns a job-envelope payload; `AiOutput::content` stores output-type-specific provider results.
- `crates/ind-domain/src/archive.rs` — `ReadingProgress::scroll_position` stores renderer/client-specific reader position data.
- `crates/ind-domain/src/billing.rs` — plan entitlements/quotas, entitlement snapshots, usage-event units/metadata, and billing-event payloads mirror provider/commercial documents whose shape varies by product or event type.
- `crates/ind-domain/src/document.rs` — `UserDocumentState::scroll_position` stores renderer/client-specific reader position data (EPUB CFI, percentage, pixel offset) with no single stable shape; migrated from `ReadingProgress::scroll_position`.
- `crates/ind-domain/src/integration.rs` — connection config, OAuth token extras, and import provider reports are provider-specific extension data.
- `crates/ind-domain/src/item.rs` — `Item::source_metadata` stores capture/import source metadata with source-specific keys.
- `crates/ind-domain/src/notification.rs` — `Notification::data` is the push-provider data payload.
- `crates/ind-domain/src/ops/events.rs` — domain event, outbox, recovery, lifecycle metadata, and dead-letter payloads are event/job envelopes.
- `crates/ind-domain/src/ops/jobs.rs` — `GenericJobEnvelope::payload` carries typed job payloads across the worker boundary.
- `crates/ind-domain/src/smart_list.rs` — `FilterNode::Condition.value` is a validated heterogeneous DSL operand and preserves the existing API/persisted JSON shape.
- `crates/ind-domain/src/tts/mod.rs` — `TtsVoicePersona::pronunciation_prefs` stores provider/persona pronunciation preferences with sparse shape.
- `crates/ind-domain/src/webhook.rs` — `WebhookDelivery::payload` stores the domain-event payload captured for outbound delivery.

---

## DTO Placement

DTOs and view-models live in `routes/<domain>/dto.rs`. Never in `state.rs`, `adapters.rs`, `main.rs`, or inline in a route handler. Anything with `#[derive(Serialize)]` + `ToSchema`, or any struct returned via `Json<T>` / `IntoResponse`, is a DTO and must live in the `dto.rs` sibling of its route module. Mapping `domain entity → DTO` is a free function in that same `dto.rs`. Only `dto.rs` — no `view.rs`, no `presenter.rs`, no parallel filenames.

---

## Adapters Are Wires, Not Logic

A type in `ind-api/src/adapters/` may only:

1. Hold `Arc`s of application services / repos.
2. Forward method calls (optionally adapting the args / return types to the HTTP port shape).
3. Map `AppError` variants if the port requires a different error type.

An adapter must not contain SQL, crypto, view-projection helpers, format-rendering (Obsidian / Markdown / etc.), business validation, or orchestration across multiple services. Each `adapters/<domain>.rs` covers one bounded subdomain (auth, item, integration, …) and stays under ~600 LOC; if it grows past that, split the subdomain.

---

## File-Size Discipline

These are soft caps — if you breach one, split before merging:

| File kind | Cap | Split strategy |
|---|---|---|
| `routes/<domain>/mod.rs` | 500 LOC | Move handlers to `routes/<domain>/<handler>.rs` |
| `routes/<domain>/dto.rs` | 600 LOC | Split into `requests.rs` + `responses.rs` |
| `adapters/<domain>.rs` | 600 LOC | Split subdomain into sibling files |
| `ports/<domain>/` in `ind-application` | 600 LOC per file | Split by collaborator role into sibling modules |
| `<service>.rs` in `ind-application` | 800 LOC | Split by use-case |
| `main.rs` | 100 LOC | Move wiring to `bootstrap.rs`, `services.rs`, `router.rs` |
| `state.rs` | 400 LOC | Move trait defs to `ind-application::ports`; bundle `AppState` fields |
| Any single `impl` block | 300 LOC | Group methods into trait impls or split |
| Any single fn | 80 LOC | Extract helpers |
| Any struct | 20 fields | Group into nested bundles |

Caps apply to *committed* code. Generated code (sqlx, openapi) is exempt. Inline SQL inside `ind-persistence` is also exempt when it is the reason a repository file exceeds the soft cap; do not move SQL into `query_file!` / `query_file_as!` files to satisfy LOC targets. Port modules may be directories (`ports/<domain>/...`); apply the cap to each source file, not to the directory aggregate.

---

## Passthrough Adapters Must Earn Their Weight

A trait-impl that only does `Box::pin(self.0.method(args))` for every method is not pulling weight. If an adapter has no logic beyond forwarding, either (a) inject the service directly into `AppState`, or (b) implement the port trait directly on the service in `ind-application` and skip the adapter. Don't ship zero-value wrappers.

---

## Bootstrap Layout

`ind-api/src/main.rs` is composition only:

1. Load config (`ServerConfig::load`).
2. Init tracing.
3. Build services via `services::build(&config, &pool) -> ServiceBundle`.
4. Build router via `router::build(state) -> Router`.
5. Serve.

No inline service construction, no inline env reads, no OAuth / TTS / integration registry construction. Those live in `bootstrap.rs` / `services.rs` / `router.rs` modules under `ind-api/src/`.

## Repository Composition Roots

Repeated repository construction is centralized in app-local `Repositories` structs (`apps/ind-api/src/services/repositories.rs` and `apps/ind-worker/src/repositories.rs`). If a `Pg*Repository` type is seeded there, construct it only there for that binary and pass cloned `Arc`s to service/job builders. Single-use, non-seeded repos may stay local until they become duplicated.

## Test Corpus Ownership

Every product invariant has one authoritative test layer. Domain tests own validation and serialization; application tests own orchestration; persistence tests own SQL, transactions, ordering, isolation, and concurrency; HTTP tests own transport contracts; workers own dispatch and recovery outcomes. A higher layer may keep one wiring smoke, but must not repeat lower-layer state assertions.

The test LOC budget counts every Rust line in dedicated test/support files and the complete syntax spans of inline test-only items. Moving helpers, fixtures, fakes, or assertions between those locations does not reduce the total. Non-Rust fixtures are reported separately and must not contain executable test logic.

Before adding a test:

1. Name the invariant and its authoritative layer.
2. Search for an existing owner and extend or table-drive it when the setup/action/assertion shape is shared.
3. Use the smallest real boundary that proves the behavior; persistence and route contracts still require the shared integration harness.
4. Assert observable outcomes rather than every intermediate field.

Critical owners live in `backend/test-invariants.toml`. Changing or removing one requires assigning the same invariant to a surviving authoritative test in the same change; `scripts/verify-test-invariants.sh` fails CI when a pinned test no longer exists.

Test files are capped at 350 counted lines and shared `ind-test-support` files at 500; splitting is by scenario or dependency ownership, never by arbitrary line count. Route tests may not implement broad `*Operations` or `*Port` traits to rebuild `AppState` locally.

Test-writing decision tree:

1. If the compiler proves it, do not test it.
2. For pure input/output, extend a table or property test.
3. For a SQL invariant, use one shared real-database scenario.
4. For a public HTTP contract, use one real-app smoke.
5. For provider mapping, use shared conformance plus provider-specific cases.
6. For a cross-layer journey, keep one focused end-to-end scenario.

When adding coverage for a registered risk, delete the duplicate owner in the same change. Run `scripts/backend-test-confidence-report.sh` quarterly to expose the largest owners, crate totals, fixture bytes, runtime, coverage, and the pinned critical mutation sample.
