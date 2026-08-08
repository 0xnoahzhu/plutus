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

# ── Dependency fetch ─────────────────────────────────────────────────────
# Manifests only. This layer is keyed on the lockfile and the member
# Cargo.tomls, so it survives ordinary source edits.
#
# It exists because the previous layout copied all of `crates/` before
# building, which meant every code change invalidated the build layer and
# re-downloaded the entire crate graph. On a flaky link that turns a
# one-line change into a coin flip — two deploys in a row died here with
# "Timeout was reached (less than 10 bytes/sec)" partway through the
# download.
COPY Cargo.toml Cargo.lock ./
COPY crates/plutus-core/Cargo.toml    crates/plutus-core/Cargo.toml
COPY crates/plutus-storage/Cargo.toml crates/plutus-storage/Cargo.toml
COPY crates/plutus-api/Cargo.toml     crates/plutus-api/Cargo.toml
COPY crates/plutus-server/Cargo.toml  crates/plutus-server/Cargo.toml

# Cargo won't parse a workspace member whose target file is missing, and
# `cargo fetch` parses before it downloads. Stubs satisfy that; the real
# sources land in the next layer and overwrite them.
RUN mkdir -p crates/plutus-core/src crates/plutus-storage/src \
             crates/plutus-api/src crates/plutus-server/src \
 && touch crates/plutus-core/src/lib.rs \
          crates/plutus-storage/src/lib.rs \
          crates/plutus-api/src/lib.rs \
 && echo 'fn main() {}' > crates/plutus-server/src/main.rs \
 && cargo fetch --locked

# ── Build ────────────────────────────────────────────────────────────────
# Real sources replace the stubs. `--offline` makes the network
# irrelevant from here on: everything needed is already in the registry
# cache from the layer above, so a compile can't die halfway through on a
# download. If something genuinely is missing it fails immediately with a
# clear message instead of hanging on a 30-second timeout.
#
# Compilation itself deliberately is NOT cached across source changes —
# a stale-artifact bug would ship a binary that doesn't match the source,
# which is far worse than the couple of minutes it saves.
COPY crates ./crates
RUN cargo build --release --offline --bin plutus \
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
# long-running command.
#
# Migrations are NOT run here. `plutus-api.container`'s ExecStartPre runs
# `plutus migrate` in a throwaway container from this same image before
# every start (see scripts/bootstrap.sh), so a deploy that restarts the
# service always migrates first. Don't add a migrate step to deploy.sh —
# it would just run the same idempotent work twice.
ENTRYPOINT ["/usr/bin/tini", "--"]
CMD ["plutus", "serve"]
