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

`scripts/backup.sh` uses **`pg_dump --format=custom`** — pg_dump's
native binary container. Each run produces two files under
`~/podman-volume/plutus-backups/`:

| file | format | role |
|---|---|---|
| `plutus-YYYY-MM-DD-HHMM.dump` | binary, compressed | data backup (one per run) |
| `plutus-globals.sql` | plain SQL | roles + role passwords (refreshed each run, single current copy) |

The `.dump` is `pg_restore`-compatible: supports parallel restore
(`pg_restore -j N`), selective restore (`pg_restore -t users`),
schema-only or data-only modes, and is the de-facto "binary backup"
for logical pg_dump.

Why globals are separate plain text: `pg_dumpall --globals-only`
only emits text; roles barely change so we keep ONE current copy
instead of 14 redundant ones. The role's password hash is also in
your `.env` and recreated by `bootstrap.sh`, so the globals file is a
safety net for the rare case of cold migration without `.env`.

Retention: newest 14 `.dump` files (override `RETAIN=N`).
`plutus-globals.sql` is rewritten in place every run.

Designed for **migration**: ship a `.dump` to any postgres host,
`pg_restore` it.

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

Role and database are already there. One command replaces the data:

```bash
cat plutus-2026-05-21-0330.dump | \
  ssh noah@10.1.2.51 'podman exec -i plutus-postgres pg_restore \
      -U plutus -d plutus --clean --if-exists'

# Re-run migrate so anything outside pg_dump's scope (toasty schema
# diff) lines back up. Idempotent.
ssh noah@10.1.2.51 'systemctl --user restart plutus-api.service'
```

`--clean --if-exists` drops every restored object before recreating
it, so the restore is idempotent against an already-populated DB.

### Restore — fresh server (migration)

```bash
# 1. Stand up the new host (postgres container, role, empty DB, all
#    three services). bootstrap.sh creates the `plutus` role via the
#    postgres container init using POSTGRES_USER/POSTGRES_PASSWORD
#    from .env.
DEPLOY_HOST=user@new-host ./scripts/bootstrap.sh

# 2. Stream the .dump into the new postgres.
cat plutus-2026-05-21-0330.dump | \
  ssh user@new-host 'podman exec -i plutus-postgres pg_restore \
      -U plutus -d plutus --clean --if-exists'

# 3. Restart API so toasty re-syncs its schema diff.
ssh user@new-host 'systemctl --user restart plutus-api.service'
```

If you lost `.env` too (so the role can't be recreated by bootstrap
with the original password), restore globals first:

```bash
cat plutus-globals.sql | \
  ssh user@new-host 'podman exec -i plutus-postgres \
      psql -U postgres -d postgres'
# Then continue with step 2 above.
```

**Verified**: round-trip tested by restoring a `.dump` into a fresh
`plutus-postgres:18` container — 44 tables, 181 indexes, both age +
vector extensions all came back. See `scripts/backup.sh` header
comment for exact restore commands.

### Selective / inspection restore

The `.dump` is a binary archive; use `pg_restore` to drill in.

```bash
# List the archive contents (what's inside, no actual restore)
podman exec -i plutus-postgres pg_restore --list < plutus-...dump

# Restore one table only (drops + recreates that table)
podman exec -i plutus-postgres pg_restore \
    -U plutus -d plutus -t transactions --clean --if-exists \
  < plutus-...dump

# Restore just schema, no data
podman exec -i plutus-postgres pg_restore \
    -U plutus -d plutus --schema-only \
  < plutus-...dump

# Convert the binary dump back to plain SQL (e.g. to grep through)
podman exec -i plutus-postgres pg_restore -f - < plutus-...dump | less
```
