# plutus deployment

Two deployment styles ship in this repo:

1. **`compose.dev.yml` / `compose.yml`** — Compose Spec, works with
   `docker compose` and `podman compose`. Best for local-machine dev
   and single-host stacks.
2. **`scripts/bootstrap.sh` + `scripts/deploy.sh`** — remote podman host
   with Quadlet (`~/.config/containers/systemd/*.container`) +
   `systemctl --user`. Best for "my home server".

## Quick start: remote podman host

### First time

```bash
DEPLOY_HOST=noah@10.1.2.51 ./scripts/bootstrap.sh
```

This SSHes to the target, enables lingering, seeds
`~/podman-config/plutus/.env`, writes four Quadlet files
(`plutus.network`, `plutus-postgres.container`, `plutus-api.container`,
`plutus-web.container`), then hands off to `deploy.sh --all` for the
initial build + start.

After it finishes, the web UI is at <http://10.1.2.51:4100/login>
(admin login: `noah` / `vz1234` — change in
`~/podman-config/plutus/.env` after first run).

### Subsequent deploys

```bash
./scripts/deploy.sh                 # rebuild api + web, restart, verify
./scripts/deploy.sh --only api      # backend changed only
./scripts/deploy.sh --only web      # frontend changed only
./scripts/deploy.sh --all           # also rebuild postgres (rarely needed)
./scripts/deploy.sh --skip-build    # config/.env tweak — just restart
```

`deploy.sh` rsyncs source, builds the affected image(s) on the remote,
restarts the matching `systemctl --user` services, then probes
`/login` to confirm the chain is up. Typical full deploy is ~30s
warm cache, ~3-5min cold.

### Operations

```bash
# status / logs
ssh noah@10.1.2.51 'systemctl --user list-units "plutus-*"'
ssh noah@10.1.2.51 'journalctl --user -u plutus-api.service -f'

# psql into the running postgres
ssh noah@10.1.2.51 'podman exec -it plutus-postgres psql -U plutus -d plutus'
```

## Local development (compose)

Run postgres in a container, the API and web on the host (so you get hot reload).

```bash
docker compose -f deploy/compose.dev.yml up -d
# or: podman compose -f deploy/compose.dev.yml up -d

# In another terminal, with .env populated:
cargo run -p plutus-server -- migrate
cargo run -p plutus-server -- serve

# And a third for the web app:
cd web && pnpm install && pnpm dev
```

## Single-host full stack (compose)

```bash
docker compose -f deploy/compose.yml up -d
# or: podman compose -f deploy/compose.yml up -d
```

Visit the web UI at <http://127.0.0.1:3000> and the API at <http://127.0.0.1:8080>.

## Image notes

- **`postgres/Dockerfile`** — pgvector 0.8 + Apache AGE 1.7 on PG18.
- **`api/Dockerfile`** — multi-stage `rust:1.95-slim-bookworm` build →
  `debian:bookworm-slim` runtime with just `tini` + the `plutus` binary.
- **`web/Dockerfile`** — `node:24-trixie-slim` (need glibc 2.38+ for
  uWebSockets.js's prebuilt binary; bookworm 2.36 is too old).

To rebuild postgres after init.sql or Dockerfile changes:

```bash
docker compose -f deploy/compose.dev.yml build postgres
```

## Resetting state

```bash
docker compose -f deploy/compose.dev.yml down -v   # wipes pgdata
```
