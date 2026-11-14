# Stage 1: Build React 19 + Vite + TanStack Router SPA (ui/dist)
FROM node:20-alpine AS ui-builder

WORKDIR /ui
COPY ui/package.json ui/pnpm-lock.yaml* ui/pnpm-workspace.yaml* ui/.npmrc* ./
RUN corepack enable && pnpm install --frozen-lockfile
COPY ui/ ./
RUN corepack pnpm build

# Stage 2: Compile static-friendly Rust/Axum server binary
FROM rust:1-bookworm AS rust-builder

WORKDIR /build
COPY server-rs/Cargo.toml server-rs/Cargo.lock* ./
COPY server-rs/src ./src
RUN cargo build --release --locked && strip /build/target/release/lnc-styles-gallery

# Stage 3: Hardened minimal non-root runtime (uid/gid 10001)
FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd -g 10001 styles \
    && useradd -u 10001 -g 10001 -M -s /usr/sbin/nologin styles \
    && mkdir -p /app /app/ui/dist /data/refero-styles /data/prompt-driven-ui /data/lnc-components /config \
    && chown -R 10001:10001 /app /data /config

ENV PORT=8000 \
    STYLES_HOST=0.0.0.0 \
    STYLES_PORT=8000 \
    STYLES_DATA_DIR=/data \
    STYLES_CONFIG_DIR=/config \
    STYLES_ROOT_DIR=/app

WORKDIR /app

COPY --chown=10001:10001 VERSION /app/VERSION
COPY --chown=10001:10001 --from=ui-builder /ui/dist /app/ui/dist
COPY --chown=10001:10001 --from=rust-builder /build/target/release/lnc-styles-gallery /app/lnc-styles-gallery

USER 10001:10001

EXPOSE 8000

HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD ["/app/lnc-styles-gallery", "--healthcheck"]

CMD ["/app/lnc-styles-gallery"]
