#!/usr/bin/env bash
# Deploy plutus to a remote podman host running Quadlet services.
#
# Sync source → rebuild affected images → restart the matching
# `systemctl --user` services → smoke-probe the public endpoints.
# Assumes the Quadlet `.container` files are already in place on the
# target (set up once, by hand or `bootstrap.sh`).
#
# Usage:
#   ./scripts/deploy.sh                  # sync + rebuild api+web, restart
#   ./scripts/deploy.sh --all            # also rebuild postgres
#   ./scripts/deploy.sh --only api       # only the API (skip web)
#   ./scripts/deploy.sh --only web
#   ./scripts/deploy.sh --only postgres
#   ./scripts/deploy.sh --skip-build     # sync + restart (no rebuild)
#   ./scripts/deploy.sh --skip-sync      # already-synced (e.g. retrying)
#   ./scripts/deploy.sh -h
#
# Env overrides:
#   DEPLOY_HOST       default noah@10.1.2.51 (ssh target)
#   DEPLOY_PATH       default ~/app/plutus     (remote source dir)
#   PLUTUS_WEB_URL    default http://10.1.2.51:4100 (smoke probe target)

set -euo pipefail

HOST="${DEPLOY_HOST:-noah@10.1.2.51}"
REMOTE_PATH="${DEPLOY_PATH:-~/app/plutus}"
WEB_URL="${PLUTUS_WEB_URL:-http://10.1.2.51:4100}"

# ── Arg parsing ──────────────────────────────────────────────────────────
ONLY=""
SKIP_BUILD=false
SKIP_SYNC=false
INCLUDE_POSTGRES=false
while [[ $# -gt 0 ]]; do
  case "$1" in
    --only)
      ONLY="${2:-}"
      [[ -z "$ONLY" ]] && { echo "--only needs a value" >&2; exit 1; }
      shift 2
      ;;
    --all)         INCLUDE_POSTGRES=true; shift ;;
    --skip-build)  SKIP_BUILD=true; shift ;;
    --skip-sync)   SKIP_SYNC=true;  shift ;;
    -h|--help)
      # Echo back the top-of-file comment as help.
      sed -n '2,/^$/p' "$0" | sed 's/^# \?//'
      exit 0
      ;;
    *)
      echo "unknown flag: $1 (try --help)" >&2
      exit 1
      ;;
  esac
done

# Resolve target list. Postgres is opt-in via --all or --only postgres
# because the image rarely changes (re-running its build is ~30s of
# wasted work).
case "$ONLY" in
  postgres) TARGETS=(postgres) ;;
  api)      TARGETS=(api) ;;
  web)      TARGETS=(web) ;;
  "")
    TARGETS=()
    $INCLUDE_POSTGRES && TARGETS+=(postgres)
    TARGETS+=(api web)
    ;;
  *)
    echo "--only must be one of: postgres|api|web (got: $ONLY)" >&2
    exit 1
    ;;
esac

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

echo "==> deploy targets: ${TARGETS[*]}"
echo "==> host:           $HOST"
echo "==> remote path:    $REMOTE_PATH"
START_TS=$(date +%s)

# ── Pre-flight: warn on dirty tree ───────────────────────────────────────
# Not fatal — sometimes you want to push WIP. Just a heads-up so the diff
# you're shipping matches what you remember.
if git -C "$REPO_ROOT" status --porcelain 2>/dev/null | grep -q .; then
  echo
  echo "    (heads-up: working tree has uncommitted changes — shipping them anyway)"
fi

# ── 1. Sync source ───────────────────────────────────────────────────────
if [[ "$SKIP_SYNC" != true ]]; then
  echo
  echo "==> sync source"
  rsync -az --delete \
    --exclude='target/' \
    --exclude='node_modules/' \
    --exclude='web/node_modules/' \
    --exclude='.git/' \
    --exclude='*.log' \
    --exclude='.DS_Store' \
    "$REPO_ROOT/" "$HOST:$REMOTE_PATH/"
fi

# ── 2. Build images on the remote ────────────────────────────────────────
if [[ "$SKIP_BUILD" != true ]]; then
  for t in "${TARGETS[@]}"; do
    echo
    echo "==> build $t"
    case "$t" in
      postgres)
        ssh "$HOST" "cd $REMOTE_PATH && podman build -f deploy/postgres/Dockerfile -t plutus-postgres:18 ./deploy/postgres/" 2>&1 \
          | tail -8
        ;;
      api)
        ssh "$HOST" "cd $REMOTE_PATH && podman build -f deploy/api/Dockerfile -t plutus-api:latest ." 2>&1 \
          | tail -8
        ;;
      web)
        ssh "$HOST" "cd $REMOTE_PATH/web && podman build -t plutus-web:latest ." 2>&1 \
          | tail -8
        ;;
    esac
  done
fi

# ── 3. Restart services ──────────────────────────────────────────────────
# Order matters: postgres first (others depend on it), then api, then web.
# TARGETS is already in that order from the parser above.
echo
echo "==> restart services"
for t in "${TARGETS[@]}"; do
  # `reset-failed` clears any prior crash-loop state so the unit starts
  # fresh; `restart` does the actual work.
  ssh "$HOST" "systemctl --user reset-failed plutus-$t.service 2>/dev/null; systemctl --user restart plutus-$t.service"
  echo "    restarted plutus-$t.service"
done

# ── 4. Verify ────────────────────────────────────────────────────────────
echo
echo "==> verify (giving units 8s to come up)"
sleep 8

all_ok=true
for t in "${TARGETS[@]}"; do
  status=$(ssh "$HOST" "systemctl --user is-active plutus-$t.service" 2>/dev/null || true)
  printf "    plutus-%-9s %s\n" "$t" "$status"
  [[ "$status" != "active" ]] && all_ok=false
done

# Public HTTP probe (web container is the only externally reachable face).
# If the deploy didn't touch web, this still validates the chain is up.
if curl -sf -o /dev/null -m 8 "$WEB_URL/login"; then
  echo "    GET $WEB_URL/login → OK"
else
  echo "    GET $WEB_URL/login → FAIL"
  all_ok=false
fi

# OpenAPI spec is generated at runtime by the API container, so an API
# deploy automatically refreshes /api/v1/docs and /api/v1/openapi.json.
# Make that explicit: when we touched the api, fetch the just-deployed
# spec and print the path/schema counts. Mismatch vs. local source means
# something is stale.
case " ${TARGETS[*]} " in
  *" api "*)
    api_url="$(echo "$WEB_URL" | sed 's/:[0-9]*$/:8080/')"
    spec=$(curl -sf -m 8 "$api_url/api/v1/openapi.json" 2>/dev/null || true)
    if [[ -n "$spec" ]]; then
      counts=$(printf '%s' "$spec" | python3 -c "
import json, sys
try:
  s = json.loads(sys.stdin.read())
  print(f\"paths={len(s.get('paths', {}))} schemas={len(s.get('components', {}).get('schemas', {}))}\")
except Exception as e:
  print(f'parse-failed: {e}')" 2>/dev/null || echo "parse-failed")
      echo "    GET $api_url/api/v1/openapi.json → $counts"
      echo "    docs:                $api_url/api/v1/docs"
    else
      echo "    GET $api_url/api/v1/openapi.json → FAIL (spec not reachable)"
      all_ok=false
    fi
    ;;
esac

elapsed=$(( $(date +%s) - START_TS ))
echo
if $all_ok; then
  echo "==> done in ${elapsed}s"
else
  echo "==> finished in ${elapsed}s with FAILURES — check logs:"
  echo "    ssh $HOST 'journalctl --user -u plutus-api.service -n 50'"
  exit 1
fi
