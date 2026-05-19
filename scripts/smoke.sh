#!/usr/bin/env bash
# Read-only smoke test. Hits every GET endpoint and asserts a 2xx.
#
# This script does NOT write to the database. If you want to exercise the
# write paths, do so against a throwaway database (e.g. spin up a separate
# postgres container or use a `plutus_test` database).
#
# Usage:
#   PLUTUS_API_URL=http://127.0.0.1:8080 bash scripts/smoke.sh

set -euo pipefail

API="${PLUTUS_API_URL:-http://127.0.0.1:8080}"

probe() {
  local label="$1"
  local path="$2"
  local expected_min_bytes="${3:-0}"

  local response_file
  response_file=$(mktemp)
  local response_code
  response_code=$(curl -s -o "$response_file" -w '%{http_code}' "$API$path")

  if [[ ! "$response_code" =~ ^2 ]]; then
    echo "FAIL $label: HTTP $response_code"
    cat "$response_file"
    rm -f "$response_file"
    exit 1
  fi

  local size
  size=$(wc -c < "$response_file" | tr -d ' ')
  if (( size < expected_min_bytes )); then
    echo "FAIL $label: response too small ($size < $expected_min_bytes bytes)"
    cat "$response_file"
    rm -f "$response_file"
    exit 1
  fi
  echo "  ✓ $label (HTTP $response_code, ${size}b)"
  rm -f "$response_file"
}

echo "smoke: targeting $API (read-only)"

probe "healthz"        /api/v1/healthz                                2
probe "openapi"        /api/v1/openapi.json                          100
probe "markets"        /api/v1/markets                                10
probe "brokers"        /api/v1/brokers                                10
probe "accounts"       /api/v1/accounts                                2
probe "stocks"         /api/v1/stocks                                  2
probe "watchlists"     /api/v1/watchlists                              2
probe "transactions"   /api/v1/transactions                            2
probe "holdings (fifo)"  '/api/v1/holdings?method=fifo'                2
probe "holdings (lifo)"  '/api/v1/holdings?method=lifo'                2
probe "holdings (avg)"   '/api/v1/holdings?method=average'             2
probe "fx"             /api/v1/fx                                      2
probe "audit"          /api/v1/audit                                   2

echo "all read checks passed"
