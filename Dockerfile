# syntax=docker/dockerfile:1

# -----------------------------------------------------------------------------
# Builder — compile release binary with full toolchain
# -----------------------------------------------------------------------------
FROM rust:1-bookworm AS builder

RUN apt-get update \
    && apt-get install -y --no-install-recommends protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY rust-toolchain.toml Cargo.toml Cargo.lock ./
COPY .cargo ./.cargo
COPY sqlx.toml ./
COPY .sqlx ./.sqlx
COPY migrations ./migrations
COPY crates ./crates

ENV SQLX_OFFLINE=true
RUN cargo build --release -p horizon-server

# -----------------------------------------------------------------------------
# Runtime — minimal attack surface (glibc + libgcc only)
# -----------------------------------------------------------------------------
FROM gcr.io/distroless/cc-debian12

WORKDIR /app

COPY --from=builder /app/target/release/horizon-server /app/horizon-server

EXPOSE 50051

USER nonroot:nonroot

ENTRYPOINT ["/app/horizon-server"]
