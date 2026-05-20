#!/usr/bin/env bash
# Full logical backup of the plutus postgres cluster.
#
# Designed for migration: a single .sql.gz that can rebuild the cluster
# from scratch on a fresh server. Two `pg_dump*` invocations concatenated:
#
#   1. `pg_dumpall --globals-only`   roles + tablespaces + role passwords.
#      Lets the target server recreate the `plutus` role itself before
#      restoring data. The bootstrap `postgres` role will already exist
#      on any cluster — CREATE ROLE for it will warn "already exists"
#      and psql moves on (we don't pass --ON_ERROR_STOP).
#
#   2. `pg_dump --create --clean --if-exists` for the `plutus` database.
#      --create     emits `CREATE DATABASE plutus ...` so the dump is
#                   self-contained (you don't have to pre-create the DB).
#      --clean      emits `DROP TABLE / DROP DATABASE` lines so restore
#                   is idempotent against a populated target.
#      --if-exists  guards the DROPs so a truly empty target doesn't
#                   error before getting to the CREATEs.
#
# Plain SQL output, gzip-compressed. Plain because at personal scale the
# backup is tiny and we get `psql`-restorable files (no pg_restore
# needed), and `zgrep` works on them.
#
# Output: ~/podman-volume/plutus-backups/plutus-YYYY-MM-DD-HHMM.sql.gz
# Retention: keeps the most recent $RETAIN backups (default 14), deletes
# older. The directory is on the same volume as pgdata — a disk failure
# loses both. See "Off-site copies" in deploy/README.md for syncing
# backups elsewhere (rsync to another host or to your laptop).
#
# Usage (on the server):
#   ~/app/plutus/scripts/backup.sh
#   RETAIN=30 ~/app/plutus/scripts/backup.sh    # keep last 30
#
# Usage (from your dev machine):
#   ssh noah@10.1.2.51 'bash ~/app/plutus/scripts/backup.sh'
#
# Scheduling: install deploy/systemd/plutus-backup.{service,timer}, then
#   systemctl --user enable --now plutus-backup.timer
# Runs daily at the time set in the .timer file.
#
# Restore — full cluster rebuild on a fresh server:
#   gunzip -c plutus-YYYY-MM-DD-HHMM.sql.gz \
#     | podman exec -i plutus-postgres psql -U postgres -d postgres
# (Connect as `postgres` so the role + DB creation statements can run.)

set -euo pipefail

CONTAINER="${PLUTUS_PG_CONTAINER:-plutus-postgres}"
DB_USER="${PLUTUS_PG_USER:-plutus}"
DB_NAME="${PLUTUS_PG_DB:-plutus}"
BACKUP_DIR="${PLUTUS_BACKUP_DIR:-$HOME/podman-volume/plutus-backups}"
RETAIN="${RETAIN:-14}"

mkdir -p "$BACKUP_DIR"

# UTC timestamp so backups sort lexicographically regardless of TZ.
TS=$(date -u +%Y-%m-%d-%H%M)
OUT="$BACKUP_DIR/plutus-${TS}.sql.gz"

echo "==> full backup of $CONTAINER → $OUT"

# Globals first (roles), then the database (schema + data). Both streams
# are valid SQL; concatenating them produces a single self-contained
# restore script.
{
  echo "-- ──────────────────────────────────────────────────────────────"
  echo "-- 1/2: globals (roles, tablespaces) — pg_dumpall --globals-only"
  echo "-- ──────────────────────────────────────────────────────────────"
  podman exec "$CONTAINER" pg_dumpall \
      --username="$DB_USER" \
      --globals-only
  echo
  echo "-- ──────────────────────────────────────────────────────────────"
  echo "-- 2/2: database '$DB_NAME' — pg_dump --create --clean --if-exists"
  echo "-- ──────────────────────────────────────────────────────────────"
  # No --no-owner / --no-privileges: this is a FULL backup intended to
  # roundtrip on the same role names. Strip them at restore time if you
  # really need to move to different role names.
  podman exec "$CONTAINER" pg_dump \
      --username="$DB_USER" \
      --dbname="$DB_NAME" \
      --format=plain \
      --create \
      --clean \
      --if-exists
} | gzip --best > "$OUT"

# Empty / tiny output means something went wrong (pg_dump errored but
# gzip happily compressed the empty stream). Refuse to count it as a
# successful backup.
size=$(stat -c%s "$OUT" 2>/dev/null || stat -f%z "$OUT")
if (( size < 1024 )); then
  echo "ERROR: backup is $size bytes — something went wrong, refusing to keep it" >&2
  rm -f "$OUT"
  exit 1
fi

# Pretty-print size
human=$(awk -v b="$size" 'BEGIN{
  split("B KB MB GB",u);
  s=1;
  while (b >= 1024 && s < 4) { b /= 1024; s++ }
  printf "%.1f %s\n", b, u[s]
}')
echo "    wrote ${OUT##*/} ($human)"

# Rotate: newest first, delete index $RETAIN onwards.
mapfile -t backups < <(ls -t "$BACKUP_DIR"/plutus-*.sql.gz 2>/dev/null)
total=${#backups[@]}
echo "    total backups: $total (retaining newest $RETAIN)"

if (( total > RETAIN )); then
  for ((i = RETAIN; i < total; i++)); do
    rm -f "${backups[i]}"
    echo "    deleted ${backups[i]##*/}"
  done
fi

echo "==> done"
