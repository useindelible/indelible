# syntax=docker/dockerfile:1.7
FROM rust:1.94-bookworm AS chef
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo install cargo-chef --locked
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo chef cook --release --bin ind-renderer --recipe-path recipe.json
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo build --release -p ind-renderer && \
    cp /app/target/release/ind-renderer /usr/local/bin/ind-renderer

# Pinned so a release rebuild gets the same Chrome as the main-channel build
# of the same commit. Bump deliberately alongside renderer testing.
FROM chromedp/headless-shell:151.0.7922.109

RUN apt-get update && apt-get install -y \
    fonts-liberation \
    fonts-noto-core \
    fonts-noto-cjk \
    fonts-noto-color-emoji \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -u 1000 app

WORKDIR /app
COPY --from=builder --chown=app:app /usr/local/bin/ind-renderer ./ind-renderer

USER app

ENV CHROMIUM_PATH=/headless-shell/headless-shell
ENV RENDERER_HOST=0.0.0.0
ENV RENDERER_PORT=3100
ENV TZ=Etc/UTC

EXPOSE 3100
ENTRYPOINT []
CMD ["./ind-renderer"]
