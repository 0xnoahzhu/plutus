#!/usr/bin/env bash
# Back up the plutus postgres database via `pg_dump`.
#
# Plain SQL output, gzip-compressed. Plain SQL because at personal scale
# the backup is small (a few MB) and we get grep-able / `psql`-restorable
# files that don't need `pg_restore`. Custom-format (`-Fc`) is faster to
# restore at large scale but introduces a tool dependency we don't need.
#
# Output: ~/podman-volume/plutus-backups/plutus-YYYY-MM-DD-HHMM.sql.gz
# Retention: keeps the most recent $RETAIN backups (default 14), deletes
# older ones. The directory is on the same volume as pgdata, which means
# a disk failure loses both — see "Off-site copies" in deploy/README.md
# for syncing backups elsewhere.
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

echo "==> dumping $DB_NAME from $CONTAINER → $OUT"

# --no-owner / --no-privileges keep the dump portable: a restore into a
# different cluster won't try to `ALTER OWNER TO ...` for a role that
# doesn't exist there.
podman exec "$CONTAINER" pg_dump \
    --username="$DB_USER" \
    --dbname="$DB_NAME" \
    --format=plain \
    --no-owner \
    --no-privileges \
  | gzip --best > "$OUT"

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
