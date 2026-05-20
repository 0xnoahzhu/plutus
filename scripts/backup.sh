#!/usr/bin/env bash
# Logical backup of the plutus postgres cluster — pg_dump custom format.
#
# Each backup = one `.dump` file. Custom format is pg_dump's native
# binary container: built-in compression (zstd-level by default),
# pg_restore-compatible, supports parallel restore (`pg_restore -j N`),
# and selective restore (`pg_restore -t table_name`). The de-facto
# "binary backup" for logical pg_dump.
#
# Output files in ~/podman-volume/plutus-backups/:
#
#   plutus-2026-05-21-0330.dump      data backup, one per run
#   plutus-2026-05-21-0331.dump      ...
#   plutus-globals.sql               role definitions, refreshed each run.
#                                    Plain text (pg_dumpall's only mode);
#                                    tiny (~1KB) so we keep ONE current copy,
#                                    not one per day.
#
# Retention: keeps the most recent $RETAIN .dump files (default 14),
# deletes older. The .sql globals file is rewritten on each run.
#
# Why not bundle globals + dump into a tar per backup: roles barely
# change. Keeping one current globals.sql avoids 14 redundant copies
# of the same thing. If you need point-in-time globals, the file is
# also in the daily off-site rsync — restore from there.
#
# Usage (on the server):
#   ~/app/plutus/scripts/backup.sh
#   RETAIN=30 ~/app/plutus/scripts/backup.sh
#
# Usage (from your dev machine):
#   ssh noah@10.1.2.51 'bash ~/app/plutus/scripts/backup.sh'
#
# Restore — same host, replace data (role + DB already exist):
#   cat plutus-YYYY-MM-DD-HHMM.dump | podman exec -i plutus-postgres \
#       pg_restore -U plutus -d plutus --clean --if-exists
#
# Restore — fresh host (full migration):
#   1. ./scripts/bootstrap.sh on the new host
#      (creates plutus-postgres container, .env, role, empty plutus DB)
#   2. Stream the .dump in:
#      cat plutus-YYYY-MM-DD-HHMM.dump | ssh user@new-host \
#          'podman exec -i plutus-postgres pg_restore \
#              -U plutus -d plutus --clean --if-exists'
#   3. Optional: if .env / role password is also lost, restore globals first:
#      cat plutus-globals.sql | ssh user@new-host \
#          'podman exec -i plutus-postgres psql -U postgres -d postgres'

set -euo pipefail

CONTAINER="${PLUTUS_PG_CONTAINER:-plutus-postgres}"
DB_USER="${PLUTUS_PG_USER:-plutus}"
DB_NAME="${PLUTUS_PG_DB:-plutus}"
BACKUP_DIR="${PLUTUS_BACKUP_DIR:-$HOME/podman-volume/plutus-backups}"
RETAIN="${RETAIN:-14}"

mkdir -p "$BACKUP_DIR"

# UTC timestamp so backups sort lexicographically regardless of TZ.
# Format: YYYY-MM-DD-HHMM (sorts as text, no ambiguity).
TS=$(date -u +%Y-%m-%d-%H%M)
DATA_OUT="$BACKUP_DIR/plutus-${TS}.dump"
GLOBALS_OUT="$BACKUP_DIR/plutus-globals.sql"

# ── 1. Refresh globals (overwrites the single current copy) ──────────────
echo "==> dump globals (roles, etc.) → ${GLOBALS_OUT##*/}"
# Atomic write: dump to .tmp, then rename. If pg_dumpall errors, the old
# globals.sql stays intact.
podman exec "$CONTAINER" pg_dumpall \
    --username="$DB_USER" \
    --globals-only \
  > "$GLOBALS_OUT.tmp"
mv "$GLOBALS_OUT.tmp" "$GLOBALS_OUT"

# ── 2. The actual data backup, pg_dump custom format ─────────────────────
echo "==> dump $DB_NAME (custom format) → ${DATA_OUT##*/}"

# --format=custom: pg_dump's native binary container.
# --compress=9:    maximum compression. The container picks the best
#                  algorithm available (zstd if compiled in, else gzip).
# --clean --if-exists --create: lets pg_restore --clean drop and rebuild
#                  every object inside the target DB, idempotent.
podman exec "$CONTAINER" pg_dump \
    --username="$DB_USER" \
    --dbname="$DB_NAME" \
    --format=custom \
    --compress=9 \
    --clean \
    --if-exists \
    --create \
  > "$DATA_OUT.tmp"
mv "$DATA_OUT.tmp" "$DATA_OUT"

# Sanity check — empty backup = pg_dump silently failed.
size=$(stat -c%s "$DATA_OUT" 2>/dev/null || stat -f%z "$DATA_OUT")
if (( size < 256 )); then
  echo "ERROR: backup is $size bytes — pg_dump likely failed, removing" >&2
  rm -f "$DATA_OUT"
  exit 1
fi

# Verify the dump is well-formed by listing its TOC. pg_restore --list
# walks the archive header and prints a manifest; if the file is
# corrupt, it errors here instead of silently shipping a bad backup.
if ! podman exec -i "$CONTAINER" pg_restore --list < "$DATA_OUT" > /dev/null; then
  echo "ERROR: pg_restore --list failed on the backup — refusing to keep it" >&2
  rm -f "$DATA_OUT"
  exit 1
fi

# Pretty-print size.
human=$(awk -v b="$size" 'BEGIN{
  split("B KB MB GB",u);
  s=1;
  while (b >= 1024 && s < 4) { b /= 1024; s++ }
  printf "%.1f %s\n", b, u[s]
}')
echo "    wrote ${DATA_OUT##*/} ($human)"

# ── 3. Rotate old .dump files ────────────────────────────────────────────
mapfile -t backups < <(ls -t "$BACKUP_DIR"/plutus-*.dump 2>/dev/null)
total=${#backups[@]}
echo "    total .dump backups: $total (retaining newest $RETAIN)"

if (( total > RETAIN )); then
  for ((i = RETAIN; i < total; i++)); do
    rm -f "${backups[i]}"
    echo "    deleted ${backups[i]##*/}"
  done
fi

echo "==> done"
