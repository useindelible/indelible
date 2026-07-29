# Web Frontend

## API SDK

The web app consumes a generated SDK from the backend OpenAPI document.

- Source of truth: `backend/crates/ind-http-api`
- Generated output: `src/lib/api/generated/`
- Runtime wiring: `src/lib/api/client.ts`
- Public frontend API surface: `src/lib/api/index.ts`
- Cross-client policy: `../docs/api-client-generation.md`

Regenerate the SDK after backend API changes:

```sh
pnpm api:generate
```

What that command does:

1. Exports the current OpenAPI document from the Rust backend.
2. Generates the TypeScript SDK with `@hey-api/openapi-ts`.
3. Formats the generated files.

Rules:

- Do not hand-edit files under `src/lib/api/generated/`.
- If frontend code needs a new endpoint or shape, update the backend OpenAPI source and regenerate.
- App code should import from `$lib/api`, not generated `client.gen`/`sdk.gen` files and not ad hoc `fetch`/`axios`/`ky` calls.
- In local dev, the frontend defaults API requests to `http://localhost:38473` when `VITE_API_BASE_URL` is not set.
