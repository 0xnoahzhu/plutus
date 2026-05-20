## Multi-stage build for the plutus Rust workspace.
##
## - Builder: rust:slim, builds the `plutus` binary from the plutus-server
##   crate. Build deps (pkg-config, libssl-dev) are needed for some
##   crates; everything else compiles from source.
## - Runtime: debian:bookworm-slim with just the bits the binary needs
##   (ca-certificates for TLS, libssl3 for native TLS, no toolchain).
##
## Build from the repo root (the build context is the workspace so we
## can copy every member crate):
##   podman build -f deploy/api/Dockerfile -t plutus-api:latest .

# ── Builder ──────────────────────────────────────────────────────────────
FROM docker.io/library/rust:1.95-slim-bookworm AS builder

WORKDIR /build

# Build-time deps. `pkg-config` + `libssl-dev` cover most crates that
# bind to system OpenSSL; `build-essential` gives us a C linker for any
# `*-sys` crate that needs one.
RUN apt-get update \
 && apt-get install -y --no-install-recommends \
        build-essential \
        pkg-config \
        libssl-dev \
        ca-certificates \
 && rm -rf /var/lib/apt/lists/*

# Copy the workspace. Bring in the lockfile + every member crate so
# cargo can resolve and build from the same graph it uses locally.
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Release build. The binary name is `plutus` (declared in
# plutus-server/Cargo.toml as `[[bin]] name = "plutus"`).
RUN cargo build --release --bin plutus \
 && strip target/release/plutus

# ── Runtime ──────────────────────────────────────────────────────────────
FROM docker.io/library/debian:bookworm-slim AS runtime

# Runtime deps:
#  - ca-certificates: TLS trust store for any outbound HTTPS the server makes
#  - libssl3:         dynamic linker target for OpenSSL-backed crates
#  - tini:            small init so Ctrl-C / systemd stops propagate cleanly
RUN apt-get update \
 && apt-get install -y --no-install-recommends \
        ca-certificates \
        libssl3 \
        tini \
 && rm -rf /var/lib/apt/lists/*

# Non-root runtime user. Postgres credentials and bind addr come from
# the environment (set by the Quadlet / compose).
RUN groupadd --system plutus \
 && useradd  --system --gid plutus --home-dir /app --shell /usr/sbin/nologin plutus

WORKDIR /app
COPY --from=builder /build/target/release/plutus /usr/local/bin/plutus

USER plutus
EXPOSE 8080

# tini reaps zombies + forwards signals; `plutus serve` is the actual
# long-running command. Migrations are run as a separate
# `podman exec ... plutus migrate` step from the deploy script.
ENTRYPOINT ["/usr/bin/tini", "--"]
CMD ["plutus", "serve"]
