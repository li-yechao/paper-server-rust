# syntax = docker/dockerfile:1.0-experimental
FROM liyechao/rust:1.53.0-0-musl AS build

ARG TARGETPLATFORM

WORKDIR /app

COPY . /app

RUN --mount=type=cache,target=/usr/local/cargo/registry,id=cargo_cache \
    --mount=type=cache,target=/app/target,id=paper-graphql-target \
    set -eux; \
    case "$TARGETPLATFORM" in \
    # amd64
    linux/amd64) \
    cargo build \
    -p paper_graphql \
    --bin paper \
    --release \
    --target x86_64-unknown-linux-musl; \
    cp target/x86_64-unknown-linux-musl/release/paper /; \
    ;; \
    # aarch64
    linux/arm64) \
    cargo build \
    -p paper_graphql \
    --bin paper \
    --release \
    --target aarch64-unknown-linux-musl; \
    cp target/aarch64-unknown-linux-musl/release/paper /; \
    ;; \
    *) echo >&2 "unsupported architecture: $TARGETPLATFORM"; exit 1 ;; \
    esac

FROM alpine:3.12

WORKDIR /app

COPY --from=build /paper .
COPY graphql/sample.cfg ./paper.cfg

CMD ["./paper", "-c", "./paper.cfg"]
