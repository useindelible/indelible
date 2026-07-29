# syntax=docker/dockerfile:1.7
# Build context is the repository root: this image carries both the API and the
# web app so a self-hosted instance serves them from one origin, which is what
# the browser extension and the SPA's same-origin API base already assume.

FROM node:22-bookworm-slim AS web
RUN corepack enable
WORKDIR /src/web
COPY web/package.json web/pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile
# The reader imports ../../../../../shared/highlight-source, so shared/ has to
# keep its position as a sibling of web/ for the build to resolve it.
COPY shared/ /src/shared/
COPY web/ ./
# No VITE_API_BASE_URL: the production build falls back to the empty base, so
# the SPA calls the same origin it was served from.
RUN pnpm exec svelte-kit sync && pnpm build

FROM rust:1.94-bookworm AS chef
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo install cargo-chef --locked
WORKDIR /app

FROM chef AS planner
COPY backend/ .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo chef cook --release --recipe-path recipe.json
COPY backend/ .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo build --release -p ind-api && \
    cp /app/target/release/ind-api /usr/local/bin/ind-api

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y \
    ca-certificates \
    fonts-dejavu-core \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -u 1000 app
WORKDIR /app
COPY --from=builder /usr/local/bin/ind-api ./ind-api
COPY --from=web /src/web/build ./web
RUN chown -R app:app /app

USER app

ENV IND_HOST=0.0.0.0
ENV IND_PORT=8080
ENV WEB_ROOT=/app/web

EXPOSE 8080
CMD ["./ind-api"]
