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

## Backups

`scripts/backup.sh` produces a **self-contained full-cluster dump**:

1. `pg_dumpall --globals-only` for roles + tablespaces (so a fresh
   target server can recreate the `plutus` role with its original
   password hash)
2. `pg_dump --create --clean --if-exists` for the `plutus` database
   itself (so the dump includes `CREATE DATABASE plutus` and is
   restorable without any pre-setup)

Concatenated into one `.sql.gz`, dropped under
`~/podman-volume/plutus-backups/`, rotated to newest 14 (override with
`RETAIN=N`).

Designed for **migration**: a single file you can ship to any
postgres host and replay to rebuild the cluster from scratch.

### Manual

```bash
# On the server
~/app/plutus/scripts/backup.sh

# From your laptop
ssh noah@10.1.2.51 'bash ~/app/plutus/scripts/backup.sh'
```

### Scheduled (daily 03:30)

```bash
mkdir -p ~/.config/systemd/user
cp ~/app/plutus/deploy/systemd/plutus-backup.{service,timer} \
   ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable --now plutus-backup.timer

# Verify
systemctl --user list-timers plutus-backup.timer
journalctl --user -u plutus-backup.service -n 30
```

### Off-site copies

Backups sit on the same volume as `pgdata`, so a disk failure kills
both. Sync them off-host periodically:

```bash
# Pull all backups to your laptop (one-time or via cron)
rsync -avz noah@10.1.2.51:~/podman-volume/plutus-backups/ \
            ~/backups/plutus/
```

### Restore — same host

The dump drops and recreates `plutus` itself (`--clean --if-exists
--create`), so you don't pre-create anything. Connect as `postgres`
so the role + DB statements have privilege to run:

```bash
gunzip -c ~/backups/plutus/plutus-2026-05-21-0330.sql.gz \
  | ssh noah@10.1.2.51 'podman exec -i plutus-postgres psql -U postgres -d postgres'

# Re-run migrate so anything outside pg_dump's scope (toasty schema diff)
# lines back up. Idempotent.
ssh noah@10.1.2.51 'systemctl --user restart plutus-api.service'
```

### Restore — fresh server (migration)

The point of the full backup. On a brand-new podman host with
`plutus-postgres` running but no `plutus` role or database:

```bash
gunzip -c plutus-2026-05-21-0330.sql.gz \
  | ssh noah@new-host 'podman exec -i plutus-postgres psql -U postgres -d postgres'
```

A `CREATE ROLE postgres` line will warn "already exists" (the
bootstrap superuser is there) — psql doesn't have `ON_ERROR_STOP` set
so it moves on. The `plutus` role is created with its original
password hash, then `CREATE DATABASE plutus` runs, then the schema +
data follows.

After restore: deploy the app pointing at the new host
(`DEPLOY_HOST=user@new-host ./scripts/bootstrap.sh` if the host
doesn't have plutus yet, or `./scripts/deploy.sh` if it does).

**Verified**: round-trip tested by restoring a backup into a fresh
container — 44 tables, 3 extensions (age, vector, plpgsql), `plutus`
role all came back. See `scripts/backup.sh` header comment for the
exact restore command.

### Selective restore (no full DB rebuild)

The dump is plain SQL — `zgrep` any chunk you want:

```bash
# Just the row data for one table
zgrep -A 9999 "^COPY public.transactions" backup.sql.gz \
  | sed '/^\\\\\.$/q'

# Hand-craft a partial restore
zgrep -A2 "INSERT INTO users" ~/backups/plutus/plutus-2026-05-21-0330.sql.gz
```
