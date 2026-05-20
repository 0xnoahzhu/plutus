#!/usr/bin/env bash
# First-time setup of plutus on a fresh podman host.
#
# Idempotent — safe to re-run. Does NOT touch existing data: if the
# postgres volume already has a cluster, it's left alone. If Quadlet
# files already exist, they're overwritten (so re-running picks up
# template changes).
#
# What it does:
#   1. ssh-key trust + lingering (so user services survive reboot)
#   2. creates ~/podman-config/plutus/.env from a template (if missing)
#   3. writes the four Quadlet files (network + 3 containers)
#   4. ensures the host data dirs exist
#   5. delegates to `deploy.sh --all` for the actual image build + start
#
# Usage:
#   ./scripts/bootstrap.sh                  # uses defaults below
#
# Env overrides (same names as deploy.sh):
#   DEPLOY_HOST       default noah@10.1.2.51
#   DEPLOY_PATH       default ~/app/plutus
#   PLUTUS_ADMIN_USERNAME   default noah
#   PLUTUS_ADMIN_PASSWORD   default vz1234 (CHANGE THIS in .env after first run)

set -euo pipefail

HOST="${DEPLOY_HOST:-noah@10.1.2.51}"
REMOTE_PATH="${DEPLOY_PATH:-~/app/plutus}"
ADMIN_USER="${PLUTUS_ADMIN_USERNAME:-noah}"
ADMIN_PASS="${PLUTUS_ADMIN_PASSWORD:-vz1234}"

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

echo "==> bootstrap target: $HOST"

# ── 1. Lingering ─────────────────────────────────────────────────────────
echo
echo "==> ensure lingering (user services persist after logout/reboot)"
ssh "$HOST" 'loginctl enable-linger $(whoami) 2>/dev/null || true; loginctl show-user $(whoami) | grep Linger'

# ── 2. Host directories ──────────────────────────────────────────────────
echo
echo "==> create host directories"
ssh "$HOST" "mkdir -p ${REMOTE_PATH} ~/podman-config/plutus ~/podman-volume/plutus-pgdata ~/.config/containers/systemd && ls -d ${REMOTE_PATH} ~/podman-config/plutus ~/podman-volume/plutus-pgdata ~/.config/containers/systemd"

# ── 3. .env (only if not already present — never clobber secrets) ────────
echo
echo "==> seed ~/podman-config/plutus/.env (if missing)"
ssh "$HOST" "test -f ~/podman-config/plutus/.env" && {
  echo "    .env already exists — leaving it alone"
} || {
  ssh "$HOST" "cat > ~/podman-config/plutus/.env <<'ENV'
# Plutus deployment config — sourced by API container.
POSTGRES_USER=plutus
POSTGRES_PASSWORD=plutus
POSTGRES_DB=plutus

# Admin credentials. The admin account is NOT in the users table; these
# env vars are the source of truth. CHANGE THESE on a real deployment.
PLUTUS_ADMIN_USERNAME=${ADMIN_USER}
PLUTUS_ADMIN_PASSWORD=${ADMIN_PASS}

PLUTUS_BIND_ADDR=0.0.0.0:8080
PLUTUS_API_REQUIRE_AUTH=false

RUST_LOG=info,plutus=debug
ENV
chmod 600 ~/podman-config/plutus/.env"
  echo "    wrote .env"
}

# ── 4. Quadlet files ─────────────────────────────────────────────────────
echo
echo "==> write Quadlet files"

ssh "$HOST" "cat > ~/.config/containers/systemd/plutus.network <<'EOF'
[Unit]
Description=Plutus internal network (postgres + api + web)

[Network]
NetworkName=plutus
EOF"
echo "    plutus.network"

ssh "$HOST" "cat > ~/.config/containers/systemd/plutus-postgres.container <<'EOF'
[Unit]
Description=Plutus Postgres (pgvector + Apache AGE)
After=network-online.target
Wants=network-online.target

[Container]
Image=localhost/plutus-postgres:18
ContainerName=plutus-postgres
Network=plutus.network
EnvironmentFile=%h/podman-config/plutus/.env
# :U chowns the bind mount to the container's mapped uid (rootless-friendly).
Volume=%h/podman-volume/plutus-pgdata:/var/lib/postgresql:U
PublishPort=127.0.0.1:5432:5432
HealthCmd=pg_isready -U plutus -d plutus
HealthInterval=5s
HealthRetries=10
HealthStartPeriod=15s

[Service]
Restart=always
TimeoutStartSec=120

[Install]
WantedBy=default.target
EOF"
echo "    plutus-postgres.container"

ssh "$HOST" "cat > ~/.config/containers/systemd/plutus-api.container <<'EOF'
[Unit]
Description=Plutus API (axum + toasty)
After=network-online.target plutus-postgres.service
Wants=network-online.target plutus-postgres.service

[Container]
Image=localhost/plutus-api:latest
ContainerName=plutus-api
Network=plutus.network
EnvironmentFile=%h/podman-config/plutus/.env
Environment=DATABASE_URL=postgres://plutus:plutus@plutus-postgres:5432/plutus
PublishPort=127.0.0.1:8080:8080

[Service]
Restart=always
TimeoutStartSec=60
# Run migrate before serve on every start. Toasty push_schema + the
# post_migrate SQL are idempotent so re-running is safe.
ExecStartPre=/usr/bin/podman run --rm \\
    --network=plutus \\
    --env-file %h/podman-config/plutus/.env \\
    --env DATABASE_URL=postgres://plutus:plutus@plutus-postgres:5432/plutus \\
    localhost/plutus-api:latest plutus migrate

[Install]
WantedBy=default.target
EOF"
echo "    plutus-api.container"

ssh "$HOST" "cat > ~/.config/containers/systemd/plutus-web.container <<'EOF'
[Unit]
Description=Plutus Web (Remix SSR)
After=network-online.target plutus-api.service
Wants=network-online.target plutus-api.service

[Container]
Image=localhost/plutus-web:latest
ContainerName=plutus-web
Network=plutus.network
Environment=PLUTUS_API_URL=http://plutus-api:8080
Environment=NODE_ENV=production
Environment=PORT=4100
# Publish on all interfaces — this is the public-facing entry point.
PublishPort=4100:4100

[Service]
Restart=always
TimeoutStartSec=60

[Install]
WantedBy=default.target
EOF"
echo "    plutus-web.container"

# ── 5. Reload systemd and hand off to deploy.sh for the actual build ─────
echo
echo "==> reload systemd-user generator"
ssh "$HOST" 'systemctl --user daemon-reload'

echo
echo "==> hand off to deploy.sh --all (initial build of all 3 images + start)"
exec "$(dirname "$0")/deploy.sh" --all
